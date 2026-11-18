//! Comprehensive OS-agnostic test suite for `winscroller-mouse-core`.
//! Covers motion physics, shake-to-find, all 8 color modes, all 6 trails, all 6 halos,
//! all 14 arrows, 30 surfaces, 22 lines, 18 ornaments, 12 looks, 16 cursors,
//! CUR/ANI/ICO binary encoding, and `winscroller-mouse.example.json` schema validation.

use winscroller_mouse_core::{
    colors::{ColorCtx, ColorMode, color_at},
    cursor_forge::{
        arrows::ARROW_KEYS,
        encode_ani, encode_cur, encode_ico, encode_png,
        glyphs::arrow_glyph,
        lines::LINE_KEYS,
        ornaments::ORNAMENT_KEYS,
        rasterize_svg,
        surfaces::SURFACE_KEYS,
        svg::{Design, render_svg},
    },
    draw::DrawCtx,
    motion::{MotionState, Point},
    paint::{FrameState, paint_frame_into},
    settings::{
        AppStateFile, HaloKind, SLOTS, Settings, TrailKind, builtin_cursors, builtin_looks,
    },
    trails::TrailMemory,
};

#[test]
fn example_json_parses_cleanly_and_matches_defaults() {
    let raw = include_str!("../../../winscroller-mouse.example.json");
    let parsed: AppStateFile =
        serde_json::from_str(raw).expect("winscroller-mouse.example.json must be valid JSON");
    assert_eq!(parsed.settings, Settings::default());
    assert_eq!(parsed.slots.len(), SLOTS);
}

#[test]
fn motion_tracks_speed_prunes_and_detects_shake() {
    let mut m = MotionState::new();
    let mut t = 100.0;
    let mut shook = false;
    for i in 0..12 {
        let x = if i % 2 == 0 { 100.0 } else { 240.0 };
        assert!(m.add(x, 120.0, t));
        if m.shaken(t, 5) {
            shook = true;
            break;
        }
        t += 40.0;
    }
    assert!(
        shook,
        "expected shake-to-find to trigger on rapid reversals"
    );
    assert!(m.speed01() > 0.0);

    // Prune everything after life expires
    m.prune(t + 5000.0, 0.45);
    let mut pts: Vec<Point> = Vec::new();
    m.points_for_into(t + 5000.0, 0.45, 0.0, 0.0, &mut pts);
    assert!(pts.is_empty());
}

#[test]
fn all_color_modes_produce_valid_rgb() {
    let ctx = ColorCtx {
        speed: 0.75,
        time: 4.2,
        ..ColorCtx::default()
    };
    for &mode in ColorMode::ALL {
        for &t in &[0.0f32, 0.25, 0.5, 0.75, 1.0] {
            let rgb = color_at(mode, t, &ctx);
            assert!(
                rgb.iter()
                    .all(|c| c.is_finite() && *c >= 0.0 && *c <= 255.0),
                "invalid rgb for {:?} at t={}: {:?}",
                mode,
                t,
                rgb
            );
        }
    }
}

#[test]
fn all_trails_and_halos_paint_cleanly() {
    let theme = ColorCtx::default();
    let pts = vec![
        Point {
            x: 200.0,
            y: 200.0,
            t: 0.0,
        },
        Point {
            x: 185.0,
            y: 192.0,
            t: 0.2,
        },
        Point {
            x: 160.0,
            y: 180.0,
            t: 0.45,
        },
        Point {
            x: 130.0,
            y: 165.0,
            t: 0.7,
        },
        Point {
            x: 100.0,
            y: 150.0,
            t: 0.95,
        },
    ];
    let mut mem = TrailMemory::new();
    let mut draw = DrawCtx::new(800.0, 600.0);

    for &trail in TrailKind::ALL {
        for &halo in HaloKind::ALL {
            let cfg = Settings {
                trail,
                halo,
                ..Settings::default()
            };
            let mut fs = FrameState {
                settings: &cfg,
                pts: &pts,
                cursor_x: 200.0,
                cursor_y: 200.0,
                screen_x: 0.0,
                screen_y: 0.0,
                screen_w: 800.0,
                screen_h: 600.0,
                cursor_on_this_screen: true,
                color_ctx: &theme,
                time_secs: 1.25,
                dt: 0.016,
                moved: 18.0,
                still_secs: 0.0,
                find_age: 0.3,
                trail_mem: &mut mem,
            };
            let busy = paint_frame_into(&mut draw, &mut fs);
            assert!(busy, "find burst should keep frame busy");
            assert!(!draw.cmds.is_empty());
        }
    }
}

#[test]
fn all_cursor_parts_render_and_rasterize() {
    let theme = ColorCtx::default();
    assert_eq!(ARROW_KEYS.len(), 14);
    assert_eq!(SURFACE_KEYS.len(), 30);
    assert_eq!(LINE_KEYS.len(), 22);
    assert_eq!(ORNAMENT_KEYS.len(), 18);

    for &(ak, _) in ARROW_KEYS {
        let s = Settings {
            cursor_shape: ak.into(),
            ..Settings::default()
        };
        let d = Design::from_settings(&s, &theme);
        let svg = render_svg(&arrow_glyph(&d.arrow), &d);
        let rgba = rasterize_svg(&svg, 32).expect("arrow rasterize");
        assert_eq!(rgba.len(), 32 * 32 * 4);
        assert!(
            rgba.as_chunks::<4>().0.iter().any(|p| p[3] > 0),
            "empty arrow: {ak}"
        );
    }

    for &(sk, _) in SURFACE_KEYS {
        let s = Settings {
            cursor_surface: sk.into(),
            ..Settings::default()
        };
        let d = Design::from_settings(&s, &theme);
        let svg = render_svg(&arrow_glyph(&d.arrow), &d);
        assert!(rasterize_svg(&svg, 24).is_some(), "surface failed: {sk}");
    }

    for &(lk, _) in LINE_KEYS {
        let s = Settings {
            cursor_line: lk.into(),
            ..Settings::default()
        };
        let d = Design::from_settings(&s, &theme);
        let svg = render_svg(&arrow_glyph(&d.arrow), &d);
        assert!(rasterize_svg(&svg, 24).is_some(), "line failed: {lk}");
    }

    for &(ok, _) in ORNAMENT_KEYS {
        let s = Settings {
            cursor_ornament: ok.into(),
            ..Settings::default()
        };
        let d = Design::from_settings(&s, &theme);
        let svg = render_svg(&arrow_glyph(&d.arrow), &d);
        assert!(rasterize_svg(&svg, 24).is_some(), "ornament failed: {ok}");
    }

    // CUR, ANI, and ICO binary header checks
    let rgba = vec![255u8; 16 * 16 * 4];
    let cur = encode_cur(&rgba, 16, 2, 2).expect("cur encode");
    assert_eq!(&cur[0..6], &[0, 0, 2, 0, 1, 0]);
    let ani = encode_ani(&[cur.clone(), cur], 70);
    assert_eq!(&ani[0..4], b"RIFF");
    assert_eq!(&ani[8..12], b"ACON");
    let png_bytes = encode_png(&rgba, 16, 16).expect("png encode");
    let ico = encode_ico(&[(16, png_bytes)]);
    assert_eq!(&ico[0..6], &[0, 0, 1, 0, 1, 0]);
}

#[test]
fn looks_cursors_and_cli_set_roundtrip() {
    assert_eq!(builtin_looks().len(), 12);
    assert_eq!(builtin_cursors().len(), 16);

    let mut s = Settings::default();
    builtin_looks()[1].data.apply(&mut s);
    let exported = s.export_look_json();
    let mut s2 = Settings::default();
    assert!(s2.import_look_json(&exported));
    assert_eq!(s2.trail, s.trail);
    assert_eq!(s2.halo, s.halo);
    assert_eq!(s2.color_mode, s.color_mode);

    assert!(s.set_by_key("trail", "sparks").is_ok());
    assert_eq!(s.trail, TrailKind::Sparks);
    assert!(s.set_by_key("unknownKey", "x").is_err());
}
