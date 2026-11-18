//! System tray icon (procedural Plume/Winscroller icon) + full Windows context menu.
//! Interactions match `BarWidget.qml`:
//! - Left click: toggle effects on/off
//! - Right click: context menu (Settings, Quick Looks, Quick Cursors, Find, Next Look, Reset Cursor, Quit)
//! - Middle click: trigger Find pointer rings
//! - Double click: open Settings panel

use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        UI::{Shell::*, WindowsAndMessaging::*},
    },
    core::*,
};

use winscroller_mouse_core::settings::{builtin_cursors, builtin_looks};

pub const WM_TRAY_CALLBACK: u32 = WM_APP + 10;

pub const CMD_TOGGLE: u32 = 1001;
pub const CMD_SETTINGS: u32 = 1002;
pub const CMD_NEXT_LOOK: u32 = 1003;
pub const CMD_FIND: u32 = 1004;
pub const CMD_UNDO: u32 = 1005;
pub const CMD_RESET_CURSOR: u32 = 1006;
pub const CMD_QUIT: u32 = 1007;

pub const CMD_LOOK_BASE: u32 = 2000; // 2000..2011
pub const CMD_CURSOR_BASE: u32 = 2100; // 2100..2115

const TRAY_ID: u32 = 1;
const TRAY_CLASS: PCWSTR = w!("WinscrollerMouse_TrayWnd");

pub struct TrayIcon {
    pub hwnd: HWND,
    icon: HICON,
}

/// Create a crisp 32x32 procedural feather/cursor icon (no external .ico file needed!).
unsafe fn create_procedural_icon(accent: [u8; 3]) -> HICON {
    let w = 32i32;
    let h = 32i32;
    let mut pixels = vec![0u8; (w * h * 4) as usize];

    // Draw a glowing rounded badge + sleek arrow silhouette in BGRA
    let cx = 16.0f32;
    let cy = 16.0f32;
    for y in 0..h {
        for x in 0..w {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            let dist = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt();
            let idx = ((y * w + x) * 4) as usize;

            // Background circle
            if dist <= 15.0 {
                let edge_aa = (15.0 - dist).clamp(0.0, 1.0);
                // Soft glow ring around border
                let rim = ((dist - 12.2) / 2.2).clamp(0.0, 1.0);
                let r = (22.0 * (1.0 - rim) + accent[0] as f32 * rim) as u8;
                let g = (24.0 * (1.0 - rim) + accent[1] as f32 * rim) as u8;
                let b = (34.0 * (1.0 - rim) + accent[2] as f32 * rim) as u8;
                let a = (edge_aa * 245.0) as u8;
                pixels[idx] = b;
                pixels[idx + 1] = g;
                pixels[idx + 2] = r;
                pixels[idx + 3] = a;
            }

            // Classic crisp pointer arrow inside
            // Triangle from tip (10,7) -> (10,23) -> (22,17) minus notch
            let px = fx - 10.0;
            let py = fy - 7.0;
            if px >= 0.0 && (0.0..=17.5).contains(&py) {
                let right_edge = py * 0.78;
                let bottom_cut = 13.5 - px * 0.25;
                let in_head = px <= right_edge && py <= bottom_cut + 3.5;
                let in_left = px <= right_edge && py <= 16.5 && px <= (16.5 - py) * 1.1 + 4.5;
                let in_tail = (4.2..=8.2).contains(&px)
                    && (12.0..=18.0).contains(&py)
                    && ((px - 4.5) - (py - 12.0) * 0.45).abs() < 1.65;
                if (in_head && in_left) || in_tail {
                    pixels[idx] = 250;
                    pixels[idx + 1] = 248;
                    pixels[idx + 2] = 245;
                    pixels[idx + 3] = 255;
                }
            }
        }
    }

    let bi = BITMAPV5HEADER {
        bV5Size: std::mem::size_of::<BITMAPV5HEADER>() as u32,
        bV5Width: w,
        bV5Height: -h, // top-down DIB
        bV5Planes: 1,
        bV5BitCount: 32,
        bV5Compression: BI_BITFIELDS,
        bV5RedMask: 0x00FF_0000,
        bV5GreenMask: 0x0000_FF00,
        bV5BlueMask: 0x0000_00FF,
        bV5AlphaMask: 0xFF00_0000,
        ..Default::default()
    };

    let hdc = GetDC(None);
    let mut bits_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let hbm_color = CreateDIBSection(
        Some(hdc),
        &bi as *const BITMAPV5HEADER as *const BITMAPINFO,
        DIB_RGB_COLORS,
        &mut bits_ptr,
        None,
        0,
    );
    let _ = ReleaseDC(None, hdc);

    if let Ok(hbm_color) = hbm_color {
        if !bits_ptr.is_null() {
            std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits_ptr as *mut u8, pixels.len());
        }
        let hbm_mask = CreateBitmap(w, h, 1, 1, None);
        let ii = ICONINFO {
            fIcon: TRUE,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: hbm_mask,
            hbmColor: hbm_color,
        };
        let icon = CreateIconIndirect(&ii).unwrap_or_default();
        let _ = DeleteObject(hbm_color.into());
        let _ = DeleteObject(hbm_mask.into());
        if !icon.is_invalid() {
            return icon;
        }
    }
    LoadIconW(None, IDI_APPLICATION).unwrap_or_default()
}

pub unsafe fn create_tray(instance: HINSTANCE, wndproc: WNDPROC, accent: [u8; 3]) -> TrayIcon {
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpszClassName: TRAY_CLASS,
        hInstance: instance,
        lpfnWndProc: wndproc,
        ..Default::default()
    };
    RegisterClassExW(&wc);

    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        TRAY_CLASS,
        w!("Winscroller Mouse"),
        WS_OVERLAPPED,
        0,
        0,
        0,
        0,
        Some(HWND_MESSAGE),
        None,
        Some(instance),
        None,
    )
    .unwrap_or_default();

    let icon = create_procedural_icon(accent);

    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        uFlags: NIF_ICON | NIF_TIP | NIF_MESSAGE,
        uCallbackMessage: WM_TRAY_CALLBACK,
        hIcon: icon,
        ..Default::default()
    };
    set_tip(
        &mut nid,
        "Winscroller Mouse — Left-click: Toggle · Right-click: Menu",
    );
    let _ = Shell_NotifyIconW(NIM_ADD, &nid);

    TrayIcon { hwnd, icon }
}

pub unsafe fn update_tray_tip(hwnd: HWND, enabled: bool) {
    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        uFlags: NIF_TIP,
        ..Default::default()
    };
    let text = if enabled {
        "Winscroller Mouse (Active) — Left: Toggle · Right: Menu"
    } else {
        "Winscroller Mouse (Paused) — Left: Toggle · Right: Menu"
    };
    set_tip(&mut nid, text);
    let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
}

fn set_tip(nid: &mut NOTIFYICONDATAW, text: &str) {
    let tip: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let len = tip.len().min(nid.szTip.len());
    nid.szTip[..len].copy_from_slice(&tip[..len]);
    nid.szTip[nid.szTip.len() - 1] = 0;
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            let nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ID,
                ..Default::default()
            };
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
            let _ = DestroyIcon(self.icon);
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

/// Show the right-click tray menu at the cursor position and return the chosen command ID (or 0).
pub unsafe fn show_tray_menu(hwnd: HWND, enabled: bool, has_undo: bool) -> u32 {
    let Ok(menu) = CreatePopupMenu() else {
        return 0;
    };
    let Ok(looks_sub) = CreatePopupMenu() else {
        let _ = DestroyMenu(menu);
        return 0;
    };
    let Ok(cursors_sub) = CreatePopupMenu() else {
        let _ = DestroyMenu(looks_sub);
        let _ = DestroyMenu(menu);
        return 0;
    };

    for (i, lk) in builtin_looks().iter().enumerate() {
        let w: Vec<u16> = lk.label.encode_utf16().chain(std::iter::once(0)).collect();
        let _ = AppendMenuW(
            looks_sub,
            MF_STRING,
            (CMD_LOOK_BASE as usize) + i,
            PCWSTR(w.as_ptr()),
        );
    }
    for (i, ck) in builtin_cursors().iter().enumerate() {
        let w: Vec<u16> = ck.label.encode_utf16().chain(std::iter::once(0)).collect();
        let _ = AppendMenuW(
            cursors_sub,
            MF_STRING,
            (CMD_CURSOR_BASE as usize) + i,
            PCWSTR(w.as_ptr()),
        );
    }

    let _ = AppendMenuW(
        menu,
        MF_STRING,
        CMD_SETTINGS as usize,
        w!("Settings Panel..."),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING | if enabled { MF_CHECKED } else { MF_UNCHECKED },
        CMD_TOGGLE as usize,
        w!("Effects Enabled"),
    );
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
    let _ = AppendMenuW(menu, MF_POPUP, looks_sub.0 as usize, w!("Quick Looks"));
    let _ = AppendMenuW(
        menu,
        MF_POPUP,
        cursors_sub.0 as usize,
        w!("Ready-Made Cursors"),
    );
    let _ = AppendMenuW(menu, MF_STRING, CMD_NEXT_LOOK as usize, w!("Next Look"));
    let _ = AppendMenuW(menu, MF_STRING, CMD_FIND as usize, w!("Find Pointer"));
    if has_undo {
        let _ = AppendMenuW(
            menu,
            MF_STRING,
            CMD_UNDO as usize,
            w!("Undo Last Look/Cursor"),
        );
    }
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        CMD_RESET_CURSOR as usize,
        w!("Restore System Cursor"),
    );
    let _ = AppendMenuW(
        menu,
        MF_STRING,
        CMD_QUIT as usize,
        w!("Quit Winscroller Mouse"),
    );

    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);

    let _ = SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(
        menu,
        TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_NONOTIFY,
        pt.x,
        pt.y,
        Some(0),
        hwnd,
        None,
    );
    let _ = DestroyMenu(menu);
    cmd.0 as u32
}
