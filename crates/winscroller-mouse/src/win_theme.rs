//! Windows OS integration:
//! - Live Windows Accent Color & Dark/Light theme reading from Registry
//! - Fullscreen window detection (games/videos)
//! - Monitor refresh rate detection
//! - Start with Windows (Registry Run key)
//! - Clipboard text copy/paste (bounded to 4KB for safety)

use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{DataExchange::*, Memory::*, Registry::*},
        UI::{Shell::*, WindowsAndMessaging::*},
    },
    core::*,
};

use winscroller_mouse_core::colors::{ColorCtx, Rgb, hsl_deg, hue_of, mix_rgb};

/// Read a DWORD value from HKCU registry path.
fn read_hkcu_dword(subkey: PCWSTR, value_name: PCWSTR) -> Option<u32> {
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, Some(0), KEY_READ, &mut hkey).is_err() {
            return None;
        }
        let mut data: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;
        let res = RegQueryValueExW(
            hkey,
            value_name,
            None,
            None,
            Some(&mut data as *mut u32 as *mut u8),
            Some(&mut size),
        );
        let _ = RegCloseKey(hkey);
        if res.is_ok() { Some(data) } else { None }
    }
}

/// Read the current Windows DWM accent color & dark/light theme and populate a `ColorCtx`.
pub fn read_windows_theme() -> ColorCtx {
    // HKCU\Software\Microsoft\Windows\DWM\AccentColor is 0xAABBGGRR
    let accent: Rgb = if let Some(abgr) =
        read_hkcu_dword(w!("Software\\Microsoft\\Windows\\DWM"), w!("AccentColor"))
    {
        let r = (abgr & 0xFF) as f32;
        let g = ((abgr >> 8) & 0xFF) as f32;
        let b = ((abgr >> 16) & 0xFF) as f32;
        [r, g, b]
    } else if let Some(argb) = read_hkcu_dword(
        w!("Software\\Microsoft\\Windows\\DWM"),
        w!("ColorizationColor"),
    ) {
        let r = ((argb >> 16) & 0xFF) as f32;
        let g = ((argb >> 8) & 0xFF) as f32;
        let b = (argb & 0xFF) as f32;
        [r, g, b]
    } else {
        [122.0, 162.0, 247.0]
    };

    let light_mode = read_hkcu_dword(
        w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
        w!("AppsUseLightTheme"),
    )
    .map(|v| v != 0)
    .unwrap_or(false);

    let (background, foreground) = if light_mode {
        ([244.0, 244.0, 248.0], [28.0, 28.0, 34.0])
    } else {
        ([20.0, 20.0, 24.0], [228.0, 228.0, 236.0])
    };

    // Build a vibrant 8-color harmonious palette anchored on Windows' accent hue
    let h = hue_of(accent);
    let palette = vec![
        accent,
        hsl_deg(h + 38.0, 0.82, 0.66),
        hsl_deg(h + 85.0, 0.78, 0.62),
        hsl_deg(h + 135.0, 0.72, 0.60),
        hsl_deg(h + 185.0, 0.78, 0.64),
        hsl_deg(h + 230.0, 0.84, 0.66),
        hsl_deg(h + 280.0, 0.82, 0.68),
        mix_rgb(accent, foreground, 0.45),
    ];

    let border2 = hsl_deg(h + 52.0, 0.85, 0.65);

    ColorCtx {
        accent,
        foreground,
        background,
        palette,
        border: vec![accent, border2],
        border_angle: 45.0,
        custom: [122.0, 162.0, 247.0],
        custom2: [247.0, 118.0, 142.0],
        speed: 0.0,
        time: 0.0,
    }
}

/// Detect whether a fullscreen application (game, video player, etc.) is currently active.
pub fn is_fullscreen_active() -> bool {
    unsafe {
        // Fast check via Windows Shell notification state
        if let Ok(state) = SHQueryUserNotificationState()
            && (state == QUNS_BUSY
                || state == QUNS_RUNNING_D3D_FULL_SCREEN
                || state == QUNS_PRESENTATION_MODE)
        {
            return true;
        }

        let fg = GetForegroundWindow();
        if fg.0.is_null() {
            return false;
        }
        // Ignore desktop / shell windows
        let mut cls = [0u16; 64];
        let len = GetClassNameW(fg, &mut cls) as usize;
        if len > 0 {
            let name = String::from_utf16_lossy(&cls[..len]);
            if matches!(
                name.as_str(),
                "Progman"
                    | "WorkerW"
                    | "Shell_TrayWnd"
                    | "Shell_SecondaryTrayWnd"
                    | "WinscrollerMouse_Overlay"
            ) {
                return false;
            }
        }

        let mut win_rect = RECT::default();
        if GetWindowRect(fg, &mut win_rect).is_err() {
            return false;
        }

        let hmon = MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(hmon, &mut mi).as_bool() {
            return false;
        }
        let mr = mi.rcMonitor;

        // Fullscreen if window covers entire monitor without caption/border or exact monitor bounds
        win_rect.left <= mr.left
            && win_rect.top <= mr.top
            && win_rect.right >= mr.right
            && win_rect.bottom >= mr.bottom
    }
}

/// Query primary monitor refresh rate in Hz (fallback 60).
pub fn primary_refresh_rate() -> u32 {
    unsafe {
        let mut dm = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };
        if EnumDisplaySettingsW(None, ENUM_CURRENT_SETTINGS, &mut dm).as_bool()
            && dm.dmDisplayFrequency >= 30
        {
            dm.dmDisplayFrequency
        } else {
            60
        }
    }
}

/// Enable or disable Start with Windows via HKCU\Software\Microsoft\Windows\CurrentVersion\Run.
pub fn set_start_with_windows(enable: bool) {
    unsafe {
        let mut hkey = HKEY::default();
        if RegCreateKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            Some(0),
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        )
        .is_err()
        {
            return;
        }
        if enable {
            if let Ok(exe) = std::env::current_exe() {
                let cmd = format!("\"{}\"", exe.display());
                let wide: Vec<u16> = cmd.encode_utf16().chain(std::iter::once(0)).collect();
                let bytes = std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2);
                let _ = RegSetValueExW(hkey, w!("WinscrollerMouse"), Some(0), REG_SZ, Some(bytes));
            }
        } else {
            let _ = RegDeleteValueW(hkey, w!("WinscrollerMouse"));
        }
        let _ = RegCloseKey(hkey);
    }
}

/// Copy UTF-8 string to Windows clipboard.
pub fn clipboard_set_text(text: &str) -> bool {
    unsafe {
        if OpenClipboard(None).is_err() {
            return false;
        }
        let _ = EmptyClipboard();
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let byte_len = wide.len() * 2;
        if let Ok(hmem) = GlobalAlloc(GMEM_MOVEABLE, byte_len) {
            let ptr = GlobalLock(hmem) as *mut u16;
            if !ptr.is_null() {
                std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
                let _ = GlobalUnlock(hmem);
                let _ = SetClipboardData(13 /* CF_UNICODETEXT */, Some(HANDLE(hmem.0)));
                let _ = CloseClipboard();
                return true;
            }
        }
        let _ = CloseClipboard();
        false
    }
}

/// Read UTF-8 string from Windows clipboard (capped at 4096 chars).
pub fn clipboard_get_text() -> Option<String> {
    unsafe {
        if OpenClipboard(None).is_err() {
            return None;
        }
        let res = (|| {
            let handle = GetClipboardData(13 /* CF_UNICODETEXT */).ok()?;
            let hmem = HGLOBAL(handle.0);
            let ptr = GlobalLock(hmem) as *const u16;
            if ptr.is_null() {
                return None;
            }
            let mut len = 0usize;
            while len < 4096 && *ptr.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(ptr, len);
            let s = String::from_utf16_lossy(slice);
            let _ = GlobalUnlock(hmem);
            Some(s)
        })();
        let _ = CloseClipboard();
        res
    }
}
