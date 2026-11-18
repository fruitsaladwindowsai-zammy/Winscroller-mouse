//! Settings -> Design -> SVG generator — 1-to-1 port of `Cursor.js`.

use std::f64::consts::PI;

use crate::colors::{ColorCtx, Rgb, contrast, mix_rgb, parse_hex, to_hex};
use crate::settings::Settings;

use super::geo::{Piece, Pt, Rng, area, f, offset, poly_d, polygon_of};
use super::glyphs::{Glyph, Layer};
use super::lines::{LineCtx, draw_line, is_traced, line_reach};
use super::ornaments::{OrnamentCtx, Side, draw_ornament};
use super::surfaces::{SurfaceCtx, is_open, no_shadow, paint_surface};

pub const SIZE: f64 = 32.0;
pub const FRAMES: usize = 12;

const WIDTHS: [f64; 4] = [0.55, 0.9, 1.3, 1.8];

pub fn light(hex: &str, t: f64) -> String {
    let c = parse_hex(hex).unwrap_or([128.0, 128.0, 128.0]);
    to_hex(mix_rgb(c, [255.0, 255.0, 255.0], t as f32))
}

pub fn dark(hex: &str, t: f64) -> String {
    let c = parse_hex(hex).unwrap_or([128.0, 128.0, 128.0]);
    to_hex(mix_rgb(c, [0.0, 0.0, 0.0], t as f32))
}

fn pick(key: &str, custom: &str, theme: &ColorCtx) -> Rgb {
    match key {
        "accent" => theme.accent,
        "foreground" => theme.foreground,
        "background" => theme.background,
        "border" => theme.border.first().copied().unwrap_or(theme.accent),
        "border2" => theme
            .border
            .get(1)
            .copied()
            .or_else(|| theme.border.first().copied())
            .unwrap_or(theme.foreground),
        "white" => [250.0, 250.0, 250.0],
        "black" => [16.0, 16.0, 16.0],
        _ => parse_hex(custom).unwrap_or([122.0, 162.0, 247.0]),
    }
}

fn next_color(c: Rgb, theme: &ColorCtx) -> Rgb {
    for &p in &theme.palette {
        if (p[0] - c[0]).abs() + (p[1] - c[1]).abs() + (p[2] - c[2]).abs() > 40.0 {
            return p;
        }
    }
    theme.accent
}

fn toward(key: &str, c1: Rgb, c2: Rgb, theme: &ColorCtx) -> Rgb {
    match key {
        "second" => c2,
        "lighter" => mix_rgb(c1, [255.0, 255.0, 255.0], 0.45),
        "darker" => mix_rgb(c1, [0.0, 0.0, 0.0], 0.45),
        "accent" => pick("accent", "", theme),
        "palette" => next_color(c1, theme),
        "border" => pick("border", "", theme),
        _ => c2,
    }
}

#[derive(Clone, Debug)]
pub struct Design {
    pub arrow: String,
    pub tilt: f64,
    pub round: f64,
    pub surface: String,
    pub fill: String,
    pub grad_to: String,
    pub grad_dir: String,
    pub texture: String,
    pub depth: String,
    pub opacity: f64,
    pub c1: String,
    pub c2: String,
    pub acc: String,
    pub line: String,
    pub line_color: String,
    pub line_width: f64,
    pub line_fade: bool,
    pub line_fade_to: String,
    pub line_fade_dir: String,
    pub border_angle: f64,
    pub ornament: String,
    pub ornament_size: f64,
    pub ornaments_on: String,
    pub cap: String,
    pub ink: String,
    pub badge_ink: String,
    pub shadow: String,
    pub shadow_color: String,
    pub shadow_dir: String,
    pub shadow_strength: f64,
    pub detail: String,
    pub hand: String,
    pub beam: String,
    pub busy: String,
    pub delay_ms: u32,
}

impl Design {
    pub fn from_settings(cfg: &Settings, theme: &ColorCtx) -> Self {
        let acc = pick("accent", "", theme);
        let mut c1 = pick(&cfg.cursor_color, &cfg.cursor_color_custom, theme);
        c1 = mix_rgb(c1, acc, (cfg.cursor_tint as f32) / 100.0);
        let c2 = pick(&cfg.cursor_color2, &cfg.cursor_color2_custom, theme);

        let surf = cfg.cursor_surface.clone();
        let see = is_open(&surf) || cfg.cursor_fill == "glass";
        let mut line = cfg.cursor_line.clone();
        let mut line_col = match cfg.cursor_line_color.as_str() {
            "auto" => {
                if see {
                    c1
                } else {
                    contrast(c1)
                }
            }
            "subtle" => mix_rgb(contrast(c1), c1, 0.55),
            other => pick(other, &cfg.cursor_line_custom, theme),
        };
        if (surf == "clear" || surf == "shadow") && line == "none" {
            line = "solid".into();
            line_col = c1;
        }
        let lw_idx = (cfg.cursor_line_width.clamp(1, 4) - 1) as usize;
        let line_fade_to = if cfg.cursor_line_fade_to == "lighter" {
            mix_rgb(line_col, [255.0, 255.0, 255.0], 0.55)
        } else {
            toward(&cfg.cursor_line_fade_to, line_col, c2, theme)
        };
        let delay_ms = match cfg.cursor_busy_speed.as_str() {
            "slow" => 110,
            "fast" => 45,
            _ => 70,
        };

        Design {
            arrow: cfg.cursor_shape.clone(),
            tilt: cfg.cursor_tilt as f64,
            round: (cfg.cursor_round as f64) / 100.0 * 2.0,
            surface: surf.clone(),
            fill: if cfg.cursor_fill == "gradient" || cfg.cursor_fill == "glass" {
                cfg.cursor_fill.clone()
            } else {
                "solid".into()
            },
            grad_to: to_hex(toward(&cfg.cursor_gradient_to, c1, c2, theme)),
            grad_dir: cfg.cursor_gradient_dir.clone(),
            texture: cfg.cursor_pattern.clone(),
            depth: cfg.cursor_depth.clone(),
            opacity: ((cfg.cursor_opacity as f64) / 100.0).clamp(0.3, 1.0),
            c1: to_hex(c1),
            c2: to_hex(c2),
            acc: to_hex(acc),
            line,
            line_color: to_hex(line_col),
            line_width: WIDTHS[lw_idx],
            line_fade: cfg.cursor_line_gradient,
            line_fade_to: to_hex(line_fade_to),
            line_fade_dir: cfg.cursor_line_fade_dir.clone(),
            border_angle: theme.border_angle as f64,
            ornament: cfg.cursor_ornament.clone(),
            ornament_size: (cfg.cursor_ornament_size.max(3) as f64) / 5.0,
            ornaments_on: cfg.cursor_ornaments_on.clone(),
            cap: cfg.cursor_cap.clone(),
            ink: to_hex(if see { line_col } else { contrast(c1) }),
            badge_ink: to_hex(contrast(c2)),
            shadow: if no_shadow(&surf) {
                "none".into()
            } else {
                cfg.cursor_shadow.clone()
            },
            shadow_color: cfg.cursor_shadow_color.clone(),
            shadow_dir: cfg.cursor_shadow_dir.clone(),
            shadow_strength: (cfg.cursor_shadow_strength as f64) / 100.0,
            detail: cfg.cursor_detail.clone(),
            hand: cfg.cursor_hand.clone(),
            beam: cfg.cursor_beam.clone(),
            busy: cfg.cursor_busy.clone(),
            delay_ms,
        }
    }

    pub fn outer_grow(&self) -> f64 {
        self.round + line_reach(&self.line, self.line_width)
    }
}

#[derive(Default)]
pub struct Kit {
    pub defs: String,
    n: usize,
}

impl Kit {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn id(&mut self, prefix: &str) -> String {
        self.n += 1;
        format!("{}{}", prefix, self.n)
    }
    pub fn def(&mut self, s: &str) {
        self.defs.push_str(s);
    }
    fn stops(list: &[(f64, &str, f64)]) -> String {
        let mut s = String::new();
        for &(off, col, op) in list {
            s.push_str(&format!(
                "<stop offset=\"{}\" stop-color=\"{}\"",
                f(off),
                col
            ));
            if op < 0.999 {
                s.push_str(&format!(" stop-opacity=\"{}\"", f(op)));
            }
            s.push_str("/>");
        }
        s
    }
    pub fn lin(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, list: &[(f64, &str, f64)]) -> String {
        let i = self.id("g");
        self.def(&format!(
            "<linearGradient id=\"{}\" gradientUnits=\"userSpaceOnUse\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\">{}</linearGradient>",
            i, f(x1), f(y1), f(x2), f(y2), Self::stops(list)
        ));
        i
    }
    pub fn rad(&mut self, cx: f64, cy: f64, r: f64, list: &[(f64, &str, f64)]) -> String {
        let i = self.id("r");
        self.def(&format!(
            "<radialGradient id=\"{}\" gradientUnits=\"userSpaceOnUse\" cx=\"{}\" cy=\"{}\" r=\"{}\">{}</radialGradient>",
            i, f(cx), f(cy), f(r.max(0.1)), Self::stops(list)
        ));
        i
    }
    pub fn pat(&mut self, w: f64, h: f64, body: &str, deg: f64) -> String {
        let i = self.id("p");
        let tf = if deg != 0.0 {
            format!(" patternTransform=\"rotate({})\"", f(deg))
        } else {
            String::new()
        };
        self.def(&format!(
            "<pattern id=\"{}\" patternUnits=\"userSpaceOnUse\" width=\"{}\" height=\"{}\"{}>{}</pattern>",
            i, f(w), f(h), tf, body
        ));
        i
    }
    pub fn blur(&mut self, sd: f64) -> String {
        let i = self.id("b");
        self.def(&format!(
            "<filter id=\"{}\" filterUnits=\"userSpaceOnUse\" x=\"-4\" y=\"-4\" width=\"40\" height=\"40\"><feGaussianBlur stdDeviation=\"{}\"/></filter>",
            i, f(sd)
        ));
        i
    }
    pub fn mask(&mut self, body: &str, inner: &str) -> String {
        let i = self.id("m");
        self.def(&format!(
            "<mask id=\"{}\" maskUnits=\"userSpaceOnUse\" x=\"-4\" y=\"-4\" width=\"40\" height=\"40\">{}</mask>",
            i, body
        ));
        format!("<g mask=\"url(#{})\">{}</g>", i, inner)
    }
}

fn el(p: &Piece, paint: &str, grow: f64) -> String {
    let op_val = p.op();
    let op = if op_val < 0.999 {
        format!(" opacity=\"{}\"", f(op_val))
    } else {
        String::new()
    };
    if let Piece::Stroke { d, w, .. } = p {
        let ww = w + 2.0 * grow;
        if ww <= 0.02 {
            return String::new();
        }
        return format!(
            "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"{}/>",
            d,
            paint,
            f(ww),
            op
        );
    }
    if grow < -0.001 {
        if let Some(pg) = polygon_of(p) {
            return format!(
                "<path d=\"{}\" fill=\"{}\"{}/>",
                poly_d(&offset(&pg, grow)),
                paint,
                op
            );
        }
        return String::new();
    }
    let st = if grow > 0.001 {
        format!(
            " stroke=\"{}\" stroke-width=\"{}\" stroke-linejoin=\"round\"",
            paint,
            f(2.0 * grow)
        )
    } else {
        String::new()
    };
    match p {
        Piece::Circle { cx, cy, r, .. } => format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"{}{}/>",
            f(*cx),
            f(*cy),
            f(*r),
            paint,
            st,
            op
        ),
        Piece::Rect {
            x, y, w, h, r, rot, ..
        } => {
            let tf = if *rot != 0.0 {
                format!(
                    " transform=\"rotate({} {} {})\"",
                    f(*rot),
                    f(x + w * 0.5),
                    f(y + h * 0.5)
                )
            } else {
                String::new()
            };
            format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{}\"{}{}{}/>",
                f(*x),
                f(*y),
                f(*w),
                f(*h),
                f(*r),
                paint,
                st,
                op,
                tf
            )
        }
        Piece::Poly { p, .. } => {
            format!("<path d=\"{}\" fill=\"{}\"{}{}/>", poly_d(p), paint, st, op)
        }
        Piece::Stroke { .. } => unreachable!(),
    }
}

fn silhouette(pieces: &[Piece], paint: &str, grow: f64) -> String {
    pieces.iter().map(|p| el(p, paint, grow)).collect()
}

fn across(b: [f64; 4], dir: &str, tip: Pt, tail: Pt, angle: f64) -> [f64; 4] {
    let cx = (b[0] + b[2]) * 0.5;
    let cy = (b[1] + b[3]) * 0.5;
    if dir == "along" {
        return [tip[0], tip[1], tail[0], tail[1]];
    }
    if dir == "across" {
        return [b[0], cy, b[2], cy];
    }
    if dir == "theme" {
        let a = angle * PI / 180.0;
        let (dy, dx) = a.sin_cos();
        let half = (dx.abs() * (b[2] - b[0]) + dy.abs() * (b[3] - b[1])) * 0.5;
        return [
            cx - dx * half,
            cy - dy * half,
            cx + dx * half,
            cy + dy * half,
        ];
    }
    [b[0], b[1], b[2], b[3]]
}

fn tail_of(glyph: &Glyph, layer: &Layer) -> Pt {
    let tip = glyph.tip.unwrap_or([layer.box_[0], layer.box_[1]]);
    if layer.pieces.len() == 1
        && let Some(pg) = polygon_of(&layer.pieces[0])
    {
        let mut best = pg[0];
        let mut bd = -1.0;
        for p in pg {
            let dd = (p[0] - tip[0]).hypot(p[1] - tip[1]);
            if dd > bd {
                bd = dd;
                best = p;
            }
        }
        return best;
    }
    [layer.box_[2], layer.box_[3]]
}

fn seed_of(glyph: &Glyph, i: usize) -> i64 {
    let b = glyph.layers[i].box_;
    ((b[0] * 7.0 + b[1] * 13.0 + b[2] * 17.0 + b[3] * 19.0) * 10.0).round() as i64
        + (i as i64) * 101
}

/// Render a glyph + design to a self-contained 32x32 SVG string.
pub fn render_svg(glyph: &Glyph, d: &Design) -> String {
    let mut k = Kit::new();
    let open = is_open(&d.surface) || d.fill == "glass";
    let r = d.round;

    let turn = if glyph.rot != 0.0 {
        format!("rotate({} 16 16) ", f(glyph.rot))
    } else {
        String::new()
    };
    let tilt = if glyph.tip.is_some() && d.tilt != 0.0 {
        format!(
            "rotate({} {} {}) ",
            f(d.tilt),
            f(glyph.hot[0]),
            f(glyph.hot[1])
        )
    } else {
        String::new()
    };
    let xform = format!("{}{}", tilt, turn);
    let xform_trim = xform.trim().to_string();
    let frame = |inner: &str| -> String {
        if xform_trim.is_empty() {
            inner.to_string()
        } else {
            format!("<g transform=\"{}\">{}</g>", xform_trim, inner)
        }
    };

    let mut body = String::new();
    for (li, layer) in glyph.layers.iter().enumerate() {
        let pieces = &layer.pieces;
        let b = layer.box_;
        let tip = glyph.tip.unwrap_or([b[0], b[1]]);
        let tail = tail_of(glyph, layer);
        let sil_fn = |paint: &str, grow: f64| silhouette(pieces, paint, r + grow);

        if layer.badge {
            if d.line != "none" {
                body.push_str(&sil_fn(&d.line_color, d.line_width.min(0.9)));
            }
            body.push_str(&sil_fn(&d.c2, 0.0));
            if !layer.marks.is_empty() {
                body.push_str(&silhouette(&layer.marks, &d.badge_ink, 0.0));
            }
            continue;
        }

        let base = if d.fill == "gradient" {
            let g = across(b, &d.grad_dir, tip, tail, d.border_angle);
            format!(
                "url(#{})",
                k.lin(
                    g[0],
                    g[1],
                    g[2],
                    g[3],
                    &[(0.0, &d.c1, 1.0), (1.0, &d.grad_to, 1.0)]
                )
            )
        } else {
            d.c1.clone()
        };

        let clip_mask_sil = sil_fn("#fff", 0.0);
        let base_fill_svg = if d.fill == "glass" {
            let gl = k.lin(
                b[0],
                b[1],
                b[2],
                b[3],
                &[(0.0, "#fff", 0.7), (0.35, "#fff", 0.12), (1.0, "#fff", 0.0)],
            );
            let sheen = k.mask(
                &clip_mask_sil,
                &format!(
                    "<rect x=\"-4\" y=\"-4\" width=\"40\" height=\"40\" fill=\"url(#{})\"/>",
                    gl
                ),
            );
            format!("<g opacity=\".45\">{}</g>{}", sil_fn(&base, 0.0), sheen)
        } else {
            sil_fn(&base, 0.0)
        };

        let mut lower = String::new();
        let mut rnd = Rng::new(seed_of(glyph, li));
        let mut fill_svg = {
            let mut sc = SurfaceCtx {
                k: &mut k,
                c1: &d.c1,
                c2: &d.c2,
                acc: &d.acc,
                ink: &d.ink,
                box_: b,
                cx: (b[0] + b[2]) * 0.5,
                cy: (b[1] + b[3]) * 0.5,
                w: b[2] - b[0],
                h: b[3] - b[1],
                tip,
                tail,
                rnd: &mut rnd,
                base_fill_svg: &base_fill_svg,
                sil_fn: &sil_fn,
                clip_mask_sil: &clip_mask_sil,
                lower: &mut lower,
            };
            paint_surface(&d.surface, &mut sc)
        };

        let solid_inside = d.surface != "shadow" && d.surface != "clear";
        if solid_inside && d.texture != "none" {
            let c = &d.ink;
            let pat = match d.texture.as_str() {
                "stripes" => k.pat(1.6, 1.6, &format!("<rect width=\".55\" height=\"1.6\" fill=\"{}\"/>", c), 45.0),
                "dots" => k.pat(1.5, 1.5, &format!("<circle cx=\".75\" cy=\".75\" r=\".3\" fill=\"{}\"/>", c), 0.0),
                _ => k.pat(1.6, 1.6, &format!("<rect width=\"1.6\" height=\".18\" fill=\"{c}\"/><rect width=\".18\" height=\"1.6\" fill=\"{c}\"/>"), 0.0),
            };
            let tex = k.mask(&clip_mask_sil, &format!("<rect x=\"-4\" y=\"-4\" width=\"40\" height=\"40\" fill=\"url(#{})\" opacity=\".26\"/>", pat));
            fill_svg.push_str(&tex);
        }

        if solid_inside && (d.depth == "raised" || d.depth == "sunken") {
            let up = d.depth == "raised";
            let mut add_rim = |dx: f64, dy: f64, paint: &str, a: f64| {
                let m = format!(
                    "{}<g transform=\"translate({} {})\">{}</g>",
                    sil_fn("#fff", 0.0),
                    f(dx),
                    f(dy),
                    sil_fn("#000", 0.0)
                );
                let inner = format!("<g opacity=\"{}\">{}</g>", f(a), sil_fn(paint, 0.0));
                fill_svg.push_str(&k.mask(&m, &inner));
            };
            add_rim(
                0.65,
                0.65,
                if up { "#fff" } else { "#000" },
                if up { 0.5 } else { 0.4 },
            );
            add_rim(
                -0.65,
                -0.65,
                if up { "#000" } else { "#fff" },
                if up { 0.4 } else { 0.35 },
            );
        }

        // Line
        let line_paint = if d.line_fade && d.line != "none" {
            let g = across(b, &d.line_fade_dir, tip, tail, d.border_angle);
            format!(
                "url(#{})",
                k.lin(
                    g[0],
                    g[1],
                    g[2],
                    g[3],
                    &[(0.0, &d.line_color, 1.0), (1.0, &d.line_fade_to, 1.0)]
                )
            )
        } else {
            d.line_color.clone()
        };

        let mut line_under = String::new();
        let mut line_over = String::new();

        if is_traced(&d.line) {
            let strokes: Vec<Piece> = pieces.iter().filter(|p| p.is_stroke()).cloned().collect();
            if !strokes.is_empty() {
                let m = format!(
                    "{}{}",
                    silhouette(&strokes, "#fff", r + d.line_width),
                    silhouette(&strokes, "#000", r)
                );
                let inner = silhouette(&strokes, &line_paint, r + d.line_width);
                line_over.push_str(&k.mask(&m, &inner));
            }
        }

        {
            let mut lc = LineCtx {
                k: &mut k,
                c: &d.line_color,
                paint: &line_paint,
                w: d.line_width,
                r,
                open,
                box_: b,
                cx: (b[0] + b[2]) * 0.5,
                cy: (b[1] + b[3]) * 0.5,
                pieces,
                sil_fn: &sil_fn,
                line_under: &mut line_under,
                line_over: &mut line_over,
            };
            draw_line(&d.line, &mut lc);
        }

        let all = format!("{}{}{}{}", lower, line_under, fill_svg, line_over);
        if d.opacity < 0.999 {
            body.push_str(&format!("<g opacity=\"{}\">{}</g>", f(d.opacity), all));
        } else {
            body.push_str(&all);
        }
    }

    if !glyph.creases.is_empty() && d.surface != "clear" && d.surface != "shadow" {
        body.push_str(&silhouette(&glyph.creases, &d.ink, 0.0));
    }

    body.push_str(&render_detail(glyph, d));
    body.push_str(&render_ornament(glyph, d, &mut k));
    body.push_str(&render_cap(glyph, d));

    // Shadow
    let shadow = render_shadow(glyph, d, open, &frame, &mut k);

    let defs = if k.defs.is_empty() {
        String::new()
    } else {
        format!("<defs>{}</defs>", k.defs)
    };
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"32\" height=\"32\" viewBox=\"0 0 32 32\">{}{}{}</svg>",
        defs,
        shadow,
        frame(&body)
    )
}

fn render_shadow(
    glyph: &Glyph,
    d: &Design,
    open: bool,
    frame: &dyn Fn(&str) -> String,
    k: &mut Kit,
) -> String {
    if d.shadow == "none" || d.shadow_strength <= 0.0 {
        return String::new();
    }
    let (dist, blur, alpha, long_steps): (f64, f64, f64, usize) = match d.shadow.as_str() {
        "soft" => (1.2, 1.05, 0.85, 0),
        "strong" => (1.8, 1.6, 1.0, 0),
        "hard" => (1.25, 0.0, 0.7, 0),
        "long" => (0.0, 0.0, 0.5, 7),
        _ => return String::new(),
    };

    let color = if d.shadow_color == "accent" {
        d.acc.as_str()
    } else {
        "#000"
    };
    let outer = d.outer_grow();
    let mut all = String::new();
    for l in &glyph.layers {
        let g = silhouette(&l.pieces, color, outer);
        if !l.badge && open && d.surface != "shadow" {
            let m = format!(
                "{}{}",
                silhouette(&l.pieces, "#fff", outer),
                silhouette(&l.pieces, "#000", d.round)
            );
            all.push_str(&k.mask(&m, &g));
        } else {
            all.push_str(&g);
        }
    }

    let around = d.shadow_dir == "around" || d.shadow_color == "accent";
    let mut sbody = String::new();
    if long_steps > 0 {
        for i in 1..=long_steps {
            let off = (i as f64) * 0.42;
            sbody.push_str(&format!(
                "<g transform=\"translate({} {})\">{}</g>",
                f(off),
                f(off),
                frame(&all)
            ));
        }
    } else {
        let dx = if around { 0.0 } else { dist * 0.3 };
        let dy = if around { 0.0 } else { dist };
        sbody.push_str(&format!(
            "<g transform=\"translate({} {})\">{}</g>",
            f(dx),
            f(dy),
            frame(&all)
        ));
    }

    let blur_sd = if blur > 0.0 {
        blur * if d.shadow_color == "accent" { 1.4 } else { 1.0 }
    } else {
        0.0
    };
    let filter = if blur_sd > 0.0 {
        format!(" filter=\"url(#{})\"", k.blur(blur_sd))
    } else {
        String::new()
    };
    format!(
        "<g opacity=\"{}\"{}>{}</g>",
        f((alpha * d.shadow_strength).min(1.0)),
        filter,
        sbody
    )
}

fn render_ornament(glyph: &Glyph, d: &Design, k: &mut Kit) -> String {
    if d.ornament == "none" {
        return String::new();
    }
    if d.ornaments_on != "all" && glyph.tip.is_none() {
        return String::new();
    }
    let layer = &glyph.layers[0];
    let b = layer.box_;
    let pg = if layer.pieces.len() == 1 {
        polygon_of(&layer.pieces[0])
    } else {
        None
    };
    let tip = glyph.tip.unwrap_or([b[0], b[1]]);
    let tail = tail_of(glyph, layer);
    let ax = tail[0] - tip[0];
    let ay = tail[1] - tip[1];
    let al = ax.hypot(ay).max(1.0);

    let (corners, sides, inner) = if let Some(ref poly) = pg {
        let n = poly.len();
        let mut corners = Vec::new();
        for i in 0..n {
            let p = poly[i];
            let a = poly[(i + n - 1) % n];
            let c = poly[(i + 1) % n];
            let v1 = [a[0] - p[0], a[1] - p[1]];
            let v2 = [c[0] - p[0], c[1] - p[1]];
            let cos = (v1[0] * v2[0] + v1[1] * v2[1])
                / (v1[0].hypot(v1[1]).max(1.0) * v2[0].hypot(v2[1]).max(1.0));
            if cos > -0.75 && (p[0] - tip[0]).hypot(p[1] - tip[1]) > 2.0 {
                corners.push(p);
            }
        }
        if corners.is_empty() {
            corners.push(tail);
        }
        corners.truncate(6);

        let mut edges: Vec<(Pt, Pt, f64)> = (0..n)
            .map(|i| {
                let a = poly[i];
                let c = poly[(i + 1) % n];
                (a, c, (c[0] - a[0]).hypot(c[1] - a[1]))
            })
            .collect();
        edges.sort_by(|x, y| y.2.partial_cmp(&x.2).unwrap_or(std::cmp::Ordering::Equal));
        let sgn = if area(poly) > 0.0 { 1.0 } else { -1.0 };
        let sides: Vec<Side> = edges
            .into_iter()
            .take(2)
            .map(|(a, c, len)| {
                let l = len.max(1.0);
                let tx = (c[0] - a[0]) / l;
                let ty = (c[1] - a[1]) / l;
                Side {
                    x: (a[0] + c[0]) * 0.5,
                    y: (a[1] + c[1]) * 0.5,
                    tx,
                    ty,
                    nx: sgn * ty,
                    ny: -sgn * tx,
                }
            })
            .collect();
        (corners, sides, Some(offset(poly, d.round - 1.1)))
    } else {
        (
            vec![[b[2], b[1]], [b[2], b[3]], [b[0], b[3]]],
            vec![
                Side {
                    x: (b[0] + b[2]) * 0.5,
                    y: b[1],
                    tx: 1.0,
                    ty: 0.0,
                    nx: 0.0,
                    ny: -1.0,
                },
                Side {
                    x: (b[0] + b[2]) * 0.5,
                    y: b[3],
                    tx: 1.0,
                    ty: 0.0,
                    nx: 0.0,
                    ny: 1.0,
                },
            ],
            None,
        )
    };

    let mask_sil = silhouette(&layer.pieces, "#fff", d.round);
    let mut ctx = OrnamentCtx {
        k,
        corners,
        sides,
        tip,
        tail,
        axis: [ax / al, ay / al],
        box_: b,
        cx: (b[0] + b[2]) * 0.5,
        cy: (b[1] + b[3]) * 0.5,
        s: d.ornament_size,
        c: &d.c2,
        inner,
        mask_sil,
    };
    draw_ornament(&d.ornament, &mut ctx)
}

fn render_cap(g: &Glyph, d: &Design) -> String {
    let Some(tip) = g.tip else {
        return String::new();
    };
    if d.cap == "none" {
        return String::new();
    }
    let tail = tail_of(g, &g.layers[0]);
    let u = g.inward;
    let mut out = String::new();
    if d.cap == "tip" || d.cap == "both" {
        out.push_str(&format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"1.25\" fill=\"{}\"/>",
            f(tip[0] + u[0] * 1.6),
            f(tip[1] + u[1] * 1.6),
            d.acc
        ));
    }
    if d.cap == "tail" || d.cap == "both" {
        out.push_str(&format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"1.35\" fill=\"{}\"/>",
            f(tail[0]),
            f(tail[1]),
            d.acc
        ));
    }
    out
}

fn render_detail(g: &Glyph, d: &Design) -> String {
    let Some(t) = g.tip else { return String::new() };
    let u = g.inward;
    let at = |k: f64| [t[0] + u[0] * k, t[1] + u[1] * k];
    match d.detail.as_str() {
        "tip" => {
            let p = at(3.1);
            format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"1.3\" fill=\"{}\"/>",
                f(p[0]),
                f(p[1]),
                d.c2
            )
        }
        "spine" => {
            let a = at(3.4);
            let b = at(11.0);
            format!(
                "<path d=\"M{} {} L{} {}\" stroke=\"{}\" stroke-width=\".8\" stroke-linecap=\"round\" opacity=\".7\"/>",
                f(a[0]),
                f(a[1]),
                f(b[0]),
                f(b[1]),
                d.ink
            )
        }
        "spark" => {
            let c = at(7.2);
            let s = 2.3;
            let k = 0.55;
            let pts: Vec<Pt> = [
                [0.0, -s],
                [k, -k],
                [s, 0.0],
                [k, k],
                [0.0, s],
                [-k, k],
                [-s, 0.0],
                [-k, -k],
            ]
            .iter()
            .map(|q| [c[0] + q[0], c[1] + q[1]])
            .collect();
            format!(
                "<path d=\"{}\" fill=\"#fff\" opacity=\".9\"/>",
                poly_d(&pts)
            )
        }
        _ => String::new(),
    }
}

/// Hotspot normalized 0..1 across 32x32 cursor canvas.
pub fn hotspot_norm(glyph: &Glyph, d: &Design) -> (f64, f64) {
    let mut h = glyph.hot;
    if glyph.tip.is_some() {
        let g = d.outer_grow() * 0.75;
        h = [h[0] - glyph.inward[0] * g, h[1] - glyph.inward[1] * g];
    }
    ((h[0] / SIZE).clamp(0.0, 1.0), (h[1] / SIZE).clamp(0.0, 1.0))
}
