//! Sleek 8-Tab Direct2D + DirectWrite Settings Panel for Winscroller Mouse.
//!
//! Features:
//! - Live animated Trail & Halo preview header (interactive on mouse hover + auto figure-8 demo)
//! - Live Custom Cursor Showcase on light & dark strips (Arrow, Hand, Beam, Busy, Help, Move)
//! - Instant cross-tab Search bar
//! - All 8 tabs from omarchy-plume: Looks, Trail, Halo, Colors, Cursor, Find, Behavior, About
//! - Panel color theming (Follow theme, Theme palette, Window borders, Custom, Monochrome), tint, opacity & outline
//! - 12 Ready-Made Looks, 16 Ready-Made Cursors, 12 User Saved Slots, Undo, Copy/Paste Look JSON
//! - Zero memory overhead when closed (window & D2D resources are created on open and dropped on close)

#![allow(clippy::too_many_arguments)]

use std::cell::RefCell;
use std::time::Instant;

use windows::{
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::{Common::*, *},
            DirectWrite::*,
            Dwm::*,
            Dxgi::{Common::*, *},
        },
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
    core::*,
};

use winscroller_mouse_core::{
    colors::{ColorCtx, ColorMode, Rgb, Rgba, contrast, mix_rgb, parse_hex, to_rgba},
    cursor_forge::{
        arrows::ARROW_KEYS, lines::LINE_KEYS, ornaments::ORNAMENT_KEYS, surfaces::SURFACE_KEYS,
    },
    draw::DrawCtx,
    motion::{MotionState, Point},
    paint::{FrameState, paint_frame_into},
    settings::{
        AppStateFile, HaloKind, HaloShow, MINE_THEME, PALETTE_PRESETS, SLOTS, Settings, TrailKind,
        builtin_cursors, builtin_looks,
    },
    trails::TrailMemory,
};

use crate::renderer::{GpuContext, execute_draw_cmds, pt, rgba_d2d};
use crate::win_cursor::render_preview_rgba;
use crate::win_theme::{clipboard_get_text, clipboard_set_text};

const SETTINGS_CLASS: PCWSTR = w!("WinscrollerMouse_SettingsWnd");
const TAB_NAMES: [&str; 8] = [
    "Looks", "Trail", "Halo", "Colors", "Cursor", "Find", "Behavior", "About",
];

#[derive(Clone)]
struct HitItem {
    rect: D2D_RECT_F,
    action: UiAction,
}

#[derive(Clone)]
enum UiAction {
    SelectTab(usize),
    ClearSearch,
    FocusSearch,
    TestFind,
    Undo,
    CopyLook,
    PasteLook,
    ApplyLook(usize),
    ApplyCursor(usize),
    SlotClick(usize),
    ResetAll,
    RestoreSystemCursor,
    SetBool(&'static str, bool),
    SetStr(&'static str, String),
    SetInt(&'static str, i32),
    Slider {
        key: &'static str,
        min: i32,
        max: i32,
        step: i32,
        x0: f32,
        x1: f32,
    },
    PasteHex(&'static str),
}

struct PanelTheme {
    bg: Rgb,
    card: Rgb,
    card_hover: Rgb,
    fg: Rgb,
    fg_dim: Rgb,
    accent: Rgb,
    border: Rgb,
    outline: Option<[Rgb; 2]>,
}

/// Actions sent back from Settings UI to the main loop.
#[derive(Default)]
pub struct UiEvents {
    pub settings_changed: bool,
    pub trigger_find: bool,
    pub restore_cursor: bool,
    pub start_with_windows_changed: Option<bool>,
}

pub struct SettingsPanel {
    pub hwnd: HWND,
    width: u32,
    height: u32,
    swap_chain: IDXGISwapChain1,
    d2d_ctx: ID2D1DeviceContext,
    _target_bitmap: Option<ID2D1Bitmap1>,
    solid_brush: ID2D1SolidColorBrush,
    _dw_factory: IDWriteFactory,
    _font_title: IDWriteTextFormat,
    font_head: IDWriteTextFormat,
    font_body: IDWriteTextFormat,
    font_small: IDWriteTextFormat,

    // UI state
    active_tab: usize,
    scroll_y: f32,
    max_scroll: f32,
    search: String,
    search_focused: bool,
    hover_x: f32,
    hover_y: f32,
    dragging_slider: Option<(&'static str, i32, i32, i32, f32, f32)>,
    hits: Vec<HitItem>,
    status_toast: Option<(String, Instant)>,

    // Live preview state
    demo_motion: MotionState,
    demo_pts: Vec<Point>,
    demo_trail_mem: TrailMemory,
    demo_draw: DrawCtx,
    demo_find_start: Option<Instant>,
    preview_hover: bool,
    last_frame: Instant,
    start_time: Instant,

    // Cached cursor preview bitmaps (Arrow, Hand, Beam, Busy, Help, Move)
    cursor_preview_hash: u64,
    cursor_bitmaps: Vec<Option<ID2D1Bitmap1>>,
    last_opacity: u32,
}

// Per-window event queue populated by Win32 WndProc
#[derive(Clone, Debug)]
enum WndMsg {
    Resize(u32, u32),
    MouseMove(f32, f32),
    LButtonDown(f32, f32),
    LButtonUp,
    RButtonDown(f32, f32),
    MButtonDown(f32, f32),
    Wheel(f32),
    Char(char),
    KeyDown(u32),
    Closed,
}

thread_local! {
    static PENDING_MSGS: RefCell<Vec<WndMsg>> = const { RefCell::new(Vec::new()) };
}

unsafe extern "system" fn settings_wndproc(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    match msg {
        WM_SIZE => {
            let w = (lp.0 & 0xFFFF) as u32;
            let h = ((lp.0 >> 16) & 0xFFFF) as u32;
            if w > 0 && h > 0 {
                PENDING_MSGS.with(|q| q.borrow_mut().push(WndMsg::Resize(w, h)));
            }
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let x = (lp.0 as i16) as f32;
            let y = ((lp.0 >> 16) as i16) as f32;
            PENDING_MSGS.with(|q| q.borrow_mut().push(WndMsg::MouseMove(x, y)));
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let x = (lp.0 as i16) as f32;
            let y = ((lp.0 >> 16) as i16) as f32;
            let _ = SetCapture(hwnd);
            PENDING_MSGS.with(|q| q.borrow_mut().push(WndMsg::LButtonDown(x, y)));
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let _ = ReleaseCapture();
            PENDING_MSGS.with(|q| q.borrow_mut().push(WndMsg::LButtonUp));
            LRESULT(0)
        }
        WM_RBUTTONDOWN => {
            let x = (lp.0 as i16) as f32;
            let y = ((lp.0 >> 16) as i16) as f32;
            PENDING_MSGS.with(|q| q.borrow_mut().push(WndMsg::RButtonDown(x, y)));
            LRESULT(0)
        }
        WM_MBUTTONDOWN => {
            let x = (lp.0 as i16) as f32;
            let y = ((lp.0 >> 16) as i16) as f32;
            PENDING_MSGS.with(|q| q.borrow_mut().push(WndMsg::MButtonDown(x, y)));
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            let delta = ((wp.0 >> 16) as i16) as f32 / 120.0;
            PENDING_MSGS.with(|q| q.borrow_mut().push(WndMsg::Wheel(delta)));
            LRESULT(0)
        }
        WM_CHAR => {
            if let Some(ch) = char::from_u32(wp.0 as u32) {
                PENDING_MSGS.with(|q| q.borrow_mut().push(WndMsg::Char(ch)));
            }
            LRESULT(0)
        }
        WM_KEYDOWN => {
            PENDING_MSGS.with(|q| q.borrow_mut().push(WndMsg::KeyDown(wp.0 as u32)));
            LRESULT(0)
        }
        WM_CLOSE => {
            PENDING_MSGS.with(|q| q.borrow_mut().push(WndMsg::Closed));
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

impl SettingsPanel {
    pub unsafe fn open(gpu: &GpuContext, instance: HINSTANCE) -> Result<Self> {
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpszClassName: SETTINGS_CLASS,
            hInstance: instance,
            lpfnWndProc: Some(settings_wndproc),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            style: CS_HREDRAW | CS_VREDRAW,
            ..Default::default()
        };
        RegisterClassExW(&wc);

        let w = 620u32;
        let h = 760u32;
        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);
        let x = ((sw - w as i32) / 2).max(40);
        let y = ((sh - h as i32) / 2).max(30);

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED,
            SETTINGS_CLASS,
            w!("Winscroller Mouse — Settings"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            x,
            y,
            w as i32,
            h as i32,
            None,
            None,
            Some(instance),
            None,
        )?;

        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 248, LWA_ALPHA);

        // Dark titlebar on Windows 10/11
        let dark_val: BOOL = TRUE;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_val as *const _ as *const _,
            std::mem::size_of::<BOOL>() as u32,
        );

        let mut cr = RECT::default();
        let _ = GetClientRect(hwnd, &mut cr);
        let cw = (cr.right - cr.left).max(1) as u32;
        let ch = (cr.bottom - cr.top).max(1) as u32;

        let sc_desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: cw,
            Height: ch,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            Stereo: FALSE,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            AlphaMode: DXGI_ALPHA_MODE_IGNORE,
            Flags: 0,
        };
        let swap_chain = gpu.dxgi_factory.CreateSwapChainForHwnd(
            &gpu.dxgi_device,
            hwnd,
            &sc_desc,
            None,
            None,
        )?;

        let d2d_ctx = gpu
            .d2d_device
            .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
        d2d_ctx.SetDpi(96.0, 96.0);
        d2d_ctx.SetUnitMode(D2D1_UNIT_MODE_PIXELS);

        let target_bitmap = Self::bind_backbuffer(&d2d_ctx, &swap_chain)?;
        let solid_brush = d2d_ctx.CreateSolidColorBrush(
            &D2D1_COLOR_F {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            None,
        )?;

        let dw_factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
        let mk_font = |size: f32, weight: DWRITE_FONT_WEIGHT| -> Result<IDWriteTextFormat> {
            let fmt = dw_factory.CreateTextFormat(
                w!("Segoe UI"),
                None,
                weight,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size,
                w!("en-us"),
            )?;
            let _ = fmt.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
            Ok(fmt)
        };

        let font_title = mk_font(16.0, DWRITE_FONT_WEIGHT_SEMI_BOLD)?;
        let font_head = mk_font(13.5, DWRITE_FONT_WEIGHT_SEMI_BOLD)?;
        let font_body = mk_font(12.5, DWRITE_FONT_WEIGHT_NORMAL)?;
        let font_small = mk_font(11.0, DWRITE_FONT_WEIGHT_NORMAL)?;

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);

        Ok(Self {
            hwnd,
            width: cw,
            height: ch,
            swap_chain,
            d2d_ctx,
            _target_bitmap: Some(target_bitmap),
            solid_brush,
            _dw_factory: dw_factory,
            _font_title: font_title,
            font_head,
            font_body,
            font_small,
            active_tab: 0,
            scroll_y: 0.0,
            max_scroll: 0.0,
            search: String::new(),
            search_focused: false,
            hover_x: -1.0,
            hover_y: -1.0,
            dragging_slider: None,
            hits: Vec::with_capacity(256),
            status_toast: None,
            demo_motion: MotionState::new(),
            demo_pts: Vec::with_capacity(160),
            demo_trail_mem: TrailMemory::new(),
            demo_draw: DrawCtx::new(620.0, 120.0),
            demo_find_start: None,
            preview_hover: false,
            last_frame: Instant::now(),
            start_time: Instant::now(),
            cursor_preview_hash: 0,
            cursor_bitmaps: Vec::new(),
            last_opacity: 97,
        })
    }

    unsafe fn bind_backbuffer(
        d2d_ctx: &ID2D1DeviceContext,
        swap_chain: &IDXGISwapChain1,
    ) -> Result<ID2D1Bitmap1> {
        let surface: IDXGISurface = swap_chain.GetBuffer(0)?;
        let props = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_IGNORE,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
            colorContext: std::mem::ManuallyDrop::new(None),
        };
        let bmp = d2d_ctx.CreateBitmapFromDxgiSurface(&surface, Some(&props))?;
        d2d_ctx.SetTarget(&bmp);
        Ok(bmp)
    }

    unsafe fn resize(&mut self, w: u32, h: u32) {
        self.width = w.max(1);
        self.height = h.max(1);
        self.d2d_ctx.SetTarget(None);
        self._target_bitmap = None;
        if self
            .swap_chain
            .ResizeBuffers(
                2,
                self.width,
                self.height,
                DXGI_FORMAT_B8G8R8A8_UNORM,
                DXGI_SWAP_CHAIN_FLAG(0),
            )
            .is_ok()
            && let Ok(bmp) = Self::bind_backbuffer(&self.d2d_ctx, &self.swap_chain)
        {
            self._target_bitmap = Some(bmp);
        }
    }

    fn resolve_theme(&self, cfg: &Settings, win_theme: &ColorCtx) -> PanelTheme {
        let mut bg = win_theme.background;
        let mut fg = win_theme.foreground;
        let mut accent = win_theme.accent;

        match cfg.panel_color.as_str() {
            "palette" => {
                if !win_theme.palette.is_empty() {
                    accent = win_theme.palette[self.active_tab % win_theme.palette.len()];
                }
            }
            "borders" => {
                if let Some(&b0) = win_theme.border.first() {
                    accent = b0;
                }
            }
            "custom" => {
                bg = parse_hex(&cfg.panel_bg).unwrap_or([20.0, 20.0, 20.0]);
                fg = parse_hex(&cfg.panel_fg).unwrap_or([224.0, 224.0, 224.0]);
                accent = parse_hex(&cfg.panel_custom_color).unwrap_or([255.0, 106.0, 31.0]);
            }
            "monochrome" => {
                let g = 0.299 * accent[0] + 0.587 * accent[1] + 0.114 * accent[2];
                accent = [g.max(170.0), g.max(170.0), g.max(170.0)];
            }
            _ => {}
        }

        if cfg.panel_tint > 0 {
            bg = mix_rgb(bg, accent, (cfg.panel_tint as f32) / 100.0);
        }

        match cfg.panel_text.as_str() {
            "custom" => {
                fg = parse_hex(&cfg.panel_fg).unwrap_or(fg);
            }
            "auto" => {
                let bg_lum = winscroller_mouse_core::colors::lum(bg);
                let fg_lum = winscroller_mouse_core::colors::lum(fg);
                if (bg_lum - fg_lum).abs() < 0.35 {
                    fg = contrast(bg);
                }
            }
            _ => {}
        }

        let is_dark = winscroller_mouse_core::colors::lum(bg) < 0.5;
        let card = if is_dark {
            mix_rgb(bg, [255.0, 255.0, 255.0], 0.06)
        } else {
            mix_rgb(bg, [0.0, 0.0, 0.0], 0.05)
        };
        let card_hover = if is_dark {
            mix_rgb(bg, [255.0, 255.0, 255.0], 0.11)
        } else {
            mix_rgb(bg, [0.0, 0.0, 0.0], 0.10)
        };
        let border = if is_dark {
            mix_rgb(bg, [255.0, 255.0, 255.0], 0.14)
        } else {
            mix_rgb(bg, [0.0, 0.0, 0.0], 0.15)
        };
        let fg_dim = mix_rgb(fg, bg, 0.42);

        let outline = if cfg.panel_frame {
            let c1 = match cfg.panel_frame_color.as_str() {
                "first" => win_theme.border.first().copied().unwrap_or(accent),
                "last" => win_theme.border.last().copied().unwrap_or(accent),
                "middle" => {
                    let b0 = win_theme.border.first().copied().unwrap_or(accent);
                    let b1 = win_theme.border.last().copied().unwrap_or(accent);
                    mix_rgb(b0, b1, 0.5)
                }
                _ => accent,
            };
            let c2 = if cfg.panel_frame_gradient {
                win_theme.border.last().copied().unwrap_or(c1)
            } else {
                c1
            };
            Some([c1, c2])
        } else {
            None
        };

        PanelTheme {
            bg,
            card,
            card_hover,
            fg,
            fg_dim,
            accent,
            border,
            outline,
        }
    }

    fn toast(&mut self, msg: impl Into<String>) {
        self.status_toast = Some((msg.into(), Instant::now()));
    }

    /// Process input events and render one frame. Returns `false` if the user closed the panel.
    pub unsafe fn tick_and_render(
        &mut self,
        gpu: &GpuContext,
        state: &mut AppStateFile,
        undo_buf: &mut Option<Settings>,
        win_theme: &ColorCtx,
        events: &mut UiEvents,
    ) -> bool {
        let msgs: Vec<WndMsg> = PENDING_MSGS.with(|q| q.borrow_mut().drain(..).collect());
        for m in msgs {
            match m {
                WndMsg::Closed => return false,
                WndMsg::Resize(w, h) => self.resize(w, h),
                WndMsg::MouseMove(x, y) => {
                    self.hover_x = x;
                    self.hover_y = y;
                    self.preview_hover =
                        (10.0..=126.0).contains(&y) && x >= 12.0 && x <= self.width as f32 - 12.0;
                    if let Some((key, min, max, step, x0, x1)) = self.dragging_slider {
                        let t = ((x - x0) / (x1 - x0).max(1.0)).clamp(0.0, 1.0);
                        let raw = min as f32 + t * (max - min) as f32;
                        let snapped = ((raw / step as f32).round() as i32 * step).clamp(min, max);
                        if state.settings.set_by_key(key, &snapped.to_string()).is_ok() {
                            events.settings_changed = true;
                        }
                    }
                }
                WndMsg::LButtonUp => {
                    self.dragging_slider = None;
                }
                WndMsg::Wheel(delta) => {
                    self.scroll_y = (self.scroll_y - delta * 46.0).clamp(0.0, self.max_scroll);
                }
                WndMsg::KeyDown(vk) => {
                    if vk == 0x08 && self.search_focused {
                        self.search.pop();
                        self.scroll_y = 0.0;
                    } else if vk == 0x1B {
                        if !self.search.is_empty() {
                            self.search.clear();
                            self.scroll_y = 0.0;
                        } else {
                            self.search_focused = false;
                        }
                    }
                }
                WndMsg::Char(ch) => {
                    if self.search_focused && !ch.is_control() && self.search.len() < 48 {
                        self.search.push(ch);
                        self.scroll_y = 0.0;
                    }
                }
                WndMsg::LButtonDown(x, y) => {
                    self.hover_x = x;
                    self.hover_y = y;
                    self.handle_click(x, y, 0, state, undo_buf, events);
                }
                WndMsg::RButtonDown(x, y) => {
                    self.handle_click(x, y, 1, state, undo_buf, events);
                }
                WndMsg::MButtonDown(x, y) => {
                    self.handle_click(x, y, 2, state, undo_buf, events);
                }
            }
        }

        // Update panel opacity if changed
        let op = state.settings.panel_opacity.clamp(50, 100);
        if op != self.last_opacity {
            self.last_opacity = op;
            let a = ((op as f32 / 100.0) * 255.0).round() as u8;
            let _ = SetLayeredWindowAttributes(self.hwnd, COLORREF(0), a, LWA_ALPHA);
        }

        // Refresh cached cursor preview bitmaps when cursor design changes
        let chash = state.settings.cursor_fingerprint(
            &winscroller_mouse_core::colors::to_hex(win_theme.accent),
            &winscroller_mouse_core::colors::to_hex(win_theme.foreground),
            &winscroller_mouse_core::colors::to_hex(win_theme.background),
            &winscroller_mouse_core::colors::to_hex(
                win_theme
                    .border
                    .first()
                    .copied()
                    .unwrap_or(win_theme.accent),
            ),
        );
        if chash != self.cursor_preview_hash {
            self.cursor_preview_hash = chash;
            self.rebuild_cursor_bitmaps(&state.settings, win_theme);
        }

        self.render(gpu, state, undo_buf.is_some(), win_theme);
        true
    }

    fn handle_click(
        &mut self,
        x: f32,
        y: f32,
        button: u8, // 0=left, 1=right, 2=middle
        state: &mut AppStateFile,
        undo_buf: &mut Option<Settings>,
        events: &mut UiEvents,
    ) {
        // Check hit list from top-most (last pushed) to bottom
        let mut clicked: Option<UiAction> = None;
        for h in self.hits.iter().rev() {
            if x >= h.rect.left && x <= h.rect.right && y >= h.rect.top && y <= h.rect.bottom {
                clicked = Some(h.action.clone());
                break;
            }
        }

        if !matches!(clicked, Some(UiAction::FocusSearch)) {
            self.search_focused = false;
        }

        let Some(action) = clicked else { return };

        match action {
            UiAction::SelectTab(i) if button == 0 => {
                self.active_tab = i;
                self.scroll_y = 0.0;
            }
            UiAction::FocusSearch if button == 0 => {
                self.search_focused = true;
            }
            UiAction::ClearSearch if button == 0 => {
                self.search.clear();
                self.scroll_y = 0.0;
            }
            UiAction::TestFind if button == 0 => {
                self.demo_find_start = Some(Instant::now());
                events.trigger_find = true;
            }
            UiAction::Undo if button == 0 => {
                if let Some(prev) = undo_buf.take() {
                    state.settings = prev;
                    events.settings_changed = true;
                    self.toast("Undid last change");
                }
            }
            UiAction::CopyLook if button == 0 => {
                let json = state.settings.export_look_json();
                if clipboard_set_text(&json) {
                    self.toast("Copied look to clipboard");
                }
            }
            UiAction::PasteLook if button == 0 => {
                if let Some(txt) = clipboard_get_text() {
                    let prev = state.settings.clone();
                    if state.settings.import_look_json(&txt) {
                        *undo_buf = Some(prev);
                        events.settings_changed = true;
                        self.toast("Imported look from clipboard");
                    } else {
                        self.toast("Clipboard does not contain a valid look");
                    }
                }
            }
            UiAction::ApplyLook(idx) if button == 0 => {
                let looks = builtin_looks();
                if let Some(lk) = looks.get(idx) {
                    *undo_buf = Some(state.settings.clone());
                    lk.data.apply(&mut state.settings);
                    events.settings_changed = true;
                    self.toast(format!("Applied look: {}", lk.label));
                }
            }
            UiAction::ApplyCursor(idx) if button == 0 => {
                let cursors = builtin_cursors();
                if let Some(ck) = cursors.get(idx) {
                    *undo_buf = Some(state.settings.clone());
                    (ck.apply)(&mut state.settings);
                    events.settings_changed = true;
                    self.toast(format!("Applied cursor: {}", ck.label));
                }
            }
            UiAction::SlotClick(idx) => {
                state.ensure_slots();
                if idx < SLOTS {
                    if button == 2 {
                        // Middle click clears slot
                        state.slots[idx] = None;
                        events.settings_changed = true;
                        self.toast(format!("Cleared slot {}", idx + 1));
                    } else if button == 1 || state.slots[idx].is_none() {
                        // Right-click overwrites; left-click on empty saves
                        state.slots[idx] = Some(state.settings.clone());
                        events.settings_changed = true;
                        self.toast(format!("Saved all settings to slot {}", idx + 1));
                    } else if let Some(saved) = state.slots[idx].clone() {
                        // Left-click on occupied slot loads it
                        *undo_buf = Some(state.settings.clone());
                        let keep_start = state.settings.start_look.clone();
                        state.settings = saved;
                        state.settings.start_look = keep_start;
                        events.settings_changed = true;
                        self.toast(format!("Loaded slot {}", idx + 1));
                    }
                }
            }
            UiAction::RestoreSystemCursor if button == 0 => {
                state.settings.pointer_theme.clear();
                events.settings_changed = true;
                events.restore_cursor = true;
                self.toast("Restored Windows system cursor");
            }
            UiAction::ResetAll if button == 0 => {
                *undo_buf = Some(state.settings.clone());
                state.settings = Settings::default();
                events.settings_changed = true;
                events.restore_cursor = true;
                self.toast("All settings reset to defaults");
            }
            UiAction::SetBool(key, val) if button == 0 => {
                if state
                    .settings
                    .set_by_key(key, if val { "true" } else { "false" })
                    .is_ok()
                {
                    events.settings_changed = true;
                    if key == "startWithWindows" {
                        events.start_with_windows_changed = Some(val);
                    }
                }
            }
            UiAction::SetStr(key, val) if button == 0 => {
                if state.settings.set_by_key(key, &val).is_ok() {
                    events.settings_changed = true;
                    if key == "pointerTheme" && val.is_empty() {
                        events.restore_cursor = true;
                    }
                }
            }
            UiAction::SetInt(key, val) if button == 0 => {
                if state.settings.set_by_key(key, &val.to_string()).is_ok() {
                    events.settings_changed = true;
                }
            }
            UiAction::Slider {
                key,
                min,
                max,
                step,
                x0,
                x1,
            } if button == 0 => {
                let t = ((x - x0) / (x1 - x0).max(1.0)).clamp(0.0, 1.0);
                let raw = min as f32 + t * (max - min) as f32;
                let snapped = ((raw / step as f32).round() as i32 * step).clamp(min, max);
                if state.settings.set_by_key(key, &snapped.to_string()).is_ok() {
                    events.settings_changed = true;
                }
                self.dragging_slider = Some((key, min, max, step, x0, x1));
            }
            UiAction::PasteHex(key) if button == 0 => {
                if let Some(txt) = clipboard_get_text() {
                    let s = txt.trim();
                    if parse_hex(s).is_some() {
                        let hex = if s.starts_with('#') {
                            s.to_string()
                        } else {
                            format!("#{}", s)
                        };
                        if state.settings.set_by_key(key, &hex).is_ok() {
                            events.settings_changed = true;
                            self.toast(format!("Set color to {}", hex));
                        }
                    } else {
                        self.toast("Clipboard is not a #RRGGBB hex color");
                    }
                }
            }
            _ => {}
        }
    }

    unsafe fn rebuild_cursor_bitmaps(&mut self, cfg: &Settings, win_theme: &ColorCtx) {
        self.cursor_bitmaps.clear();
        let kinds = ["arrow", "hand", "beam", "wait", "help", "move"];
        let sz = 36u32;
        for kind in kinds {
            let bmp = render_preview_rgba(cfg, win_theme, kind, sz).and_then(|rgba| {
                // Convert straight RGBA to premultiplied BGRA for Direct2D bitmap
                let mut bgra = vec![0u8; rgba.len()];
                for (src, dst) in rgba
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .zip(bgra.as_chunks_mut::<4>().0.iter_mut())
                {
                    let a = src[3] as u32;
                    dst[0] = ((src[2] as u32 * a + 127) / 255) as u8;
                    dst[1] = ((src[1] as u32 * a + 127) / 255) as u8;
                    dst[2] = ((src[0] as u32 * a + 127) / 255) as u8;
                    dst[3] = src[3];
                }
                let props = D2D1_BITMAP_PROPERTIES1 {
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    dpiX: 96.0,
                    dpiY: 96.0,
                    bitmapOptions: D2D1_BITMAP_OPTIONS_NONE,
                    colorContext: std::mem::ManuallyDrop::new(None),
                };
                self.d2d_ctx
                    .CreateBitmap(
                        D2D_SIZE_U {
                            width: sz,
                            height: sz,
                        },
                        Some(bgra.as_ptr() as *const _),
                        sz * 4,
                        &props,
                    )
                    .ok()
            });
            self.cursor_bitmaps.push(bmp);
        }
    }

    unsafe fn render(
        &mut self,
        gpu: &GpuContext,
        state: &AppStateFile,
        has_undo: bool,
        win_theme: &ColorCtx,
    ) {
        let w = self.width as f32;
        let h = self.height as f32;
        let cfg = &state.settings;
        let def = Settings::default();
        let theme = self.resolve_theme(cfg, win_theme);

        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32().min(0.05);
        self.last_frame = now;
        let elapsed = now.duration_since(self.start_time).as_secs_f64();
        let now_ms = elapsed * 1000.0;

        self.hits.clear();

        let ctx = self.d2d_ctx.clone();
        ctx.BeginDraw();
        ctx.SetAntialiasMode(D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
        ctx.Clear(Some(&rgba_d2d(to_rgba(theme.bg, 1.0))));

        // ---- 1. Top Live Preview Box (y = 12..126) --------------------------
        let prev_rect = D2D_RECT_F {
            left: 14.0,
            top: 12.0,
            right: w - 14.0,
            bottom: 126.0,
        };
        self.fill_rrect(
            prev_rect,
            10.0,
            to_rgba(mix_rgb(theme.bg, [0.0, 0.0, 0.0], 0.35), 1.0),
        );
        self.stroke_rrect(prev_rect, 10.0, to_rgba(theme.border, 1.0), 1.0);

        let pw = prev_rect.right - prev_rect.left;
        let ph = prev_rect.bottom - prev_rect.top;
        // Lissajous figure-8 demo path (or live mouse inside preview!)
        let (dx, dy) = if self.preview_hover {
            (self.hover_x - prev_rect.left, self.hover_y - prev_rect.top)
        } else {
            let t = elapsed as f32 * 1.85;
            (
                pw * 0.5 + (t.sin()) * (pw * 0.34),
                ph * 0.5 + ((t * 2.0).sin()) * (ph * 0.30),
            )
        };
        self.demo_motion.add(dx, dy, now_ms);
        self.demo_motion.prune(now_ms, cfg.trail_life_secs());
        self.demo_motion.points_for_into(
            now_ms,
            cfg.trail_life_secs(),
            0.0,
            0.0,
            &mut self.demo_pts,
        );

        let mut demo_cctx = win_theme.clone();
        demo_cctx.speed = self.demo_motion.speed01();
        demo_cctx.time = elapsed;
        if let Some(c) = parse_hex(&cfg.custom_color) {
            demo_cctx.custom = c;
        }
        if let Some(c) = parse_hex(&cfg.custom_color2) {
            demo_cctx.custom2 = c;
        }

        let find_age = self
            .demo_find_start
            .map(|s| s.elapsed().as_secs_f32())
            .unwrap_or(-1.0);
        if find_age > 1.5 {
            self.demo_find_start = None;
        }

        let mut fs = FrameState {
            settings: cfg,
            pts: &self.demo_pts,
            cursor_x: dx,
            cursor_y: dy,
            screen_x: 0.0,
            screen_y: 0.0,
            screen_w: pw,
            screen_h: ph,
            cursor_on_this_screen: true,
            color_ctx: &demo_cctx,
            time_secs: elapsed,
            dt,
            moved: self.demo_motion.moved,
            still_secs: self.demo_motion.still_for(now_ms),
            find_age,
            trail_mem: &mut self.demo_trail_mem,
        };
        let _ = paint_frame_into(&mut self.demo_draw, &mut fs);

        ctx.PushAxisAlignedClip(&prev_rect, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
        let xform = windows_numerics::Matrix3x2::translation(prev_rect.left, prev_rect.top);
        ctx.SetTransform(&xform);
        let _ = execute_draw_cmds(
            &gpu.d2d_factory,
            &ctx,
            &self.solid_brush,
            &gpu.round_stroke,
            &self.demo_draw.cmds,
        );
        // Draw small cursor preview at demo pointer position
        if let Some(Some(bmp)) = self.cursor_bitmaps.first() {
            let dst = D2D_RECT_F {
                left: dx - 6.0,
                top: dy - 4.0,
                right: dx + 22.0,
                bottom: dy + 24.0,
            };
            ctx.DrawBitmap(
                bmp,
                Some(&dst),
                1.0,
                D2D1_INTERPOLATION_MODE_LINEAR,
                None,
                None,
            );
        }
        ctx.SetTransform(&windows_numerics::Matrix3x2::identity());
        ctx.PopAxisAlignedClip();

        // Preview label + Find test button
        self.draw_text(
            "WINSCROLLER MOUSE  ·  Live Preview (hover to test)",
            D2D_RECT_F {
                left: prev_rect.left + 10.0,
                top: prev_rect.top + 4.0,
                right: prev_rect.right - 90.0,
                bottom: prev_rect.top + 22.0,
            },
            &self.font_small.clone(),
            to_rgba(theme.fg_dim, 0.9),
        );
        let find_btn = D2D_RECT_F {
            left: prev_rect.right - 74.0,
            top: prev_rect.top + 6.0,
            right: prev_rect.right - 8.0,
            bottom: prev_rect.top + 28.0,
        };
        self.draw_button(find_btn, "Find", false, &theme, UiAction::TestFind);

        // ---- 2. Search Bar (y = 134..164) -----------------------------------
        let search_rect = D2D_RECT_F {
            left: 14.0,
            top: 134.0,
            right: w - 14.0,
            bottom: 164.0,
        };
        self.fill_rrect(search_rect, 7.0, to_rgba(theme.card, 1.0));
        self.stroke_rrect(
            search_rect,
            7.0,
            to_rgba(
                if self.search_focused {
                    theme.accent
                } else {
                    theme.border
                },
                1.0,
            ),
            if self.search_focused { 1.5 } else { 1.0 },
        );
        let search_label = if self.search.is_empty() {
            if self.search_focused {
                "| Type to search any setting..."
            } else {
                "Search settings..."
            }
        } else if self.search_focused {
            &format!("{}|", self.search)
        } else {
            &self.search
        };
        let search_col = if self.search.is_empty() {
            theme.fg_dim
        } else {
            theme.fg
        };
        self.draw_text(
            search_label,
            D2D_RECT_F {
                left: search_rect.left + 10.0,
                top: search_rect.top,
                right: search_rect.right - 34.0,
                bottom: search_rect.bottom,
            },
            &self.font_body.clone(),
            to_rgba(search_col, 1.0),
        );
        self.hits.push(HitItem {
            rect: search_rect,
            action: UiAction::FocusSearch,
        });
        if !self.search.is_empty() {
            let clr = D2D_RECT_F {
                left: search_rect.right - 28.0,
                top: search_rect.top + 3.0,
                right: search_rect.right - 4.0,
                bottom: search_rect.bottom - 3.0,
            };
            self.draw_button(clr, "x", false, &theme, UiAction::ClearSearch);
        }

        // ---- 3. 8 Tab Bar (y = 172..204) ------------------------------------
        let tab_top = 172.0;
        let tab_h = 30.0;
        let tab_w = (w - 28.0) / TAB_NAMES.len() as f32;
        for (i, name) in TAB_NAMES.iter().enumerate() {
            let tr = D2D_RECT_F {
                left: 14.0 + i as f32 * tab_w + 2.0,
                top: tab_top,
                right: 14.0 + (i + 1) as f32 * tab_w - 2.0,
                bottom: tab_top + tab_h,
            };
            let active = self.search.is_empty() && self.active_tab == i;
            self.draw_button(tr, name, active, &theme, UiAction::SelectTab(i));
        }

        // ---- 4. Scrollable Content Area (y = 212 .. h - 28) -----------------
        let content_rect = D2D_RECT_F {
            left: 14.0,
            top: 212.0,
            right: w - 14.0,
            bottom: h - 28.0,
        };
        ctx.PushAxisAlignedClip(&content_rect, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);

        let mut y = content_rect.top + 4.0 - self.scroll_y;
        let cw = content_rect.right - content_rect.left - 8.0;
        let x0 = content_rect.left + 2.0;
        let q = self.search.trim().to_lowercase();

        if !q.is_empty() {
            self.draw_section_header(&mut y, x0, cw, "Search Results", &theme);
            self.draw_trail_tab(&mut y, x0, cw, cfg, &def, &theme, &q);
            self.draw_halo_tab(&mut y, x0, cw, cfg, &def, &theme, &q);
            self.draw_colors_tab(&mut y, x0, cw, cfg, &def, &theme, &q);
            self.draw_cursor_tab(&mut y, x0, cw, cfg, &def, &theme, &q);
            self.draw_find_tab(&mut y, x0, cw, cfg, &def, &theme, &q);
            self.draw_behavior_tab(&mut y, x0, cw, cfg, &def, &theme, &q);
        } else {
            match self.active_tab {
                0 => self.draw_looks_tab(&mut y, x0, cw, state, has_undo, &theme),
                1 => self.draw_trail_tab(&mut y, x0, cw, cfg, &def, &theme, ""),
                2 => self.draw_halo_tab(&mut y, x0, cw, cfg, &def, &theme, ""),
                3 => self.draw_colors_tab(&mut y, x0, cw, cfg, &def, &theme, ""),
                4 => self.draw_cursor_tab(&mut y, x0, cw, cfg, &def, &theme, ""),
                5 => self.draw_find_tab(&mut y, x0, cw, cfg, &def, &theme, ""),
                6 => self.draw_behavior_tab(&mut y, x0, cw, cfg, &def, &theme, ""),
                _ => self.draw_about_tab(&mut y, x0, cw, &theme),
            }
        }

        // Update scroll bounds
        let content_h = (y + self.scroll_y) - content_rect.top + 20.0;
        let view_h = content_rect.bottom - content_rect.top;
        self.max_scroll = (content_h - view_h).max(0.0);
        self.scroll_y = self.scroll_y.clamp(0.0, self.max_scroll);

        // Remove hits outside visible clip rect so offscreen buttons aren't clickable
        self.hits.retain(|item| {
            item.rect.bottom <= content_rect.top // header/tab bar items
                || (item.rect.bottom > content_rect.top && item.rect.top < content_rect.bottom)
        });

        // Subtle scrollbar indicator
        if self.max_scroll > 1.0 {
            let thumb_h = (view_h * (view_h / content_h)).max(24.0);
            let frac = self.scroll_y / self.max_scroll;
            let ty = content_rect.top + frac * (view_h - thumb_h);
            self.fill_rrect(
                D2D_RECT_F {
                    left: w - 11.0,
                    top: ty,
                    right: w - 6.0,
                    bottom: ty + thumb_h,
                },
                2.5,
                to_rgba(theme.accent, 0.55),
            );
        }

        ctx.PopAxisAlignedClip();

        // ---- 5. Status Bar / Toast (bottom 24px) ----------------------------
        let status_txt = if let Some((ref msg, ts)) = self.status_toast {
            if ts.elapsed().as_secs_f32() < 3.0 {
                msg.as_str()
            } else {
                "All changes apply live and save automatically."
            }
        } else {
            "All changes apply live and save automatically."
        };
        self.draw_text(
            status_txt,
            D2D_RECT_F {
                left: 16.0,
                top: h - 24.0,
                right: w - 16.0,
                bottom: h - 4.0,
            },
            &self.font_small.clone(),
            to_rgba(theme.fg_dim, 0.9),
        );

        // Optional colored panel outline
        if let Some([c1, _c2]) = theme.outline {
            self.stroke_rrect(
                D2D_RECT_F {
                    left: 1.0,
                    top: 1.0,
                    right: w - 1.0,
                    bottom: h - 1.0,
                },
                4.0,
                to_rgba(c1, 0.95),
                2.0,
            );
        }

        let _ = ctx.EndDraw(None, None);
        let _ = self.swap_chain.Present(1, DXGI_PRESENT(0));
    }

    // ---- Tab Renderers ------------------------------------------------------

    unsafe fn draw_looks_tab(
        &mut self,
        y: &mut f32,
        x: f32,
        w: f32,
        state: &AppStateFile,
        has_undo: bool,
        theme: &PanelTheme,
    ) {
        // Action bar: Undo, Copy Look, Paste Look
        let bw = (w - 12.0) / 3.0;
        self.draw_button(
            D2D_RECT_F {
                left: x,
                top: *y,
                right: x + bw,
                bottom: *y + 28.0,
            },
            if has_undo { "Undo" } else { "Undo (none)" },
            false,
            theme,
            UiAction::Undo,
        );
        self.draw_button(
            D2D_RECT_F {
                left: x + bw + 6.0,
                top: *y,
                right: x + bw * 2.0 + 6.0,
                bottom: *y + 28.0,
            },
            "Copy Look JSON",
            false,
            theme,
            UiAction::CopyLook,
        );
        self.draw_button(
            D2D_RECT_F {
                left: x + bw * 2.0 + 12.0,
                top: *y,
                right: x + w,
                bottom: *y + 28.0,
            },
            "Paste Look JSON",
            false,
            theme,
            UiAction::PasteLook,
        );
        *y += 38.0;

        // 12 Ready-Made Looks
        self.draw_section_header(y, x, w, "Ready-Made Looks (12)", theme);
        let looks = builtin_looks();
        let cols = 4;
        let cw = (w - (cols as f32 - 1.0) * 6.0) / cols as f32;
        for (i, lk) in looks.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let rx = x + col as f32 * (cw + 6.0);
            let ry = *y + row as f32 * 34.0;
            let active = state.settings.trail == lk.data.trail
                && state.settings.halo == lk.data.halo
                && state.settings.color_mode == lk.data.color_mode
                && state.settings.halo_size == lk.data.halo_size;
            self.draw_button(
                D2D_RECT_F {
                    left: rx,
                    top: ry,
                    right: rx + cw,
                    bottom: ry + 28.0,
                },
                lk.label,
                active,
                theme,
                UiAction::ApplyLook(i),
            );
        }
        *y += looks.len().div_ceil(cols) as f32 * 34.0 + 10.0;

        // 16 Ready-Made Cursors
        self.draw_section_header(y, x, w, "Ready-Made Cursors (16)", theme);
        let cursors = builtin_cursors();
        for (i, ck) in cursors.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let rx = x + col as f32 * (cw + 6.0);
            let ry = *y + row as f32 * 34.0;
            self.draw_button(
                D2D_RECT_F {
                    left: rx,
                    top: ry,
                    right: rx + cw,
                    bottom: ry + 28.0,
                },
                ck.label,
                false,
                theme,
                UiAction::ApplyCursor(i),
            );
        }
        *y += cursors.len().div_ceil(cols) as f32 * 34.0 + 10.0;

        // 12 User Saved Slots
        self.draw_section_header(
            y,
            x,
            w,
            "Your Saved Places (L: Load/Save · R: Overwrite · M: Clear)",
            theme,
        );
        for i in 0..SLOTS {
            let col = i % cols;
            let row = i / cols;
            let rx = x + col as f32 * (cw + 6.0);
            let ry = *y + row as f32 * 34.0;
            let filled = state.slots.get(i).and_then(|s| s.as_ref()).is_some();
            let label = if filled {
                format!("Slot {} *", i + 1)
            } else {
                format!("Slot {}", i + 1)
            };
            self.draw_button(
                D2D_RECT_F {
                    left: rx,
                    top: ry,
                    right: rx + cw,
                    bottom: ry + 28.0,
                },
                &label,
                filled,
                theme,
                UiAction::SlotClick(i),
            );
        }
        *y += 3.0 * 34.0 + 10.0;
    }

    unsafe fn draw_trail_tab(
        &mut self,
        y: &mut f32,
        x: f32,
        w: f32,
        cfg: &Settings,
        def: &Settings,
        theme: &PanelTheme,
        q: &str,
    ) {
        if q.is_empty() || "trail style comet beads ribbon sparks echoes bubbles".contains(q) {
            self.draw_section_header(y, x, w, "Trail Style", theme);
            let opts: Vec<(&str, &str)> = TrailKind::ALL
                .iter()
                .map(|t| (t.key(), t.label()))
                .collect();
            self.draw_chip_grid(y, x, w, "trail", cfg.trail.key(), &opts, 4, theme);
            self.draw_hint_line(y, x, w, cfg.trail.about(), theme);
        }
        if q.is_empty() || "trail length duration seconds".contains(q) {
            self.draw_slider_row(
                y,
                x,
                w,
                "Length (s)",
                "trailLife",
                cfg.trail_life as i32,
                def.trail_life as i32,
                15,
                150,
                5,
                100.0,
                theme,
            );
        }
        if q.is_empty() || "trail thickness size".contains(q) {
            self.draw_slider_row(
                y,
                x,
                w,
                "Thickness",
                "trailSize",
                cfg.trail_size as i32,
                def.trail_size as i32,
                2,
                30,
                1,
                1.0,
                theme,
            );
        }
        if q.is_empty() || "trail strength opacity".contains(q) {
            self.draw_slider_row(
                y,
                x,
                w,
                "Strength (%)",
                "trailStrength",
                cfg.trail_strength as i32,
                def.trail_strength as i32,
                10,
                100,
                5,
                1.0,
                theme,
            );
        }
    }

    unsafe fn draw_halo_tab(
        &mut self,
        y: &mut f32,
        x: f32,
        w: f32,
        cfg: &Settings,
        def: &Settings,
        theme: &PanelTheme,
        q: &str,
    ) {
        if q.is_empty() || "halo style glow ring pulse spotlight crosshair target".contains(q) {
            self.draw_section_header(y, x, w, "Halo Style", theme);
            let opts: Vec<(&str, &str)> =
                HaloKind::ALL.iter().map(|h| (h.key(), h.label())).collect();
            self.draw_chip_grid(y, x, w, "halo", cfg.halo.key(), &opts, 4, theme);
            self.draw_hint_line(y, x, w, cfg.halo.about(), theme);
        }
        if q.is_empty() || "halo size radius".contains(q) {
            self.draw_slider_row(
                y,
                x,
                w,
                "Size",
                "haloSize",
                cfg.halo_size as i32,
                def.halo_size as i32,
                20,
                240,
                4,
                1.0,
                theme,
            );
        }
        if q.is_empty() || "halo strength opacity".contains(q) {
            self.draw_slider_row(
                y,
                x,
                w,
                "Strength (%)",
                "haloStrength",
                cfg.halo_strength as i32,
                def.halo_strength as i32,
                10,
                100,
                5,
                1.0,
                theme,
            );
        }
        if q.is_empty() || "halo show always moving still".contains(q) {
            self.draw_section_header(y, x, w, "Show Halo", theme);
            let opts: Vec<(&str, &str)> =
                HaloShow::ALL.iter().map(|s| (s.key(), s.label())).collect();
            self.draw_chip_grid(y, x, w, "haloShow", cfg.halo_show.key(), &opts, 3, theme);
        }
    }

    unsafe fn draw_colors_tab(
        &mut self,
        y: &mut f32,
        x: f32,
        w: f32,
        cfg: &Settings,
        def: &Settings,
        theme: &PanelTheme,
        q: &str,
    ) {
        if q.is_empty()
            || "color mode theme palette border custom gradient rainbow drift speed".contains(q)
        {
            self.draw_section_header(y, x, w, "Trail & Halo Color Mode", theme);
            let opts: Vec<(&str, &str)> = ColorMode::ALL
                .iter()
                .map(|c| (c.key(), c.label()))
                .collect();
            self.draw_chip_grid(y, x, w, "colorMode", cfg.color_mode.key(), &opts, 4, theme);
            self.draw_hint_line(y, x, w, cfg.color_mode.about(), theme);
        }
        if q.is_empty() || "custom color first".contains(q) {
            self.draw_color_row(
                y,
                x,
                w,
                "Custom Color",
                "customColor",
                &cfg.custom_color,
                PALETTE_PRESETS,
                theme,
            );
        }
        if q.is_empty() || "second color gradient".contains(q) {
            self.draw_color_row(
                y,
                x,
                w,
                "Second Color",
                "customColor2",
                &cfg.custom_color2,
                PALETTE_PRESETS,
                theme,
            );
        }
        if q.is_empty() || "panel colors theme palette borders custom monochrome".contains(q) {
            self.draw_section_header(y, x, w, "This Panel", theme);
            self.draw_chip_grid(
                y,
                x,
                w,
                "panelColor",
                &cfg.panel_color,
                &[
                    ("theme", "Follow theme"),
                    ("palette", "Theme palette"),
                    ("borders", "Window borders"),
                    ("custom", "Custom"),
                    ("monochrome", "Monochrome"),
                ],
                3,
                theme,
            );
        }
        if cfg.panel_color == "custom"
            && (q.is_empty() || "panel background text accent".contains(q))
        {
            self.draw_color_row(
                y,
                x,
                w,
                "Panel Background",
                "panelBg",
                &cfg.panel_bg,
                &[
                    "#141414", "#1a1b26", "#1e1e2e", "#282828", "#f2efe9", "#ffffff",
                ],
                theme,
            );
            self.draw_color_row(
                y,
                x,
                w,
                "Panel Text",
                "panelFg",
                &cfg.panel_fg,
                &[
                    "#e0e0e0", "#c0caf5", "#cdd6f4", "#ebdbb2", "#202020", "#000000",
                ],
                theme,
            );
            self.draw_color_row(
                y,
                x,
                w,
                "Panel Accent",
                "panelCustomColor",
                &cfg.panel_custom_color,
                PALETTE_PRESETS,
                theme,
            );
        }
        if q.is_empty() || "panel text readable".contains(q) {
            self.draw_chip_grid(
                y,
                x,
                w,
                "panelText",
                &cfg.panel_text,
                &[
                    ("auto", "Always readable"),
                    ("theme", "Theme"),
                    ("custom", "My text color"),
                ],
                3,
                theme,
            );
        }
        if q.is_empty() || "panel outline frame".contains(q) {
            self.draw_toggle_row(
                y,
                x,
                w,
                "Panel Outline",
                "panelFrame",
                cfg.panel_frame,
                theme,
            );
        }
        if q.is_empty() || "panel tint".contains(q) {
            self.draw_slider_row(
                y,
                x,
                w,
                "Panel Tint",
                "panelTint",
                cfg.panel_tint as i32,
                def.panel_tint as i32,
                0,
                30,
                1,
                1.0,
                theme,
            );
        }
        if q.is_empty() || "panel opacity".contains(q) {
            self.draw_slider_row(
                y,
                x,
                w,
                "Panel Opacity (%)",
                "panelOpacity",
                cfg.panel_opacity as i32,
                def.panel_opacity as i32,
                50,
                100,
                1,
                1.0,
                theme,
            );
        }
    }

    unsafe fn draw_cursor_tab(
        &mut self,
        y: &mut f32,
        x: f32,
        w: f32,
        cfg: &Settings,
        def: &Settings,
        theme: &PanelTheme,
        q: &str,
    ) {
        // Light & Dark live cursor showcase strip!
        if q.is_empty() {
            let strip_h = 46.0;
            let half = w * 0.5;
            self.fill_rrect(
                D2D_RECT_F {
                    left: x,
                    top: *y,
                    right: x + half,
                    bottom: *y + strip_h,
                },
                6.0,
                [0.95, 0.95, 0.96, 1.0],
            );
            self.fill_rrect(
                D2D_RECT_F {
                    left: x + half,
                    top: *y,
                    right: x + w,
                    bottom: *y + strip_h,
                },
                6.0,
                [0.10, 0.10, 0.12, 1.0],
            );
            let count = self.cursor_bitmaps.len().max(1);
            let step = (half - 16.0) / count as f32;
            for (i, obmp) in self.cursor_bitmaps.iter().enumerate() {
                if let Some(bmp) = obmp {
                    let cx1 = x + 8.0 + i as f32 * step;
                    let cx2 = x + half + 8.0 + i as f32 * step;
                    let cy = *y + 5.0;
                    self.d2d_ctx.DrawBitmap(
                        bmp,
                        Some(&D2D_RECT_F {
                            left: cx1,
                            top: cy,
                            right: cx1 + 36.0,
                            bottom: cy + 36.0,
                        }),
                        1.0,
                        D2D1_INTERPOLATION_MODE_LINEAR,
                        None,
                        None,
                    );
                    self.d2d_ctx.DrawBitmap(
                        bmp,
                        Some(&D2D_RECT_F {
                            left: cx2,
                            top: cy,
                            right: cx2 + 36.0,
                            bottom: cy + 36.0,
                        }),
                        1.0,
                        D2D1_INTERPOLATION_MODE_LINEAR,
                        None,
                        None,
                    );
                }
            }
            *y += strip_h + 10.0;
        }

        if q.is_empty() || "cursor theme custom made winscroller system".contains(q) {
            self.draw_section_header(y, x, w, "Cursor Mode", theme);
            let mode_val = if cfg.is_custom_cursor() {
                MINE_THEME
            } else {
                ""
            };
            self.draw_chip_grid(
                y,
                x,
                w,
                "pointerTheme",
                mode_val,
                &[
                    ("", "Windows Default"),
                    (MINE_THEME, "Made in Winscroller Mouse"),
                ],
                2,
                theme,
            );
            self.draw_slider_row(
                y,
                x,
                w,
                "Cursor Size (px)",
                "pointerSize",
                cfg.pointer_size as i32,
                def.pointer_size as i32,
                16,
                96,
                2,
                1.0,
                theme,
            );
        }

        if q.is_empty() || "arrow shape classic slim chunky wedge dart kite needle teardrop leaf bullet crest chevron feather dot".contains(q) {
            self.draw_section_header(y, x, w, "Arrow Shape (14)", theme);
            self.draw_chip_grid(y, x, w, "cursorShape", &cfg.cursor_shape, ARROW_KEYS, 5, theme);
            self.draw_slider_row(y, x, w, "Tilt (deg)", "cursorTilt", cfg.cursor_tilt, def.cursor_tilt, -30, 30, 2, 1.0, theme);
            self.draw_slider_row(y, x, w, "Round Corners", "cursorRound", cfg.cursor_round as i32, def.cursor_round as i32, 0, 100, 5, 1.0, theme);
        }

        if q.is_empty()
            || "surface solid faint metal carbon candy holo marble nebula stained glass".contains(q)
        {
            self.draw_section_header(y, x, w, "Surface (30)", theme);
            self.draw_chip_grid(
                y,
                x,
                w,
                "cursorSurface",
                &cfg.cursor_surface,
                SURFACE_KEYS,
                5,
                theme,
            );
        }

        if q.is_empty()
            || "fill gradient glass texture stripes dots grid depth raised sunken opacity tint"
                .contains(q)
        {
            self.draw_section_header(y, x, w, "Fill, Texture & Depth", theme);
            self.draw_chip_grid(
                y,
                x,
                w,
                "cursorFill",
                &cfg.cursor_fill,
                &[
                    ("solid", "Solid"),
                    ("gradient", "Gradient"),
                    ("glass", "Glass"),
                ],
                3,
                theme,
            );
            if cfg.cursor_fill == "gradient" {
                self.draw_chip_grid(
                    y,
                    x,
                    w,
                    "cursorGradientTo",
                    &cfg.cursor_gradient_to,
                    &[
                        ("second", "Second"),
                        ("lighter", "Lighter"),
                        ("darker", "Darker"),
                        ("accent", "Accent"),
                        ("palette", "Palette"),
                        ("border", "Border"),
                    ],
                    3,
                    theme,
                );
                self.draw_chip_grid(
                    y,
                    x,
                    w,
                    "cursorGradientDir",
                    &cfg.cursor_gradient_dir,
                    &[
                        ("along", "Tip to tail"),
                        ("across", "Across"),
                        ("diagonal", "Diagonal"),
                        ("theme", "Border angle"),
                    ],
                    4,
                    theme,
                );
            }
            self.draw_chip_grid(
                y,
                x,
                w,
                "cursorPattern",
                &cfg.cursor_pattern,
                &[
                    ("none", "No texture"),
                    ("stripes", "Stripes"),
                    ("dots", "Dots"),
                    ("grid", "Grid"),
                ],
                4,
                theme,
            );
            self.draw_chip_grid(
                y,
                x,
                w,
                "cursorDepth",
                &cfg.cursor_depth,
                &[("flat", "Flat"), ("raised", "Raised"), ("sunken", "Sunken")],
                3,
                theme,
            );
            self.draw_slider_row(
                y,
                x,
                w,
                "Opacity (%)",
                "cursorOpacity",
                cfg.cursor_opacity as i32,
                def.cursor_opacity as i32,
                30,
                100,
                5,
                1.0,
                theme,
            );
            self.draw_slider_row(
                y,
                x,
                w,
                "Accent Tint (%)",
                "cursorTint",
                cfg.cursor_tint as i32,
                def.cursor_tint as i32,
                0,
                40,
                2,
                1.0,
                theme,
            );
        }

        if q.is_empty() || "cursor color primary second custom".contains(q) {
            self.draw_section_header(y, x, w, "Cursor Colors", theme);
            let c_opts = &[
                ("accent", "Accent"),
                ("foreground", "Text"),
                ("background", "Bg"),
                ("border", "Border"),
                ("border2", "Border 2"),
                ("custom", "Custom"),
                ("white", "White"),
                ("black", "Black"),
            ];
            self.draw_chip_grid(y, x, w, "cursorColor", &cfg.cursor_color, c_opts, 4, theme);
            if cfg.cursor_color == "custom" {
                self.draw_color_row(
                    y,
                    x,
                    w,
                    "Primary Custom",
                    "cursorColorCustom",
                    &cfg.cursor_color_custom,
                    PALETTE_PRESETS,
                    theme,
                );
            }
            self.draw_chip_grid(
                y,
                x,
                w,
                "cursorColor2",
                &cfg.cursor_color2,
                c_opts,
                4,
                theme,
            );
            if cfg.cursor_color2 == "custom" {
                self.draw_color_row(
                    y,
                    x,
                    w,
                    "Second Custom",
                    "cursorColor2Custom",
                    &cfg.cursor_color2_custom,
                    PALETTE_PRESETS,
                    theme,
                );
            }
        }

        if q.is_empty()
            || "lines outline solid double neon dashed stitched wavy gilded glitch width"
                .contains(q)
        {
            self.draw_section_header(y, x, w, "Lines (22)", theme);
            self.draw_chip_grid(y, x, w, "cursorLine", &cfg.cursor_line, LINE_KEYS, 5, theme);
            if cfg.cursor_line != "none" {
                self.draw_chip_grid(
                    y,
                    x,
                    w,
                    "cursorLineColor",
                    &cfg.cursor_line_color,
                    &[
                        ("auto", "Auto"),
                        ("subtle", "Subtle"),
                        ("accent", "Accent"),
                        ("foreground", "Text"),
                        ("background", "Bg"),
                        ("white", "White"),
                        ("black", "Black"),
                        ("custom", "Custom"),
                    ],
                    4,
                    theme,
                );
                if cfg.cursor_line_color == "custom" {
                    self.draw_color_row(
                        y,
                        x,
                        w,
                        "Custom Line Color",
                        "cursorLineCustom",
                        &cfg.cursor_line_custom,
                        PALETTE_PRESETS,
                        theme,
                    );
                }
                self.draw_slider_row(
                    y,
                    x,
                    w,
                    "Line Width",
                    "cursorLineWidth",
                    cfg.cursor_line_width as i32,
                    def.cursor_line_width as i32,
                    1,
                    4,
                    1,
                    1.0,
                    theme,
                );
                self.draw_toggle_row(
                    y,
                    x,
                    w,
                    "Gradient Lines",
                    "cursorLineGradient",
                    cfg.cursor_line_gradient,
                    theme,
                );
            }
        }

        if q.is_empty()
            || "ornaments studs pearls stars crown laurels cap detail spine spark".contains(q)
        {
            self.draw_section_header(y, x, w, "Ornaments & Details (18)", theme);
            self.draw_chip_grid(
                y,
                x,
                w,
                "cursorOrnament",
                &cfg.cursor_ornament,
                ORNAMENT_KEYS,
                5,
                theme,
            );
            if cfg.cursor_ornament != "none" {
                self.draw_slider_row(
                    y,
                    x,
                    w,
                    "Ornament Size",
                    "cursorOrnamentSize",
                    cfg.cursor_ornament_size as i32,
                    def.cursor_ornament_size as i32,
                    3,
                    16,
                    1,
                    1.0,
                    theme,
                );
                self.draw_chip_grid(
                    y,
                    x,
                    w,
                    "cursorOrnamentsOn",
                    &cfg.cursor_ornaments_on,
                    &[("arrow", "The arrow"), ("all", "Every cursor")],
                    2,
                    theme,
                );
            }
            self.draw_chip_grid(
                y,
                x,
                w,
                "cursorCap",
                &cfg.cursor_cap,
                &[
                    ("none", "No cap"),
                    ("tip", "Cap: Tip"),
                    ("tail", "Cap: Tail"),
                    ("both", "Cap: Both"),
                ],
                4,
                theme,
            );
            self.draw_chip_grid(
                y,
                x,
                w,
                "cursorDetail",
                &cfg.cursor_detail,
                &[
                    ("none", "No detail"),
                    ("tip", "Tip dot"),
                    ("spine", "Spine"),
                    ("spark", "Spark"),
                ],
                4,
                theme,
            );
        }

        if q.is_empty() || "shadow soft strong hard long glow".contains(q) {
            self.draw_section_header(y, x, w, "Shadow (5)", theme);
            self.draw_chip_grid(
                y,
                x,
                w,
                "cursorShadow",
                &cfg.cursor_shadow,
                &[
                    ("none", "None"),
                    ("soft", "Soft"),
                    ("strong", "Strong"),
                    ("hard", "Hard"),
                    ("long", "Long"),
                ],
                5,
                theme,
            );
            if cfg.cursor_shadow != "none" {
                self.draw_chip_grid(
                    y,
                    x,
                    w,
                    "cursorShadowColor",
                    &cfg.cursor_shadow_color,
                    &[("dark", "Dark"), ("accent", "Accent glow")],
                    2,
                    theme,
                );
                self.draw_chip_grid(
                    y,
                    x,
                    w,
                    "cursorShadowDir",
                    &cfg.cursor_shadow_dir,
                    &[("below", "Below"), ("around", "All around")],
                    2,
                    theme,
                );
                self.draw_slider_row(
                    y,
                    x,
                    w,
                    "Shadow Strength (%)",
                    "cursorShadowStrength",
                    cfg.cursor_shadow_strength as i32,
                    def.cursor_shadow_strength as i32,
                    10,
                    100,
                    5,
                    1.0,
                    theme,
                );
            }
        }

        if q.is_empty() || "other cursors hand links beam text busy spinner".contains(q) {
            self.draw_section_header(y, x, w, "Other Cursors (Links, Text, Busy)", theme);
            self.draw_chip_grid(
                y,
                x,
                w,
                "cursorHand",
                &cfg.cursor_hand,
                &[
                    ("hand", "Links: Hand"),
                    ("arrow", "Links: Arrow+Dot"),
                    ("ring", "Links: Ring"),
                ],
                3,
                theme,
            );
            self.draw_chip_grid(
                y,
                x,
                w,
                "cursorBeam",
                &cfg.cursor_beam,
                &[
                    ("beam", "Text: Beam"),
                    ("slim", "Text: Slim"),
                    ("caret", "Text: Caret"),
                ],
                3,
                theme,
            );
            self.draw_chip_grid(
                y,
                x,
                w,
                "cursorBusy",
                &cfg.cursor_busy,
                &[
                    ("ring", "Busy: Ring"),
                    ("dots", "Busy: Dots"),
                    ("orbit", "Busy: Orbit"),
                ],
                3,
                theme,
            );
            self.draw_chip_grid(
                y,
                x,
                w,
                "cursorBusySpeed",
                &cfg.cursor_busy_speed,
                &[
                    ("slow", "Speed: Slow"),
                    ("normal", "Speed: Normal"),
                    ("fast", "Speed: Fast"),
                ],
                3,
                theme,
            );
        }

        if q.is_empty() || "hide idle typing cursor".contains(q) {
            self.draw_section_header(y, x, w, "Cursor Hiding", theme);
            self.draw_chip_grid(
                y,
                x,
                w,
                "hideAfter",
                &cfg.hide_after,
                &[
                    ("keep", "System"),
                    ("0", "Never"),
                    ("2", "2 s"),
                    ("5", "5 s"),
                    ("10", "10 s"),
                    ("30", "30 s"),
                ],
                6,
                theme,
            );
            self.draw_chip_grid(
                y,
                x,
                w,
                "hideTyping",
                &cfg.hide_typing,
                &[
                    ("keep", "Typing: System"),
                    ("on", "Hide while typing"),
                    ("off", "Never hide typing"),
                ],
                3,
                theme,
            );
        }

        if q.is_empty() {
            self.draw_button(
                D2D_RECT_F {
                    left: x,
                    top: *y + 4.0,
                    right: x + 210.0,
                    bottom: *y + 32.0,
                },
                "Restore Windows System Cursor",
                false,
                theme,
                UiAction::RestoreSystemCursor,
            );
            *y += 42.0;
        }
    }

    unsafe fn draw_find_tab(
        &mut self,
        y: &mut f32,
        x: f32,
        w: f32,
        cfg: &Settings,
        def: &Settings,
        theme: &PanelTheme,
        q: &str,
    ) {
        if q.is_empty() || "shake find sensitivity size rings".contains(q) {
            self.draw_section_header(y, x, w, "Shake to Find", theme);
            self.draw_toggle_row(y, x, w, "Enable Shake to Find", "shake", cfg.shake, theme);
            self.draw_slider_row(
                y,
                x,
                w,
                "Sensitivity",
                "shakeSensitivity",
                cfg.shake_sensitivity as i32,
                def.shake_sensitivity as i32,
                1,
                10,
                1,
                1.0,
                theme,
            );
            self.draw_slider_row(
                y,
                x,
                w,
                "Ring Size",
                "findSize",
                cfg.find_size as i32,
                def.find_size as i32,
                30,
                160,
                5,
                1.0,
                theme,
            );
            self.draw_button(
                D2D_RECT_F {
                    left: x,
                    top: *y + 4.0,
                    right: x + 170.0,
                    bottom: *y + 32.0,
                },
                "Test Find Rings Now",
                false,
                theme,
                UiAction::TestFind,
            );
            *y += 42.0;
        }
    }

    unsafe fn draw_behavior_tab(
        &mut self,
        y: &mut f32,
        x: f32,
        w: f32,
        cfg: &Settings,
        _def: &Settings,
        theme: &PanelTheme,
        q: &str,
    ) {
        if q.is_empty() || "effects enabled fullscreen fps smoothness startup windows".contains(q) {
            self.draw_section_header(y, x, w, "Effects & Performance", theme);
            self.draw_toggle_row(y, x, w, "Effects Enabled", "enabled", cfg.enabled, theme);
            self.draw_chip_grid(
                y,
                x,
                w,
                "fullscreen",
                &cfg.fullscreen,
                &[
                    ("hide", "Fullscreen: Hide (Games/Video)"),
                    ("show", "Fullscreen: Show"),
                ],
                2,
                theme,
            );
            self.draw_chip_grid(
                y,
                x,
                w,
                "fps",
                &cfg.fps,
                &[
                    ("30", "30 FPS"),
                    ("60", "60 FPS"),
                    ("120", "120 FPS"),
                    ("auto", "Auto (Monitor Hz)"),
                ],
                4,
                theme,
            );
            self.draw_section_header(y, x, w, "Startup & Tray", theme);
            self.draw_toggle_row(
                y,
                x,
                w,
                "Start with Windows",
                "startWithWindows",
                cfg.start_with_windows,
                theme,
            );
            self.draw_chip_grid(
                y,
                x,
                w,
                "startLook",
                &cfg.start_look,
                &[
                    ("keep", "Startup: As left"),
                    ("1", "Slot 1"),
                    ("2", "Slot 2"),
                    ("3", "Slot 3"),
                    ("4", "Slot 4"),
                    ("5", "Slot 5"),
                ],
                3,
                theme,
            );
        }
    }

    unsafe fn draw_about_tab(&mut self, y: &mut f32, x: f32, w: f32, theme: &PanelTheme) {
        self.draw_section_header(y, x, w, "Winscroller Mouse v1.1.1", theme);
        let lines = [
            "A faithful, hardware-accelerated Rust & DirectComposition clone of",
            "omarchy-plume by qempexe, engineered for Windows 10/11.",
            "",
            "• GPU-accelerated DirectComposition + Direct2D flip-model overlay",
            "• 6 Trails (Catmull-Rom smoothed), 6 Halos, 8 Color Modes, Shake-to-Find",
            "• Full Custom Cursor Forge: 14 Arrows, 30 Surfaces, 22 Lines, 18 Ornaments",
            "• Native Windows .cur & animated .ani generation (zero external tools)",
            "• Idle-zero rendering + DXGI memory trimming for minimal RAM/CPU",
            "",
            "CLI Usage (while running):",
            "  winscroller-mouse.exe toggle | find | look neon | cursor candy",
            "  winscroller-mouse.exe set trail sparks | save 1 | load 1 | settings",
        ];
        for line in lines {
            self.draw_text(
                line,
                D2D_RECT_F {
                    left: x + 4.0,
                    top: *y,
                    right: x + w,
                    bottom: *y + 18.0,
                },
                &self.font_body.clone(),
                to_rgba(
                    if line.starts_with("  ") {
                        theme.accent
                    } else {
                        theme.fg
                    },
                    1.0,
                ),
            );
            *y += 19.0;
        }
        *y += 12.0;
        self.draw_button(
            D2D_RECT_F {
                left: x,
                top: *y,
                right: x + 180.0,
                bottom: *y + 30.0,
            },
            "Reset All Settings",
            false,
            theme,
            UiAction::ResetAll,
        );
        *y += 40.0;
    }

    // ---- Reusable UI Widgets ------------------------------------------------

    unsafe fn draw_section_header(
        &mut self,
        y: &mut f32,
        x: f32,
        w: f32,
        title: &str,
        theme: &PanelTheme,
    ) {
        self.draw_text(
            title,
            D2D_RECT_F {
                left: x + 2.0,
                top: *y,
                right: x + w,
                bottom: *y + 22.0,
            },
            &self.font_head.clone(),
            to_rgba(theme.accent, 1.0),
        );
        *y += 24.0;
    }

    unsafe fn draw_hint_line(&self, y: &mut f32, x: f32, w: f32, text: &str, theme: &PanelTheme) {
        self.draw_text(
            text,
            D2D_RECT_F {
                left: x + 4.0,
                top: *y - 2.0,
                right: x + w,
                bottom: *y + 16.0,
            },
            &self.font_small.clone(),
            to_rgba(theme.fg_dim, 0.92),
        );
        *y += 20.0;
    }

    unsafe fn draw_button(
        &mut self,
        rect: D2D_RECT_F,
        label: &str,
        active: bool,
        theme: &PanelTheme,
        action: UiAction,
    ) {
        let hovered = self.hover_x >= rect.left
            && self.hover_x <= rect.right
            && self.hover_y >= rect.top
            && self.hover_y <= rect.bottom;
        let bg = if active {
            theme.accent
        } else if hovered {
            theme.card_hover
        } else {
            theme.card
        };
        let fg = if active {
            contrast(theme.accent)
        } else {
            theme.fg
        };

        self.fill_rrect(rect, 6.0, to_rgba(bg, 1.0));
        self.stroke_rrect(
            rect,
            6.0,
            to_rgba(if active { theme.accent } else { theme.border }, 1.0),
            1.0,
        );
        let fmt = self.font_small.clone();
        let _ = fmt.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
        self.draw_text(label, rect, &fmt, to_rgba(fg, 1.0));
        let _ = fmt.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);

        self.hits.push(HitItem { rect, action });
    }

    unsafe fn draw_chip_grid(
        &mut self,
        y: &mut f32,
        x: f32,
        w: f32,
        key: &'static str,
        current: &str,
        options: &[(&str, &str)],
        cols: usize,
        theme: &PanelTheme,
    ) {
        let cols = cols.max(1);
        let cw = (w - (cols as f32 - 1.0) * 5.0) / cols as f32;
        for (i, &(opt_key, opt_label)) in options.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let rx = x + col as f32 * (cw + 5.0);
            let ry = *y + row as f32 * 30.0;
            self.draw_button(
                D2D_RECT_F {
                    left: rx,
                    top: ry,
                    right: rx + cw,
                    bottom: ry + 25.0,
                },
                opt_label,
                current == opt_key,
                theme,
                UiAction::SetStr(key, opt_key.to_string()),
            );
        }
        let rows = options.len().div_ceil(cols);
        *y += rows as f32 * 30.0 + 6.0;
    }

    unsafe fn draw_toggle_row(
        &mut self,
        y: &mut f32,
        x: f32,
        w: f32,
        label: &str,
        key: &'static str,
        val: bool,
        theme: &PanelTheme,
    ) {
        let row = D2D_RECT_F {
            left: x,
            top: *y,
            right: x + w,
            bottom: *y + 30.0,
        };
        self.fill_rrect(row, 6.0, to_rgba(theme.card, 0.7));
        self.draw_text(
            label,
            D2D_RECT_F {
                left: x + 10.0,
                top: *y,
                right: x + w - 70.0,
                bottom: *y + 30.0,
            },
            &self.font_body.clone(),
            to_rgba(theme.fg, 1.0),
        );
        let pill = D2D_RECT_F {
            left: x + w - 52.0,
            top: *y + 5.0,
            right: x + w - 8.0,
            bottom: *y + 25.0,
        };
        self.fill_rrect(
            pill,
            10.0,
            to_rgba(if val { theme.accent } else { theme.border }, 1.0),
        );
        let knob_x = if val {
            pill.right - 10.0
        } else {
            pill.left + 10.0
        };
        self.solid_brush
            .SetColor(&rgba_d2d(to_rgba([250.0, 250.0, 250.0], 1.0)));
        self.d2d_ctx.FillEllipse(
            &D2D1_ELLIPSE {
                point: pt(knob_x, *y + 15.0),
                radiusX: 7.0,
                radiusY: 7.0,
            },
            &self.solid_brush,
        );
        self.hits.push(HitItem {
            rect: row,
            action: UiAction::SetBool(key, !val),
        });
        *y += 35.0;
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn draw_slider_row(
        &mut self,
        y: &mut f32,
        x: f32,
        w: f32,
        label: &str,
        key: &'static str,
        val: i32,
        default_val: i32,
        min: i32,
        max: i32,
        step: i32,
        divisor: f32,
        theme: &PanelTheme,
    ) {
        let row = D2D_RECT_F {
            left: x,
            top: *y,
            right: x + w,
            bottom: *y + 32.0,
        };
        self.fill_rrect(row, 6.0, to_rgba(theme.card, 0.65));

        let val_str = if divisor > 1.0 {
            format!("{:.2}", val as f32 / divisor)
        } else {
            format!("{}", val)
        };
        self.draw_text(
            label,
            D2D_RECT_F {
                left: x + 10.0,
                top: *y,
                right: x + 165.0,
                bottom: *y + 32.0,
            },
            &self.font_body.clone(),
            to_rgba(theme.fg, 1.0),
        );

        let track_x0 = x + 170.0;
        let track_x1 = x + w - 76.0;
        let cy = *y + 16.0;
        let t = ((val - min) as f32 / (max - min).max(1) as f32).clamp(0.0, 1.0);
        let kx = track_x0 + t * (track_x1 - track_x0);

        self.fill_rrect(
            D2D_RECT_F {
                left: track_x0,
                top: cy - 2.5,
                right: track_x1,
                bottom: cy + 2.5,
            },
            2.5,
            to_rgba(theme.border, 1.0),
        );
        self.fill_rrect(
            D2D_RECT_F {
                left: track_x0,
                top: cy - 2.5,
                right: kx,
                bottom: cy + 2.5,
            },
            2.5,
            to_rgba(theme.accent, 1.0),
        );
        self.solid_brush
            .SetColor(&rgba_d2d(to_rgba(theme.accent, 1.0)));
        self.d2d_ctx.FillEllipse(
            &D2D1_ELLIPSE {
                point: pt(kx, cy),
                radiusX: 6.5,
                radiusY: 6.5,
            },
            &self.solid_brush,
        );

        self.hits.push(HitItem {
            rect: D2D_RECT_F {
                left: track_x0 - 8.0,
                top: *y,
                right: track_x1 + 8.0,
                bottom: *y + 32.0,
            },
            action: UiAction::Slider {
                key,
                min,
                max,
                step,
                x0: track_x0,
                x1: track_x1,
            },
        });

        self.draw_text(
            &val_str,
            D2D_RECT_F {
                left: track_x1 + 8.0,
                top: *y,
                right: x + w - 26.0,
                bottom: *y + 32.0,
            },
            &self.font_small.clone(),
            to_rgba(theme.fg, 1.0),
        );

        // Reset-to-default button if modified
        if val != default_val {
            let rb = D2D_RECT_F {
                left: x + w - 26.0,
                top: *y + 5.0,
                right: x + w - 4.0,
                bottom: *y + 27.0,
            };
            self.draw_button(rb, "↺", false, theme, UiAction::SetInt(key, default_val));
        }

        *y += 37.0;
    }

    unsafe fn draw_color_row(
        &mut self,
        y: &mut f32,
        x: f32,
        w: f32,
        label: &str,
        key: &'static str,
        current_hex: &str,
        presets: &[&str],
        theme: &PanelTheme,
    ) {
        let row = D2D_RECT_F {
            left: x,
            top: *y,
            right: x + w,
            bottom: *y + 32.0,
        };
        self.fill_rrect(row, 6.0, to_rgba(theme.card, 0.65));
        self.draw_text(
            label,
            D2D_RECT_F {
                left: x + 10.0,
                top: *y,
                right: x + 140.0,
                bottom: *y + 32.0,
            },
            &self.font_body.clone(),
            to_rgba(theme.fg, 1.0),
        );

        let mut sx = x + 145.0;
        for &hex in presets {
            let rgb = parse_hex(hex).unwrap_or([128.0, 128.0, 128.0]);
            let r = D2D_RECT_F {
                left: sx,
                top: *y + 6.0,
                right: sx + 20.0,
                bottom: *y + 26.0,
            };
            self.fill_rrect(r, 4.0, to_rgba(rgb, 1.0));
            let sel = current_hex.eq_ignore_ascii_case(hex);
            self.stroke_rrect(
                r,
                4.0,
                to_rgba(
                    if sel {
                        [255.0, 255.0, 255.0]
                    } else {
                        theme.border
                    },
                    1.0,
                ),
                if sel { 2.0 } else { 1.0 },
            );
            self.hits.push(HitItem {
                rect: r,
                action: UiAction::SetStr(key, hex.to_string()),
            });
            sx += 25.0;
        }

        // Current hex readout + Paste Hex button
        let paste_r = D2D_RECT_F {
            left: x + w - 92.0,
            top: *y + 4.0,
            right: x + w - 6.0,
            bottom: *y + 28.0,
        };
        self.draw_button(paste_r, current_hex, false, theme, UiAction::PasteHex(key));
        *y += 37.0;
    }

    unsafe fn fill_rrect(&self, rect: D2D_RECT_F, radius: f32, color: Rgba) {
        self.solid_brush.SetColor(&rgba_d2d(color));
        self.d2d_ctx.FillRoundedRectangle(
            &D2D1_ROUNDED_RECT {
                rect,
                radiusX: radius,
                radiusY: radius,
            },
            &self.solid_brush,
        );
    }

    unsafe fn stroke_rrect(&self, rect: D2D_RECT_F, radius: f32, color: Rgba, width: f32) {
        self.solid_brush.SetColor(&rgba_d2d(color));
        self.d2d_ctx.DrawRoundedRectangle(
            &D2D1_ROUNDED_RECT {
                rect,
                radiusX: radius,
                radiusY: radius,
            },
            &self.solid_brush,
            width,
            None,
        );
    }

    unsafe fn draw_text(&self, text: &str, rect: D2D_RECT_F, fmt: &IDWriteTextFormat, color: Rgba) {
        let wide: Vec<u16> = text.encode_utf16().collect();
        self.solid_brush.SetColor(&rgba_d2d(color));
        self.d2d_ctx.DrawText(
            &wide,
            fmt,
            &rect,
            &self.solid_brush,
            D2D1_DRAW_TEXT_OPTIONS_CLIP,
            DWRITE_MEASURING_MODE_NATURAL,
        );
    }
}

impl Drop for SettingsPanel {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}
