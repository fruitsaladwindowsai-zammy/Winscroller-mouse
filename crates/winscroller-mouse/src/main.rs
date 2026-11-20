//! Winscroller Mouse — Faithful Rust + DirectComposition Windows clone of omarchy-plume.
//!
//! Features:
//! - Hardware-accelerated per-pixel transparent DirectComposition + Direct2D overlays (multi-monitor)
//! - 6 Trails (Comet, Beads, Ribbon, Sparks, Echoes, Bubbles) with exact Catmull-Rom Band.js smoothing
//! - 6 Halos (Glow, Ring, Pulse, Spotlight, Crosshair, Target) + Always/Moving/Still visibility
//! - 8 Color modes (Theme, Palette, Border, Custom, Gradient, Rainbow, Drift, Speed) synced live to Windows
//! - Shake-to-Find radar rings
//! - Full Custom Cursor Forge: 14 Arrows, 30 Surfaces, 22 Lines, 18 Ornaments, 5 Shadows, animated .ani spinners
//! - 12 Ready-Made Looks, 16 Ready-Made Cursors, 12 User Saved Slots, Undo, Copy/Paste Look JSON
//! - Full 8-Tab GPU-accelerated Settings Panel (0 MB overhead when closed)
//! - Fullscreen game/video auto-hide, idle/typing cursor hiding, Named-Pipe CLI IPC, and low-memory idle trimming

#![windows_subsystem = "windows"]
#![allow(unsafe_op_in_unsafe_fn)]

mod ipc;
mod renderer;
mod settings_ui;
mod store;
mod tray;
mod win_cursor;
mod win_theme;

use std::cell::RefCell;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{Com::*, Console::*, LibraryLoader::*, Threading::*},
        UI::{HiDpi::*, WindowsAndMessaging::*},
    },
    core::*,
};

use ipc::{IpcRequest, send_cli_command, spawn_ipc_server};
use renderer::{GpuContext, OverlayWindow};
use settings_ui::{SettingsPanel, UiEvents};
use store::{app_data_dir, load_state, save_state};
use tray::{
    CMD_CURSOR_BASE, CMD_FIND, CMD_LOOK_BASE, CMD_NEXT_LOOK, CMD_QUIT, CMD_RESET_CURSOR,
    CMD_SETTINGS, CMD_TOGGLE, CMD_UNDO, WM_TRAY_CALLBACK, create_tray, show_tray_menu,
    show_tray_notification, update_tray_tip,
};
use win_cursor::{apply_custom_cursors, hide_system_cursor, restore_system_cursors};
use win_theme::{
    is_fullscreen_active, primary_refresh_rate, read_windows_theme, set_start_with_windows,
};
use winscroller_mouse_core::{
    colors::{ColorCtx, parse_hex, to_hex},
    draw::DrawCtx,
    halos,
    motion::{MotionState, Point},
    paint::{FrameState, paint_frame_into},
    settings::{AppStateFile, SLOTS, Settings, builtin_cursors, builtin_looks},
    trails::TrailMemory,
};

const OVERLAY_CLASS: PCWSTR = w!("WinscrollerMouse_Overlay");

// Global atomic timestamp (ms since app start) updated by keyboard hook for "Hide while typing"
static LAST_KEY_MS: AtomicU64 = AtomicU64::new(0);
static APP_START: OnceLock<Instant> = OnceLock::new();

fn now_ms_global() -> u64 {
    APP_START
        .get()
        .map(|st| st.elapsed().as_millis() as u64)
        .unwrap_or(0)
}

fn install_panic_logger() {
    std::panic::set_hook(Box::new(|info| {
        // Always restore system cursors on panic so the user is never stuck with a hidden/custom cursor
        restore_system_cursors();
        let dir = app_data_dir();
        let _ = std::fs::create_dir_all(&dir);
        let log_path = dir.join("crash.log");
        let msg = format!(
            "[Winscroller Mouse v{}] PANIC: {}\n",
            env!("CARGO_PKG_VERSION"),
            info
        );
        let _ = std::fs::write(log_path, msg);
    }));
}

fn attach_parent_console() {
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

fn print_help() {
    println!(
        "Winscroller Mouse v{}\n\
         Native Windows 10/11 custom cursor forge and GPU pointer effects\n\n\
         USAGE:\n\
           winscroller-mouse.exe [COMMAND]\n\n\
         COMMANDS:\n\
           (none)              Start daemon and open Settings Panel (or focus if running)\n\
           settings            Open or focus the 8-tab Settings Panel\n\
           --silent | --daemon Run in background without opening Settings Panel\n\
           toggle | on | off   Toggle pointer effects on/off\n\
           find                Trigger Shake-to-Find radar rings\n\
           look <name>         Apply one of 12 ready-made looks (e.g., classic, neon, ember)\n\
           nextLook            Cycle to the next ready-made look\n\
           undo                Undo the last look/cursor change\n\
           cursor <name>       Apply one of 16 ready-made cursors (or 'default' to restore)\n\
           set <key> <value>   Set any setting key (e.g., set trail sparks)\n\
           save <1..12>        Save all settings to slot 1..12 (or saveNew)\n\
           load <1..12>        Load all settings from slot 1..12\n\
           exportLook          Print current look JSON\n\
           importLook <json>   Import a look JSON\n\
           status              Print current settings as JSON\n\
           quit                Cleanly exit and restore Windows system cursors\n\
           --version | -V      Print version information\n\
           --help | -h         Show this help message",
        env!("CARGO_PKG_VERSION")
    );
}

unsafe extern "system" fn low_level_keyboard_proc(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code >= 0 && (wp.0 as u32 == WM_KEYDOWN || wp.0 as u32 == WM_SYSKEYDOWN) {
        LAST_KEY_MS.store(now_ms_global(), Ordering::Relaxed);
    }
    CallNextHookEx(None, code, wp, lp)
}

// Tray events queued from WndProc into main loop
#[derive(Clone, Copy)]
enum TrayEvent {
    LeftClick,
    RightClick,
    MiddleClick,
    DoubleClick,
    DisplayChanged,
}

thread_local! {
    static TRAY_EVENTS: RefCell<Vec<TrayEvent>> = const { RefCell::new(Vec::new()) };
}

unsafe extern "system" fn tray_wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_TRAY_CALLBACK => {
            let ev = (lp.0 & 0xFFFF) as u32;
            match ev {
                WM_LBUTTONUP => TRAY_EVENTS.with(|q| q.borrow_mut().push(TrayEvent::LeftClick)),
                WM_RBUTTONUP => TRAY_EVENTS.with(|q| q.borrow_mut().push(TrayEvent::RightClick)),
                WM_MBUTTONUP => TRAY_EVENTS.with(|q| q.borrow_mut().push(TrayEvent::MiddleClick)),
                WM_LBUTTONDBLCLK => {
                    TRAY_EVENTS.with(|q| q.borrow_mut().push(TrayEvent::DoubleClick))
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_DISPLAYCHANGE => {
            TRAY_EVENTS.with(|q| q.borrow_mut().push(TrayEvent::DisplayChanged));
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

unsafe extern "system" fn overlay_wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
        WM_ERASEBKGND => LRESULT(1),
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

fn main() {
    install_panic_logger();
    let args: Vec<String> = std::env::args().skip(1).collect();

    if let Some(first) = args.first().map(|s| s.as_str()) {
        if matches!(first, "--version" | "-V" | "version") {
            attach_parent_console();
            println!("winscroller-mouse {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        if matches!(first, "--help" | "-h" | "help") {
            attach_parent_console();
            print_help();
            return;
        }
    }

    // If invoked with CLI arguments, forward directly to running instance if present
    if !args.is_empty() {
        if let Some(resp) = send_cli_command(&args) {
            if !resp.is_empty() {
                attach_parent_console();
                println!("{resp}");
            }
            return;
        }
        if matches!(args[0].as_str(), "quit" | "stop" | "status") {
            return;
        }
    }

    // Single-instance mutex — if already running with no CLI args, open its Settings panel!
    let _mutex = unsafe {
        let m = CreateMutexW(None, true, w!("Local\\WinscrollerMouse_SingleInstance"));
        if GetLastError() == ERROR_ALREADY_EXISTS {
            let _ = send_cli_command(&["settings".into()]);
            return;
        }
        m.ok()
    };

    let _ = APP_START.set(Instant::now());
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    let instance: HINSTANCE = unsafe { GetModuleHandleW(None).unwrap().into() };

    // Register overlay window class
    unsafe {
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpszClassName: OVERLAY_CLASS,
            hInstance: instance,
            lpfnWndProc: Some(overlay_wndproc),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            style: CS_HREDRAW | CS_VREDRAW,
            ..Default::default()
        };
        RegisterClassExW(&wc);
    }

    let gpu = unsafe {
        GpuContext::new().expect("Failed to initialize Direct3D11/Direct2D/DirectComposition")
    };
    let mut overlays = unsafe { create_monitor_overlays(&gpu, instance) };

    let mut state: AppStateFile = load_state();
    let mut undo_buf: Option<Settings> = None;
    let mut win_theme: ColorCtx = read_windows_theme();
    let mut monitor_hz = primary_refresh_rate();

    let accent_u8 = [
        win_theme.accent[0] as u8,
        win_theme.accent[1] as u8,
        win_theme.accent[2] as u8,
    ];
    let tray = unsafe { create_tray(instance, Some(tray_wndproc), accent_u8) };
    unsafe {
        update_tray_tip(tray.hwnd, state.settings.enabled);
    }

    // Spawn CLI named pipe server
    let (ipc_tx, ipc_rx) = mpsc::channel::<IpcRequest>();
    spawn_ipc_server(ipc_tx);

    // Optional keyboard hook for "Hide while typing"
    let kbd_hook =
        unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), None, 0).ok() };

    // Core runtime state
    let mut motion = MotionState::new();
    let mut trail_mem = TrailMemory::new();
    let mut draw_ctx = DrawCtx::new(1920.0, 1080.0);
    let mut pts_buf: Vec<Point> = Vec::with_capacity(160);
    let mut settings_panel: Option<SettingsPanel> = None;

    let mut find_start: Option<Instant> = None;
    let mut look_cycle_idx: usize = 0;

    // Cursor management state
    let mut applied_cursor_hash: u64 = 0;
    let mut custom_cursor_active = false;
    let mut cursor_hidden = false;
    let mut pending_cursor_rebuild: Option<Instant> = if state.settings.is_custom_cursor() {
        Some(Instant::now())
    } else {
        None
    };

    // Periodic timers (theme sync, fullscreen check, topmost check, DXGI trim)
    let start = Instant::now();
    let mut last_frame = Instant::now();
    let mut last_theme_poll = Instant::now();
    let mut last_fs_poll = Instant::now();
    let mut last_topmost_poll = Instant::now();
    let mut idle_since: Option<Instant> = None;
    let mut trimmed_while_idle = false;
    let mut fullscreen_suppressed = false;
    let mut running = true;

    let is_daemon = args
        .iter()
        .any(|s| matches!(s.as_str(), "--silent" | "--daemon" | "daemon"));
    let want_settings =
        !is_daemon && (args.is_empty() || args.first().map(|s| s.as_str()) == Some("settings"));

    // If double-clicked / started interactively or launched with `settings`, open Settings panel right away
    if want_settings {
        settings_panel = unsafe { SettingsPanel::open(&gpu, instance).ok() };
    }

    if !is_daemon {
        unsafe {
            show_tray_notification(
                tray.hwnd,
                "Winscroller Mouse Active",
                "Pointer trails & effects are active! Double-click tray icon to customize.",
            );
        }
    }

    // ---- Main Loop ----------------------------------------------------------
    while running {
        // 1. Pump all Windows messages without blocking
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                if msg.message == WM_QUIT {
                    running = false;
                    break;
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        if !running {
            break;
        }

        // 2. Process tray icon clicks & display change events
        let tray_evs: Vec<TrayEvent> = TRAY_EVENTS.with(|q| q.borrow_mut().drain(..).collect());
        for ev in tray_evs {
            match ev {
                TrayEvent::LeftClick => {
                    state.settings.enabled = !state.settings.enabled;
                    save_state(&state);
                    unsafe {
                        update_tray_tip(tray.hwnd, state.settings.enabled);
                    }
                }
                TrayEvent::MiddleClick => {
                    find_start = Some(Instant::now());
                }
                TrayEvent::DoubleClick => {
                    open_or_focus_settings(&mut settings_panel, &gpu, instance);
                }
                TrayEvent::RightClick => {
                    let cmd = unsafe {
                        show_tray_menu(tray.hwnd, state.settings.enabled, undo_buf.is_some())
                    };
                    handle_menu_cmd(
                        cmd,
                        &mut state,
                        &mut undo_buf,
                        &mut look_cycle_idx,
                        &mut find_start,
                        &mut settings_panel,
                        &mut pending_cursor_rebuild,
                        &mut custom_cursor_active,
                        &mut running,
                        &gpu,
                        instance,
                        tray.hwnd,
                    );
                }
                TrayEvent::DisplayChanged => {
                    monitor_hz = primary_refresh_rate();
                    overlays = unsafe { create_monitor_overlays(&gpu, instance) };
                }
            }
        }

        // 3. Process Named-Pipe CLI requests
        while let Ok(req) = ipc_rx.try_recv() {
            let resp = handle_ipc_cmd(
                &req.command,
                &mut state,
                &mut undo_buf,
                &mut look_cycle_idx,
                &mut find_start,
                &mut settings_panel,
                &mut pending_cursor_rebuild,
                &mut custom_cursor_active,
                &mut running,
                &gpu,
                instance,
                tray.hwnd,
            );
            let _ = req.reply.send(resp);
        }

        // 4. Periodic Windows theme refresh (every 3s)
        if last_theme_poll.elapsed() >= Duration::from_secs(3) {
            last_theme_poll = Instant::now();
            let new_theme = read_windows_theme();
            if new_theme.accent != win_theme.accent || new_theme.background != win_theme.background
            {
                win_theme = new_theme;
                if state.settings.is_custom_cursor() {
                    pending_cursor_rebuild = Some(Instant::now());
                }
            }
        }

        // 5. Periodic fullscreen detection (every 250ms)
        if last_fs_poll.elapsed() >= Duration::from_millis(250) {
            last_fs_poll = Instant::now();
            fullscreen_suppressed = state.settings.fullscreen == "hide" && is_fullscreen_active();
        }

        // 6. Tick Settings Panel if open
        if let Some(panel) = settings_panel.as_mut() {
            let mut ui_ev = UiEvents::default();
            let keep_open = unsafe {
                panel.tick_and_render(&gpu, &mut state, &mut undo_buf, &win_theme, &mut ui_ev)
            };
            if ui_ev.trigger_find {
                find_start = Some(Instant::now());
            }
            if ui_ev.restore_cursor {
                restore_system_cursors();
                custom_cursor_active = false;
                applied_cursor_hash = 0;
                pending_cursor_rebuild = None;
            }
            if let Some(sw) = ui_ev.start_with_windows_changed {
                set_start_with_windows(sw);
            }
            if ui_ev.settings_changed {
                save_state(&state);
                unsafe {
                    update_tray_tip(tray.hwnd, state.settings.enabled);
                }
                if state.settings.is_custom_cursor() {
                    pending_cursor_rebuild = Some(Instant::now() + Duration::from_millis(160));
                } else if custom_cursor_active {
                    restore_system_cursors();
                    custom_cursor_active = false;
                    applied_cursor_hash = 0;
                }
            }
            if !keep_open {
                // Dropping SettingsPanel destroys its window & D2D resources immediately (0 MB idle!)
                settings_panel = None;
                unsafe {
                    gpu.trim();
                }
            }
        }

        // 7. Debounced custom cursor build & apply
        if let Some(when) = pending_cursor_rebuild
            && Instant::now() >= when
        {
            pending_cursor_rebuild = None;
            if state.settings.is_custom_cursor() {
                let h = state.settings.cursor_fingerprint(
                    &to_hex(win_theme.accent),
                    &to_hex(win_theme.foreground),
                    &to_hex(win_theme.background),
                    &to_hex(
                        win_theme
                            .border
                            .first()
                            .copied()
                            .unwrap_or(win_theme.accent),
                    ),
                );
                if h != applied_cursor_hash {
                    apply_custom_cursors(&state.settings, &win_theme);
                    applied_cursor_hash = h;
                    custom_cursor_active = true;
                    cursor_hidden = false;
                }
            }
        }

        // 8. Poll cursor position & update motion
        let mut pt = POINT::default();
        unsafe {
            let _ = GetCursorPos(&mut pt);
        }
        let now_ms = start.elapsed().as_secs_f64() * 1000.0;
        let time_secs = start.elapsed().as_secs_f64();

        let moved = motion.add(pt.x as f32, pt.y as f32, now_ms);

        // Hide-when-idle / Hide-while-typing cursor logic
        let still_secs = motion.still_for(now_ms);
        let want_hide_idle = state
            .settings
            .hide_after_secs()
            .map(|limit| still_secs >= limit)
            .unwrap_or(false);
        let last_key = LAST_KEY_MS.load(Ordering::Relaxed);
        let want_hide_typing = state.settings.hide_typing == "on"
            && last_key > 0
            && (now_ms as u64).saturating_sub(last_key) < 1500
            && still_secs > 0.12;
        let want_hide = want_hide_idle || want_hide_typing;

        if moved && cursor_hidden {
            if state.settings.is_custom_cursor() {
                apply_custom_cursors(&state.settings, &win_theme);
                custom_cursor_active = true;
            } else {
                restore_system_cursors();
            }
            cursor_hidden = false;
        } else if want_hide && !cursor_hidden && settings_panel.is_none() {
            hide_system_cursor();
            cursor_hidden = true;
        }

        // Shake-to-find detection
        if state.settings.enabled
            && state.settings.shake
            && !fullscreen_suppressed
            && motion.shaken(now_ms, state.settings.shake_sensitivity)
        {
            find_start = Some(Instant::now());
        }

        let find_age = find_start
            .map(|s| s.elapsed().as_secs_f32())
            .unwrap_or(-1.0);
        if find_age > halos::FIND_LENGTH + 0.1 {
            find_start = None;
        }

        motion.prune(now_ms, state.settings.trail_life_secs());

        let dt = last_frame.elapsed().as_secs_f32().clamp(0.001, 0.05);
        last_frame = Instant::now();

        // 9. Render desktop overlays (or clear & sleep when disabled/fullscreen/idle!)
        let active_effects = state.settings.enabled && !fullscreen_suppressed;
        let mut any_busy = false;

        if !active_effects {
            for ov in &mut overlays {
                unsafe {
                    let _ = ov.clear_and_present();
                }
            }
        } else {
            // Keep overlays above newly opened popups/taskbar every 2s while active
            if moved && last_topmost_poll.elapsed() >= Duration::from_secs(2) {
                last_topmost_poll = Instant::now();
                for ov in &overlays {
                    unsafe {
                        ov.ensure_topmost();
                    }
                }
            }

            let mut frame_cctx = win_theme.clone();
            frame_cctx.speed = motion.speed01();
            frame_cctx.time = time_secs;
            if let Some(c) = parse_hex(&state.settings.custom_color) {
                frame_cctx.custom = c;
            }
            if let Some(c) = parse_hex(&state.settings.custom_color2) {
                frame_cctx.custom2 = c;
            }

            let vsync = overlays.len() == 1 && settings_panel.is_none();

            for ov in &mut overlays {
                let ox = ov.x as f32;
                let oy = ov.y as f32;
                let ow = ov.width as f32;
                let oh = ov.height as f32;

                let cursor_on = pt.x >= ov.x
                    && pt.x < ov.x + ov.width as i32
                    && pt.y >= ov.y
                    && pt.y < ov.y + ov.height as i32;

                motion.points_for_into(
                    now_ms,
                    state.settings.trail_life_secs(),
                    ox,
                    oy,
                    &mut pts_buf,
                );

                let mut fs = FrameState {
                    settings: &state.settings,
                    pts: &pts_buf,
                    cursor_x: pt.x as f32,
                    cursor_y: pt.y as f32,
                    screen_x: ox,
                    screen_y: oy,
                    screen_w: ow,
                    screen_h: oh,
                    cursor_on_this_screen: cursor_on,
                    color_ctx: &frame_cctx,
                    time_secs,
                    dt,
                    moved: motion.moved,
                    still_secs,
                    find_age,
                    trail_mem: &mut trail_mem,
                };

                let busy = paint_frame_into(&mut draw_ctx, &mut fs);
                if busy {
                    any_busy = true;
                }

                // Skip re-rendering if nothing moved and nothing is animating and overlay didn't change
                if draw_ctx.cmds.is_empty() {
                    unsafe {
                        let _ = ov.clear_and_present();
                    }
                } else if moved || busy || ov.is_clear || find_age >= 0.0 {
                    unsafe {
                        let _ = ov.render(&gpu, &draw_ctx.cmds, vsync);
                    }
                }
            }
        }

        // 10. Smart low-CPU / low-memory sleep & DXGI trim
        let is_animating = moved || any_busy || settings_panel.is_some() || find_age >= 0.0;
        if is_animating {
            idle_since = None;
            trimmed_while_idle = false;
            let fps = state.settings.target_fps(monitor_hz);
            let target = Duration::from_micros(1_000_000 / fps.max(30) as u64);
            let spent = last_frame.elapsed();
            if spent < target {
                std::thread::sleep(target - spent);
            }
        } else {
            let since = *idle_since.get_or_insert_with(Instant::now);
            if !trimmed_while_idle && since.elapsed() >= Duration::from_secs(4) {
                unsafe {
                    gpu.trim();
                }
                trimmed_while_idle = true;
            }
            // Low-power idle poll (25ms -> ~0% CPU, instant wakeup on cursor move!)
            std::thread::sleep(Duration::from_millis(25));
        }
    }

    // ---- Clean shutdown: restore Windows system cursors & unhook ------------
    if custom_cursor_active || cursor_hidden {
        restore_system_cursors();
    }
    if let Some(h) = kbd_hook {
        unsafe {
            let _ = UnhookWindowsHookEx(h);
        }
    }
    drop(settings_panel);
    drop(overlays);
    drop(tray);
}

unsafe fn create_monitor_overlays(gpu: &GpuContext, instance: HINSTANCE) -> Vec<OverlayWindow> {
    struct MonList(Vec<(i32, i32, u32, u32)>);
    let mut list = MonList(Vec::new());

    unsafe extern "system" fn enum_cb(
        _hmon: HMONITOR,
        _hdc: HDC,
        rect: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        if !rect.is_null() {
            let r = *rect;
            let out = &mut *(lparam.0 as *mut MonList);
            let w = (r.right - r.left).max(1) as u32;
            let h = (r.bottom - r.top).max(1) as u32;
            out.0.push((r.left, r.top, w, h));
        }
        TRUE
    }

    let _ = EnumDisplayMonitors(
        None,
        None,
        Some(enum_cb),
        LPARAM(&mut list as *mut _ as isize),
    );

    let mut overlays = Vec::with_capacity(list.0.len());
    for (x, y, w, h) in list.0 {
        if let Ok(ov) = OverlayWindow::new(gpu, x, y, w, h, instance, OVERLAY_CLASS) {
            overlays.push(ov);
        }
    }
    overlays
}

fn open_or_focus_settings(
    panel: &mut Option<SettingsPanel>,
    gpu: &GpuContext,
    instance: HINSTANCE,
) {
    if let Some(p) = panel.as_ref() {
        unsafe {
            let _ = ShowWindow(p.hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(p.hwnd);
        }
    } else {
        *panel = unsafe { SettingsPanel::open(gpu, instance).ok() };
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_menu_cmd(
    cmd: u32,
    state: &mut AppStateFile,
    undo_buf: &mut Option<Settings>,
    look_idx: &mut usize,
    find_start: &mut Option<Instant>,
    settings_panel: &mut Option<SettingsPanel>,
    pending_cursor_rebuild: &mut Option<Instant>,
    custom_cursor_active: &mut bool,
    running: &mut bool,
    gpu: &GpuContext,
    instance: HINSTANCE,
    tray_hwnd: HWND,
) {
    if cmd == 0 {
        return;
    }
    match cmd {
        CMD_TOGGLE => {
            state.settings.enabled = !state.settings.enabled;
            save_state(state);
            unsafe {
                update_tray_tip(tray_hwnd, state.settings.enabled);
            }
        }
        CMD_SETTINGS => open_or_focus_settings(settings_panel, gpu, instance),
        CMD_FIND => *find_start = Some(Instant::now()),
        CMD_NEXT_LOOK => {
            let looks = builtin_looks();
            *undo_buf = Some(state.settings.clone());
            let i = *look_idx % looks.len();
            *look_idx = (*look_idx + 1) % looks.len();
            looks[i].data.apply(&mut state.settings);
            save_state(state);
        }
        CMD_UNDO => {
            if let Some(prev) = undo_buf.take() {
                state.settings = prev;
                save_state(state);
                if state.settings.is_custom_cursor() {
                    *pending_cursor_rebuild = Some(Instant::now());
                } else if *custom_cursor_active {
                    restore_system_cursors();
                    *custom_cursor_active = false;
                }
            }
        }
        CMD_RESET_CURSOR => {
            state.settings.pointer_theme.clear();
            save_state(state);
            restore_system_cursors();
            *custom_cursor_active = false;
        }
        CMD_QUIT => *running = false,
        c if (CMD_LOOK_BASE..CMD_LOOK_BASE + 32).contains(&c) => {
            let idx = (c - CMD_LOOK_BASE) as usize;
            let looks = builtin_looks();
            if let Some(lk) = looks.get(idx) {
                *undo_buf = Some(state.settings.clone());
                lk.data.apply(&mut state.settings);
                save_state(state);
            }
        }
        c if (CMD_CURSOR_BASE..CMD_CURSOR_BASE + 32).contains(&c) => {
            let idx = (c - CMD_CURSOR_BASE) as usize;
            let cursors = builtin_cursors();
            if let Some(ck) = cursors.get(idx) {
                *undo_buf = Some(state.settings.clone());
                (ck.apply)(&mut state.settings);
                save_state(state);
                *pending_cursor_rebuild = Some(Instant::now());
            }
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_ipc_cmd(
    raw: &str,
    state: &mut AppStateFile,
    undo_buf: &mut Option<Settings>,
    look_idx: &mut usize,
    find_start: &mut Option<Instant>,
    settings_panel: &mut Option<SettingsPanel>,
    pending_cursor_rebuild: &mut Option<Instant>,
    custom_cursor_active: &mut bool,
    running: &mut bool,
    gpu: &GpuContext,
    instance: HINSTANCE,
    tray_hwnd: HWND,
) -> String {
    let parts: Vec<&str> = raw.split('\t').collect();
    let cmd = parts.first().copied().unwrap_or("");
    match cmd {
        "toggle" => {
            state.settings.enabled = !state.settings.enabled;
            save_state(state);
            unsafe {
                update_tray_tip(tray_hwnd, state.settings.enabled);
            }
            if state.settings.enabled {
                "on".into()
            } else {
                "off".into()
            }
        }
        "on" => {
            state.settings.enabled = true;
            save_state(state);
            unsafe {
                update_tray_tip(tray_hwnd, true);
            }
            "on".into()
        }
        "off" => {
            state.settings.enabled = false;
            save_state(state);
            unsafe {
                update_tray_tip(tray_hwnd, false);
            }
            "off".into()
        }
        "find" => {
            *find_start = Some(Instant::now());
            "ok".into()
        }
        "settings" => {
            open_or_focus_settings(settings_panel, gpu, instance);
            "ok".into()
        }
        "look" => {
            let key = parts.get(1).copied().unwrap_or("");
            let looks = builtin_looks();
            if let Some(lk) = looks
                .iter()
                .find(|l| l.key.eq_ignore_ascii_case(key) || l.label.eq_ignore_ascii_case(key))
            {
                *undo_buf = Some(state.settings.clone());
                lk.data.apply(&mut state.settings);
                save_state(state);
                format!("look: {}", lk.label)
            } else {
                format!("unknown look: {}", key)
            }
        }
        "nextLook" => {
            let looks = builtin_looks();
            *undo_buf = Some(state.settings.clone());
            let i = *look_idx % looks.len();
            *look_idx = (*look_idx + 1) % looks.len();
            looks[i].data.apply(&mut state.settings);
            save_state(state);
            format!("look: {}", looks[i].label)
        }
        "undo" => {
            if let Some(prev) = undo_buf.take() {
                state.settings = prev;
                save_state(state);
                if state.settings.is_custom_cursor() {
                    *pending_cursor_rebuild = Some(Instant::now());
                } else if *custom_cursor_active {
                    restore_system_cursors();
                    *custom_cursor_active = false;
                }
                "undone".into()
            } else {
                "nothing to undo".into()
            }
        }
        "cursor" => {
            let key = parts.get(1).copied().unwrap_or("");
            if key.is_empty() || key == "default" || key == "reset" {
                state.settings.pointer_theme.clear();
                save_state(state);
                restore_system_cursors();
                *custom_cursor_active = false;
                return "restored system cursor".into();
            }
            let cursors = builtin_cursors();
            if let Some(ck) = cursors
                .iter()
                .find(|c| c.key.eq_ignore_ascii_case(key) || c.label.eq_ignore_ascii_case(key))
            {
                *undo_buf = Some(state.settings.clone());
                (ck.apply)(&mut state.settings);
                save_state(state);
                *pending_cursor_rebuild = Some(Instant::now());
                format!("cursor: {}", ck.label)
            } else {
                format!("unknown cursor: {}", key)
            }
        }
        "set" => {
            let k = parts.get(1).copied().unwrap_or("");
            let v = parts.get(2).copied().unwrap_or("");
            match state.settings.set_by_key(k, v) {
                Ok(()) => {
                    save_state(state);
                    if k == "startWithWindows" {
                        set_start_with_windows(state.settings.start_with_windows);
                    }
                    if k.starts_with("cursor") || k.starts_with("pointer") {
                        if state.settings.is_custom_cursor() {
                            *pending_cursor_rebuild = Some(Instant::now());
                        } else if *custom_cursor_active {
                            restore_system_cursors();
                            *custom_cursor_active = false;
                        }
                    }
                    "ok".into()
                }
                Err(e) => e,
            }
        }
        "save" => {
            state.ensure_slots();
            if let Some(idx) = parts.get(1).and_then(|s| s.parse::<usize>().ok())
                && (1..=SLOTS).contains(&idx)
            {
                state.slots[idx - 1] = Some(state.settings.clone());
                save_state(state);
                return format!("saved slot {}", idx);
            }
            "usage: save <1..12>".into()
        }
        "saveNew" => {
            state.ensure_slots();
            if let Some(pos) = state.slots.iter().position(|s| s.is_none()) {
                state.slots[pos] = Some(state.settings.clone());
                save_state(state);
                format!("saved slot {}", pos + 1)
            } else {
                "all 12 slots full".into()
            }
        }
        "load" => {
            state.ensure_slots();
            if let Some(idx) = parts.get(1).and_then(|s| s.parse::<usize>().ok())
                && (1..=SLOTS).contains(&idx)
            {
                if let Some(saved) = state.slots[idx - 1].clone() {
                    *undo_buf = Some(state.settings.clone());
                    state.settings = saved;
                    save_state(state);
                    if state.settings.is_custom_cursor() {
                        *pending_cursor_rebuild = Some(Instant::now());
                    }
                    return format!("loaded slot {}", idx);
                }
                return format!("slot {} is empty", idx);
            }
            "usage: load <1..12>".into()
        }
        "exportLook" => state.settings.export_look_json(),
        "importLook" => {
            let json = parts.get(1..).map(|s| s.join(" ")).unwrap_or_default();
            let prev = state.settings.clone();
            if state.settings.import_look_json(&json) {
                *undo_buf = Some(prev);
                save_state(state);
                "imported".into()
            } else {
                "invalid look JSON".into()
            }
        }
        "status" => serde_json::to_string_pretty(&state.settings).unwrap_or_default(),
        "quit" | "stop" => {
            *running = false;
            "quitting".into()
        }
        _ => format!("unknown command: {}", cmd),
    }
}
