//! All 22 cursor lines — 1-to-1 port of `parts/Lines.js`.

use super::geo::{Piece, Pt, displaced, f, offset, poly_d, polygon_of, walk, wobble};
use super::svg::{Kit, dark, light};
use std::f64::consts::PI;

pub const LINE_KEYS: &[(&str, &str)] = &[
    ("none", "None"),
    ("solid", "Solid"),
    ("double", "Double"),
    ("triple", "Triple"),
    ("glow", "Glow ring"),
    ("neon", "Neon tube"),
    ("dashed", "Dashed"),
    ("dotted", "Dotted"),
    ("dashdot", "Dash and dot"),
    ("stitched", "Stitched"),
    ("inset", "Inset line"),
    ("mat", "Mat"),
    ("rail", "Rail"),
    ("wavy", "Wavy"),
    ("sawtooth", "Sawtooth"),
    ("rope", "Rope"),
    ("chain", "Chain"),
    ("beads", "Beads"),
    ("ladder", "Ladder"),
    ("sketch", "Sketched"),
    ("gilded", "Gilded frame"),
    ("glitch", "Glitch"),
];

pub fn line_reach(key: &str, w: f64) -> f64 {
    match key {
        "none" | "inset" => 0.0,
        "double" => w + 0.6,
        "triple" => (w * 0.4).max(0.3) * 3.0 + 0.9,
        "glow" => w + 1.0,
        "neon" => w + 1.2,
        "mat" => w * 0.5,
        "rail" => w * 1.5,
        "wavy" | "sawtooth" => w + 0.5,
        "rope" | "beads" => w + 0.3,
        "chain" | "sketch" => w + 0.4,
        "ladder" => w + 0.8,
        "gilded" => w + 0.7,
        "glitch" => w + 0.6,
        _ => w,
    }
}

pub fn is_traced(key: &str) -> bool {
    matches!(
        key,
        "dashed"
            | "dotted"
            | "dashdot"
            | "stitched"
            | "inset"
            | "wavy"
            | "sawtooth"
            | "rope"
            | "chain"
            | "beads"
            | "ladder"
            | "sketch"
    )
}

fn circle_svg(x: f64, y: f64, r: f64, fill: &str, extra: &str) -> String {
    format!(
        "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"{}/>",
        f(x),
        f(y),
        f(r),
        fill,
        extra
    )
}

fn stroke_poly(pg: &[Pt], paint: &str, w: f64, extra: &str) -> String {
    format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linejoin=\"round\"{}/>",
        poly_d(pg),
        paint,
        f(w),
        extra
    )
}

pub struct LineCtx<'a> {
    pub k: &'a mut Kit,
    pub c: &'a str,
    pub paint: &'a str,
    pub w: f64,
    pub r: f64,
    pub open: bool,
    pub box_: [f64; 4],
    pub cx: f64,
    pub cy: f64,
    pub pieces: &'a [Piece],
    pub sil_fn: &'a dyn Fn(&str, f64) -> String,
    pub line_under: &'a mut String,
    pub line_over: &'a mut String,
}

impl<'a> LineCtx<'a> {
    fn inside(&mut self, inner: &str) -> String {
        let m = (self.sil_fn)("#fff", 0.0);
        self.k.mask(&m, inner)
    }
    fn outside(&mut self, inner: &str) -> String {
        let m = format!(
            "<rect x=\"-4\" y=\"-4\" width=\"40\" height=\"40\" fill=\"#fff\"/>{}",
            (self.sil_fn)("#000", -0.02)
        );
        self.k.mask(&m, inner)
    }
    fn band(&mut self, from: f64, to: f64, paint: &str) -> String {
        let m = format!(
            "{}{}",
            (self.sil_fn)("#fff", to),
            (self.sil_fn)("#000", from)
        );
        let inner = (self.sil_fn)(paint, to);
        self.k.mask(&m, &inner)
    }
    fn ring(&mut self, from: f64, to: f64, paint: &str) {
        if from <= 0.0 && !self.open {
            if from < 0.0 {
                let b = self.band(from, to, paint);
                self.line_under.push_str(&b);
            } else {
                self.line_under.push_str(&(self.sil_fn)(paint, to));
            }
        } else {
            let b = self.band(from, to, paint);
            self.line_over.push_str(&b);
        }
    }
    fn paths(&self, off: f64) -> Vec<Vec<Pt>> {
        self.pieces
            .iter()
            .filter_map(polygon_of)
            .map(|pg| offset(&pg, self.r + off))
            .collect()
    }
}

fn draw_dashed(l: &mut LineCtx, dash: &str, round_cap: bool) {
    let cap = if round_cap {
        " stroke-linecap=\"round\""
    } else {
        " stroke-linecap=\"butt\""
    };
    let extra = format!(" stroke-dasharray=\"{}\"{}", dash, cap);
    for p in l.paths(l.w * 0.5) {
        let st = stroke_poly(&p, l.paint, l.w, &extra);
        let out = l.outside(&st);
        l.line_over.push_str(&out);
    }
}

pub fn draw_line(key: &str, l: &mut LineCtx) {
    let paint = l.paint.to_string();
    let c = l.c.to_string();
    let w = l.w;
    match key {
        "none" => {}
        "solid" => l.ring(0.0, w, &paint),
        "double" => {
            l.ring(0.0, w * 0.55, &paint);
            l.ring(w * 0.55 + 0.45, w + 0.6, &paint);
        }
        "triple" => {
            let t = (w * 0.4).max(0.3);
            for i in 0..3 {
                let s = (i as f64) * (t + 0.45);
                l.ring(s, s + t, &paint);
            }
        }
        "glow" => {
            let blur = l.k.blur(0.9);
            let b = l.band(0.0, w + 1.2, &paint);
            let out = l.outside(&b);
            l.line_over.push_str(&format!(
                "<g opacity=\".9\" filter=\"url(#{})\">{}</g>",
                blur, out
            ));
            l.ring(0.0, w * 0.6, &paint);
        }
        "neon" => {
            let blur = l.k.blur(1.1);
            let b = l.band(0.0, w + 1.4, &paint);
            let out = l.outside(&b);
            l.line_over
                .push_str(&format!("<g filter=\"url(#{})\">{}</g>", blur, out));
            l.ring(0.0, w, &paint);
            l.ring(w * 0.3, w * 0.72, &light(&c, 0.75));
        }
        "dashed" => draw_dashed(l, "1.8 1.1", false),
        "dotted" => draw_dashed(l, &format!("0.01 {}", f(w + 0.8)), true),
        "dashdot" => draw_dashed(
            l,
            &format!("1.8 {} 0.01 {}", f(w * 0.5 + 0.7), f(w * 0.5 + 0.7)),
            true,
        ),
        "stitched" => {
            l.ring(0.0, (w * 0.5).min(0.5), &paint);
            let col = light(&c, 0.55);
            for p in l.paths(-0.95) {
                let st = stroke_poly(&p, &col, 0.3, " stroke-dasharray=\"0.9 0.7\"");
                let ins = l.inside(&st);
                l.line_over.push_str(&ins);
            }
        }
        "inset" => {
            for p in l.paths(-0.75 - w * 0.35) {
                let st = stroke_poly(&p, &paint, w * 0.7, "");
                let ins = l.inside(&st);
                l.line_over.push_str(&ins);
            }
        }
        "mat" => {
            l.ring(-1.3, 0.0, &light(&c, 0.75));
            l.ring(0.0, w * 0.5, &paint);
        }
        "rail" => {
            l.ring(0.0, (w * 0.35).min(0.4), &paint);
            let id = l.k.id("m");
            l.k.def(&format!(
                "<mask id=\"{}\" maskUnits=\"userSpaceOnUse\" x=\"-4\" y=\"-4\" width=\"40\" height=\"40\"><path d=\"M{} {} L{} {} L{} {} Z\" fill=\"#fff\"/></mask>",
                id, f(l.cx + 30.0), f(l.cy - 30.0), f(l.cx - 30.0), f(l.cy + 30.0), f(l.cx + 30.0), f(l.cy + 30.0)
            ));
            let b = l.band(0.0, w * 1.5, &paint);
            let out = l.outside(&b);
            l.line_over
                .push_str(&format!("<g mask=\"url(#{})\">{}</g>", id, out));
        }
        "wavy" => {
            for p in l.paths(w * 0.5 + 0.2) {
                let dp = displaced(&p, 0.25, |s| 0.42 * (s * 2.6).sin());
                let st = stroke_poly(&dp, &paint, w * 0.75, "");
                let out = l.outside(&st);
                l.line_over.push_str(&out);
            }
        }
        "sawtooth" => {
            for p in l.paths(w * 0.5 + 0.2) {
                let dp = displaced(&p, 0.2, |s| {
                    0.5 * (2.0 * ((s / 1.5).rem_euclid(1.0) - 0.5).abs() * 2.0 - 1.0)
                });
                let st = stroke_poly(&dp, &paint, w * 0.7, "");
                let out = l.outside(&st);
                l.line_over.push_str(&out);
            }
        }
        "rope" => {
            l.ring(0.0, w + 0.3, &dark(&c, 0.3));
            let lc = light(&c, 0.4);
            for p in l.paths((w + 0.3) * 0.5) {
                let mut twists = String::new();
                for q in walk(&p, 0.8) {
                    let a = q.ty.atan2(q.tx) + 0.9;
                    let h = (w + 0.3) * 0.62;
                    twists.push_str(&format!(
                        "<path d=\"M{} {} L{} {}\" stroke=\"{}\" stroke-width=\".28\" stroke-linecap=\"round\"/>",
                        f(q.x - a.cos() * h), f(q.y - a.sin() * h), f(q.x + a.cos() * h), f(q.y + a.sin() * h), lc
                    ));
                }
                let out = l.outside(&twists);
                l.line_over.push_str(&out);
            }
        }
        "chain" => {
            for p in l.paths(w * 0.5 + 0.2) {
                let mut links = String::new();
                for (n, q) in walk(&p, 1.15).into_iter().enumerate() {
                    let a = q.ty.atan2(q.tx) * 180.0 / PI;
                    let flat = n % 2 == 0;
                    links.push_str(&format!(
                        "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" transform=\"rotate({} {} {})\" fill=\"none\" stroke=\"{}\" stroke-width=\".26\"/>",
                        f(q.x), f(q.y), f(if flat { 0.75 } else { 0.5 }), f(if flat { 0.38 } else { 0.12 }),
                        f(a), f(q.x), f(q.y), c
                    ));
                }
                let out = l.outside(&links);
                l.line_over.push_str(&out);
            }
        }
        "beads" => {
            let r = (w * 0.5 + 0.1).max(0.35);
            for p in l.paths(r) {
                let mut beads = String::new();
                for q in walk(&p, r * 2.2) {
                    beads.push_str(&circle_svg(q.x, q.y, r, &c, ""));
                    beads.push_str(&circle_svg(
                        q.x - r * 0.3,
                        q.y - r * 0.3,
                        r * 0.35,
                        "#fff",
                        " opacity=\".55\"",
                    ));
                }
                let out = l.outside(&beads);
                l.line_over.push_str(&out);
            }
        }
        "ladder" => {
            let ww = w + 0.8;
            l.ring(0.0, 0.28, &paint);
            l.ring(ww - 0.28, ww, &paint);
            for p in l.paths(ww * 0.5) {
                let mut rungs = String::new();
                for q in walk(&p, 0.85) {
                    rungs.push_str(&format!(
                        "<path d=\"M{} {} L{} {}\" stroke=\"{}\" stroke-width=\".22\"/>",
                        f(q.x - q.nx * ww * 0.5),
                        f(q.y - q.ny * ww * 0.5),
                        f(q.x + q.nx * ww * 0.5),
                        f(q.y + q.ny * ww * 0.5),
                        c
                    ));
                }
                let out = l.outside(&rungs);
                l.line_over.push_str(&out);
            }
        }
        "sketch" => {
            for p in l.paths(w * 0.5) {
                let wa = wobble(1.3);
                let wb = wobble(4.7);
                let s1 = stroke_poly(
                    &displaced(&p, 0.3, |s| wa(s) * 0.35),
                    &paint,
                    w * 0.55,
                    " opacity=\".9\"",
                );
                let s2 = stroke_poly(
                    &displaced(&p, 0.3, |s| wb(s) * 0.4 + 0.15),
                    &paint,
                    w * 0.4,
                    " opacity=\".7\"",
                );
                let out = l.outside(&format!("{}{}", s1, s2));
                l.line_over.push_str(&out);
            }
        }
        "gilded" => {
            let l6 = light(&c, 0.6);
            let d4 = dark(&c, 0.4);
            let l5 = light(&c, 0.5);
            let d55 = dark(&c, 0.55);
            let g = l.k.lin(
                l.box_[0],
                l.box_[1],
                l.box_[2],
                l.box_[3],
                &[
                    (0.0, &l6, 1.0),
                    (0.3, &c, 1.0),
                    (0.55, &d4, 1.0),
                    (0.8, &l5, 1.0),
                    (1.0, &c, 1.0),
                ],
            );
            l.ring(0.0, w + 0.7, &format!("url(#{})", g));
            l.ring(0.0, 0.22, &d55);
            l.ring(w + 0.48, w + 0.7, &d55);
        }
        "glitch" => {
            let b1 = l.band(0.0, w, "#ff2fa0");
            let o1 = l.outside(&b1);
            let b2 = l.band(0.0, w, "#22e6ff");
            let o2 = l.outside(&b2);
            l.line_over.push_str(&format!(
                "<g opacity=\".8\" transform=\"translate(-.55 .1)\">{}</g>",
                o1
            ));
            l.line_over.push_str(&format!(
                "<g opacity=\".8\" transform=\"translate(.55 -.1)\">{}</g>",
                o2
            ));
            l.ring(0.0, w * 0.6, &paint);
        }
        _ => l.ring(0.0, w, &paint),
    }
}
