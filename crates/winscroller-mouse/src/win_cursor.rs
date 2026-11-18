//! Windows native cursor builder & applier:
//! - Uses `winscroller-mouse-core` SVG generator + `resvg` + CUR/ANI encoders
//! - Applies generated `.cur` and `.ani` files live to all Windows system cursor slots via `SetSystemCursor`
//! - Cleanly restores original Windows system cursors via `SystemParametersInfoW(SPI_SETCURSORS)`
//! - Supports transparent blank cursor for Hide-when-idle / Hide-while-typing

use std::path::PathBuf;
use windows::{Win32::UI::WindowsAndMessaging::*, core::*};

use winscroller_mouse_core::{
    colors::ColorCtx,
    cursor_forge::{
        encode_ani, encode_cur,
        glyphs::*,
        rasterize_svg,
        svg::{Design, FRAMES, hotspot_norm, render_svg},
    },
    settings::Settings,
};

use crate::store::local_data_dir;

fn cursors_dir() -> PathBuf {
    let dir = local_data_dir().join("cursors");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

unsafe fn apply_cursor_file(path: &std::path::Path, ocr_id: SYSTEM_CURSOR_ID) {
    let wide: Vec<u16> = path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    if let Ok(hcur) = unsafe { LoadCursorFromFileW(PCWSTR(wide.as_ptr())) } {
        let _ = unsafe { SetSystemCursor(hcur, ocr_id) };
    }
}

/// Build and apply the entire custom Winscroller Mouse cursor theme to Windows.
pub fn apply_custom_cursors(cfg: &Settings, theme: &ColorCtx) {
    let d = Design::from_settings(cfg, theme);
    let size = cfg.pointer_size.clamp(16, 96);
    let dir = cursors_dir();

    let static_specs: [(&str, SYSTEM_CURSOR_ID, Glyph); 11] = [
        ("arrow", OCR_NORMAL, arrow_glyph(&d.arrow)),
        ("hand", OCR_HAND, pointing_glyph(&d.hand, &d.arrow)),
        ("ibeam", OCR_IBEAM, beam_glyph(&d.beam, 0.0)),
        ("cross", OCR_CROSS, crosshair_glyph()),
        ("no", OCR_NO, not_allowed_glyph()),
        ("sizeall", OCR_SIZEALL, move_glyph()),
        ("sizens", OCR_SIZENS, double_arrow_glyph(0.0)),
        ("sizewe", OCR_SIZEWE, double_arrow_glyph(90.0)),
        ("sizenesw", OCR_SIZENESW, double_arrow_glyph(45.0)),
        ("sizenwse", OCR_SIZENWSE, double_arrow_glyph(-45.0)),
        ("up", OCR_UP, up_arrow_glyph()),
    ];

    for (name, ocr_id, glyph) in &static_specs {
        let svg = render_svg(glyph, &d);
        if let Some(rgba) = rasterize_svg(&svg, size) {
            let (hx, hy) = hotspot_norm(glyph, &d);
            let hot_x = ((hx * size as f64).round() as u16).min((size - 1) as u16);
            let hot_y = ((hy * size as f64).round() as u16).min((size - 1) as u16);
            if let Some(cur) = encode_cur(&rgba, size, hot_x, hot_y) {
                let path = dir.join(format!("{name}.cur"));
                if std::fs::write(&path, cur).is_ok() {
                    unsafe { apply_cursor_file(&path, *ocr_id) };
                }
            }
        }
    }

    // Animated busy cursor (OCR_WAIT)
    {
        let mut frames = Vec::with_capacity(FRAMES);
        for f in 0..FRAMES {
            let g = wait_glyph(&d.busy, f, FRAMES);
            let svg = render_svg(&g, &d);
            if let Some(rgba) = rasterize_svg(&svg, size) {
                let (hx, hy) = hotspot_norm(&g, &d);
                let hot_x = ((hx * size as f64).round() as u16).min((size - 1) as u16);
                let hot_y = ((hy * size as f64).round() as u16).min((size - 1) as u16);
                if let Some(cur) = encode_cur(&rgba, size, hot_x, hot_y) {
                    frames.push(cur);
                }
            }
        }
        if !frames.is_empty() {
            let ani = encode_ani(&frames, d.delay_ms);
            let path = dir.join("wait.ani");
            if std::fs::write(&path, ani).is_ok() {
                unsafe { apply_cursor_file(&path, OCR_WAIT) };
            }
        }
    }

    // Animated background-busy cursor (OCR_APPSTARTING)
    {
        let mut frames = Vec::with_capacity(FRAMES);
        for f in 0..FRAMES {
            let g = progress_glyph(&d.arrow, &d.busy, f, FRAMES);
            let svg = render_svg(&g, &d);
            if let Some(rgba) = rasterize_svg(&svg, size) {
                let (hx, hy) = hotspot_norm(&g, &d);
                let hot_x = ((hx * size as f64).round() as u16).min((size - 1) as u16);
                let hot_y = ((hy * size as f64).round() as u16).min((size - 1) as u16);
                if let Some(cur) = encode_cur(&rgba, size, hot_x, hot_y) {
                    frames.push(cur);
                }
            }
        }
        if !frames.is_empty() {
            let ani = encode_ani(&frames, d.delay_ms);
            let path = dir.join("appstarting.ani");
            if std::fs::write(&path, ani).is_ok() {
                unsafe { apply_cursor_file(&path, OCR_APPSTARTING) };
            }
        }
    }

    // Help cursor (OCR_HELP = 32651)
    {
        let g = help_glyph(&d.arrow);
        let svg = render_svg(&g, &d);
        if let Some(rgba) = rasterize_svg(&svg, size) {
            let (hx, hy) = hotspot_norm(&g, &d);
            let hot_x = ((hx * size as f64).round() as u16).min((size - 1) as u16);
            let hot_y = ((hy * size as f64).round() as u16).min((size - 1) as u16);
            if let Some(cur) = encode_cur(&rgba, size, hot_x, hot_y) {
                let path = dir.join("help.cur");
                if std::fs::write(&path, cur).is_ok() {
                    unsafe { apply_cursor_file(&path, SYSTEM_CURSOR_ID(32651)) };
                }
            }
        }
    }
}

/// Restore the user's original Windows system cursors immediately.
pub fn restore_system_cursors() {
    unsafe {
        let _ = SystemParametersInfoW(
            SPI_SETCURSORS,
            0,
            None,
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
    }
}

/// Apply a completely transparent 16x16 cursor (for Hide-when-idle / Hide-while-typing).
pub fn hide_system_cursor() {
    let dir = cursors_dir();
    let path = dir.join("blank.cur");
    if !path.exists() {
        let rgba = vec![0u8; 16 * 16 * 4];
        if let Some(cur) = encode_cur(&rgba, 16, 0, 0) {
            let _ = std::fs::write(&path, cur);
        }
    }
    let slots = [
        OCR_NORMAL,
        OCR_IBEAM,
        OCR_HAND,
        OCR_CROSS,
        OCR_WAIT,
        OCR_APPSTARTING,
        OCR_SIZEALL,
        OCR_SIZENS,
        OCR_SIZEWE,
        OCR_SIZENESW,
        OCR_SIZENWSE,
        OCR_NO,
    ];
    for ocr in slots {
        unsafe { apply_cursor_file(&path, ocr) };
    }
}

/// Render a preview RGBA bitmap of any cursor glyph (for the Settings Panel light/dark showcase).
pub fn render_preview_rgba(
    cfg: &Settings,
    theme: &ColorCtx,
    kind: &str,
    size: u32,
) -> Option<Vec<u8>> {
    let d = Design::from_settings(cfg, theme);
    let glyph = match kind {
        "hand" => pointing_glyph(&d.hand, &d.arrow),
        "beam" => beam_glyph(&d.beam, 0.0),
        "wait" => wait_glyph(&d.busy, 2, FRAMES),
        "help" => help_glyph(&d.arrow),
        "move" => move_glyph(),
        "copy" => copy_glyph(&d.arrow),
        "alias" => alias_glyph(&d.arrow),
        "menu" => menu_glyph(&d.arrow),
        _ => arrow_glyph(&d.arrow),
    };
    let svg = render_svg(&glyph, &d);
    rasterize_svg(&svg, size)
}
