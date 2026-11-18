//! All 30 cursor surfaces — 1-to-1 port of `parts/Surfaces.js`.

use super::geo::{Pt, Rng, f};
use super::svg::{Kit, dark, light};
use std::f64::consts::PI;

pub const SURFACE_KEYS: &[(&str, &str)] = &[
    ("filled", "Solid"),
    ("faint", "Faint"),
    ("twotone", "Two-tone"),
    ("split", "Split"),
    ("frost", "Frost"),
    ("bevel", "Bevel"),
    ("keycap", "Keycap"),
    ("metal", "Metal"),
    ("carbon", "Carbon"),
    ("crt", "Old screen"),
    ("candy", "Candy"),
    ("halftone", "Halftone"),
    ("holo", "Hologram"),
    ("comet", "Comet"),
    ("damask", "Damask"),
    ("velvet", "Velvet"),
    ("marble", "Marble"),
    ("ledge", "Ledge"),
    ("band", "Band"),
    ("underglow", "Underglow"),
    ("spiderweb", "Spiderweb"),
    ("wood", "Wood"),
    ("terrazzo", "Terrazzo"),
    ("crackle", "Crackle"),
    ("topo", "Contours"),
    ("nebula", "Nebula"),
    ("leather", "Leather"),
    ("stained", "Stained glass"),
    ("shadow", "Only a shadow"),
    ("clear", "Clear"),
];

pub fn is_open(key: &str) -> bool {
    matches!(key, "faint" | "comet" | "shadow" | "clear")
}

pub fn no_shadow(key: &str) -> bool {
    key == "clear"
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

fn poly_svg(pts: &[Pt], fill: &str, extra: &str) -> String {
    let mut d = String::from("M");
    for (i, p) in pts.iter().enumerate() {
        if i > 0 {
            d.push_str(" L");
        }
        d.push_str(&format!("{} {}", f(p[0]), f(p[1])));
    }
    d.push_str(" Z");
    format!("<path d=\"{}\" fill=\"{}\"{}/>", d, fill, extra)
}

fn line_svg(pts: &[Pt], stroke: &str, w: f64, extra: &str) -> String {
    let mut d = String::from("M");
    for (i, p) in pts.iter().enumerate() {
        if i > 0 {
            d.push_str(" L");
        }
        d.push_str(&format!("{} {}", f(p[0]), f(p[1])));
    }
    format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"{}/>",
        d,
        stroke,
        f(w),
        extra
    )
}

fn rect_full(paint: &str, extra: &str) -> String {
    format!(
        "<rect x=\"-4\" y=\"-4\" width=\"40\" height=\"40\" fill=\"{}\"{}/>",
        paint, extra
    )
}

pub struct SurfaceCtx<'a> {
    pub k: &'a mut Kit,
    pub c1: &'a str,
    pub c2: &'a str,
    pub acc: &'a str,
    pub ink: &'a str,
    pub box_: [f64; 4],
    pub cx: f64,
    pub cy: f64,
    pub w: f64,
    pub h: f64,
    pub tip: Pt,
    pub tail: Pt,
    pub rnd: &'a mut Rng,
    pub base_fill_svg: &'a str,
    pub sil_fn: &'a dyn Fn(&str, f64) -> String,
    pub clip_mask_sil: &'a str,
    pub lower: &'a mut String,
}

impl<'a> SurfaceCtx<'a> {
    fn clip(&mut self, inner: &str) -> String {
        self.k.mask(self.clip_mask_sil, inner)
    }

    fn gloss(&mut self, a: f64) -> String {
        let g = self.k.lin(
            0.0,
            self.box_[1],
            0.0,
            self.cy,
            &[(0.0, "#fff", a), (1.0, "#fff", 0.0)],
        );
        self.clip(&rect_full(&format!("url(#{})", g), ""))
    }
}

pub fn paint_surface(key: &str, s: &mut SurfaceCtx) -> String {
    match key {
        "faint" => format!("<g opacity=\".38\">{}</g>", s.base_fill_svg),
        "twotone" => {
            let r = format!(
                "<rect x=\"{}\" y=\"-4\" width=\"40\" height=\"40\" fill=\"{}\"/>",
                f(s.cx),
                s.c2
            );
            let cl = s.clip(&r);
            format!("{}{}", s.base_fill_svg, cl)
        }
        "split" => {
            let ax = s.tail[0] - s.tip[0];
            let ay = s.tail[1] - s.tip[1];
            let mx = s.tip[0] + ax * 0.62;
            let my = s.tip[1] + ay * 0.62;
            let px = -ay;
            let py = ax;
            let p = poly_svg(
                &[
                    [mx + px * 9.0, my + py * 9.0],
                    [mx - px * 9.0, my - py * 9.0],
                    [mx - px * 9.0 + ax * 9.0, my - py * 9.0 + ay * 9.0],
                    [mx + px * 9.0 + ax * 9.0, my + py * 9.0 + ay * 9.0],
                ],
                s.c2,
                "",
            );
            let cl = s.clip(&p);
            format!("{}{}", s.base_fill_svg, cl)
        }
        "frost" => {
            let mut dots = String::new();
            for _ in 0..40 {
                let x = s.box_[0] + s.rnd.next() * s.w;
                let y = s.box_[1] + s.rnd.next() * s.h;
                let r = 0.12 + s.rnd.next() * 0.22;
                let op = format!(" opacity=\"{}\"", f(0.25 + s.rnd.next() * 0.4));
                dots.push_str(&circle_svg(x, y, r, "#fff", &op));
            }
            let inner = format!("{}{}", rect_full("#fff", " opacity=\".18\""), dots);
            let cl = s.clip(&inner);
            let gl = s.gloss(0.3);
            format!("<g opacity=\".8\">{}</g>{}{}", s.base_fill_svg, cl, gl)
        }
        "bevel" => {
            let g = s.k.lin(
                s.box_[0],
                s.box_[1],
                s.box_[2],
                s.box_[3],
                &[
                    (0.0, "#fff", 0.5),
                    (0.45, "#fff", 0.0),
                    (0.55, "#000", 0.0),
                    (1.0, "#000", 0.38),
                ],
            );
            let cl = s.clip(&rect_full(&format!("url(#{})", g), ""));
            format!("{}{}", s.base_fill_svg, cl)
        }
        "keycap" => {
            let dk = dark(s.c1, 0.4);
            s.lower.push_str(&format!(
                "<g transform=\"translate(0 1.1)\">{}</g>",
                (s.sil_fn)(&dk, 0.0)
            ));
            let gl = s.gloss(0.28);
            format!("{}{}", s.base_fill_svg, gl)
        }
        "metal" => {
            let l55 = light(s.c1, 0.55);
            let d30 = dark(s.c1, 0.3);
            let l30 = light(s.c1, 0.3);
            let d25 = dark(s.c1, 0.25);
            let g = s.k.lin(
                0.0,
                s.box_[1],
                0.0,
                s.box_[3],
                &[
                    (0.0, &l55, 1.0),
                    (0.35, s.c1, 1.0),
                    (0.55, &d30, 1.0),
                    (0.75, &l30, 1.0),
                    (1.0, &d25, 1.0),
                ],
            );
            let p = s.k.pat(
                4.0,
                0.7,
                "<rect width=\"4\" height=\"0.25\" fill=\"#fff\" opacity=\".18\"/>",
                0.0,
            );
            let base = (s.sil_fn)(&format!("url(#{})", g), 0.0);
            let cl = s.clip(&rect_full(&format!("url(#{})", p), ""));
            format!("{}{}", base, cl)
        }
        "carbon" => {
            let a = dark(s.c1, 0.6);
            let b = dark(s.c1, 0.35);
            let body = format!(
                "<rect width=\"1\" height=\"1\" fill=\"{b}\"/><rect x=\"1\" y=\"1\" width=\"1\" height=\"1\" fill=\"{b}\"/><rect width=\"1\" height=\".45\" fill=\"#fff\" opacity=\".12\"/><rect x=\"1\" y=\"1\" width=\".45\" height=\"1\" fill=\"#fff\" opacity=\".12\"/>"
            );
            let p = s.k.pat(2.0, 2.0, &body, 45.0);
            let base = (s.sil_fn)(&a, 0.0);
            let cl = s.clip(&rect_full(&format!("url(#{})", p), ""));
            let gl = s.gloss(0.18);
            format!("{}{}{}", base, cl, gl)
        }
        "crt" => {
            let p = s.k.pat(
                1.0,
                1.1,
                "<rect width=\"1\" height=\".45\" fill=\"#000\" opacity=\".38\"/>",
                0.0,
            );
            let v = s.k.rad(
                s.cx,
                s.cy,
                s.w.max(s.h) * 0.7,
                &[(0.5, "#000", 0.0), (1.0, "#000", 0.4)],
            );
            let inner = format!(
                "{}{}",
                rect_full(&format!("url(#{})", p), ""),
                rect_full(&format!("url(#{})", v), "")
            );
            let cl = s.clip(&inner);
            format!("{}{}", s.base_fill_svg, cl)
        }
        "candy" => {
            let p = s.k.pat(
                3.2,
                3.2,
                &format!("<rect width=\"1.6\" height=\"3.2\" fill=\"{}\"/>", s.c2),
                45.0,
            );
            let cl = s.clip(&rect_full(&format!("url(#{})", p), ""));
            let gl = s.gloss(0.4);
            format!("{}{}{}", s.base_fill_svg, cl, gl)
        }
        "halftone" => {
            let mut dots = String::new();
            let ax = s.tail[0] - s.tip[0];
            let ay = s.tail[1] - s.tip[1];
            let len2 = (ax * ax + ay * ay).max(1.0);
            let mut y = s.box_[1];
            let mut row = 0usize;
            while y <= s.box_[3] + 1.0 {
                let mut x = s.box_[0] + if row % 2 == 1 { 0.7 } else { 0.0 };
                while x <= s.box_[2] + 1.0 {
                    let t = (((x - s.tip[0]) * ax + (y - s.tip[1]) * ay) / len2).clamp(0.0, 1.0);
                    dots.push_str(&circle_svg(x, y, 0.1 + 0.55 * t, s.c2, ""));
                    x += 1.4;
                }
                y += 1.4;
                row += 1;
            }
            let cl = s.clip(&dots);
            format!("{}{}", s.base_fill_svg, cl)
        }
        "holo" => {
            let g = s.k.lin(
                s.box_[0],
                s.box_[1],
                s.box_[2],
                s.box_[3],
                &[
                    (0.0, "#ff5f9e", 0.6),
                    (0.2, "#ffd36b", 0.6),
                    (0.4, "#7dffb0", 0.6),
                    (0.6, "#6bd5ff", 0.6),
                    (0.8, "#a78bff", 0.6),
                    (1.0, "#ff5f9e", 0.6),
                ],
            );
            let cl = s.clip(&rect_full(&format!("url(#{})", g), ""));
            let gl = s.gloss(0.35);
            format!("{}{}{}", s.base_fill_svg, cl, gl)
        }
        "comet" => {
            let l3 = light(s.c1, 0.3);
            let g = s.k.lin(
                s.tail[0],
                s.tail[1],
                s.tip[0],
                s.tip[1],
                &[(0.0, s.c1, 0.0), (0.75, s.c1, 1.0), (1.0, &l3, 1.0)],
            );
            (s.sil_fn)(&format!("url(#{})", g), 0.0)
        }
        "damask" => {
            let m = format!(
                "<path d=\"M1.6 .2 Q2.3 1 1.6 1.6 Q.9 1 1.6 .2 M.2 1.6 Q1 2.3 1.6 1.6 Q1 .9 .2 1.6 M3 1.6 Q2.2 2.3 1.6 1.6 Q2.2 .9 3 1.6 M1.6 3 Q.9 2.2 1.6 1.6 Q2.3 2.2 1.6 3\" fill=\"{}\" opacity=\".55\"/>",
                s.c2
            );
            let p = s.k.pat(3.2, 3.2, &m, 0.0);
            let cl = s.clip(&rect_full(&format!("url(#{})", p), ""));
            format!("{}{}", s.base_fill_svg, cl)
        }
        "velvet" => {
            let l2 = light(s.c1, 0.2);
            let d5 = dark(s.c1, 0.5);
            let g = s.k.rad(
                s.cx - s.w * 0.15,
                s.cy - s.h * 0.15,
                s.w.max(s.h) * 0.8,
                &[(0.0, &l2, 1.0), (0.5, s.c1, 1.0), (1.0, &d5, 1.0)],
            );
            let sh = s.k.lin(
                s.box_[0],
                s.box_[1],
                s.box_[2],
                s.box_[3],
                &[
                    (0.3, "#fff", 0.0),
                    (0.42, "#fff", 0.16),
                    (0.54, "#fff", 0.0),
                ],
            );
            let base = (s.sil_fn)(&format!("url(#{})", g), 0.0);
            let cl = s.clip(&rect_full(&format!("url(#{})", sh), ""));
            format!("{}{}", base, cl)
        }
        "marble" => {
            let mut veins = String::new();
            let dk = dark(s.c2, 0.2);
            for v in 0..4 {
                let mut pts = Vec::with_capacity(7);
                let mut x = s.box_[0] + s.rnd.next() * s.w * 0.3;
                let mut y = s.box_[1] + s.rnd.next() * s.h;
                for _ in 0..7 {
                    pts.push([x, y]);
                    x += s.w * 0.18;
                    y += (s.rnd.next() - 0.5) * s.h * 0.3;
                }
                let col = if v % 2 == 1 { dk.as_str() } else { s.c2 };
                let w = if v % 2 == 1 { 0.18 } else { 0.35 };
                veins.push_str(&line_svg(&pts, col, w, " opacity=\".7\""));
            }
            let base = (s.sil_fn)(&light(s.c1, 0.4), 0.0);
            let cl = s.clip(&veins);
            format!("{}{}", base, cl)
        }
        "ledge" => {
            let dk = dark(s.c1, 0.35);
            let r = format!(
                "<rect x=\"-4\" y=\"{}\" width=\"40\" height=\"40\" fill=\"{}\"/>",
                f(s.box_[3] - s.h * 0.3),
                dk
            );
            let cl = s.clip(&r);
            format!("{}{}", s.base_fill_svg, cl)
        }
        "band" => {
            let g = s.k.lin(
                s.box_[0],
                s.box_[3],
                s.box_[2],
                s.box_[1],
                &[(0.3, s.c2, 0.0), (0.5, s.c2, 0.85), (0.7, s.c2, 0.0)],
            );
            let cl = s.clip(&rect_full(&format!("url(#{})", g), ""));
            format!("{}{}", s.base_fill_svg, cl)
        }
        "underglow" => {
            let blur = s.k.blur(1.4);
            s.lower.push_str(&format!(
                "<g opacity=\".95\" filter=\"url(#{})\" transform=\"translate(0 1.2)\">{}</g>",
                blur,
                (s.sil_fn)(s.c2, 0.6)
            ));
            s.base_fill_svg.to_string()
        }
        "spiderweb" => {
            let mut web = String::new();
            let r_max = s.w.hypot(s.h);
            let base = (s.tail[1] - s.tip[1]).atan2(s.tail[0] - s.tip[0]);
            let spokes: Vec<f64> = (-2..=2).map(|i| base + (i as f64) * 0.32).collect();
            for &a in &spokes {
                web.push_str(&line_svg(
                    &[
                        s.tip,
                        [s.tip[0] + a.cos() * r_max, s.tip[1] + a.sin() * r_max],
                    ],
                    s.ink,
                    0.18,
                    " opacity=\".45\"",
                ));
            }
            for k in 1..=4 {
                let r = (k as f64) * r_max / 5.0;
                let pts: Vec<Pt> = spokes
                    .iter()
                    .map(|&a| [s.tip[0] + a.cos() * r, s.tip[1] + a.sin() * r])
                    .collect();
                web.push_str(&line_svg(&pts, s.ink, 0.16, " opacity=\".4\""));
            }
            let cl = s.clip(&web);
            format!("{}{}", s.base_fill_svg, cl)
        }
        "wood" => {
            let mut rings = String::new();
            let dk = dark(s.c1, 0.35);
            let kx = s.cx + s.w * 0.15;
            let ky = s.cy + s.h * 0.1;
            for k in 1..=9 {
                let mut pts = Vec::with_capacity(25);
                for i in 0..=24 {
                    let a = 2.0 * PI * (i as f64) / 24.0;
                    let r = (k as f64) * 0.9 * (1.0 + 0.12 * (a * 3.0 + k as f64).sin());
                    pts.push([kx + a.cos() * r * 1.6, ky + a.sin() * r * 0.8]);
                }
                rings.push_str(&line_svg(&pts, &dk, 0.2, " opacity=\".55\""));
            }
            let cl = s.clip(&rings);
            format!("{}{}", s.base_fill_svg, cl)
        }
        "terrazzo" => {
            let mut chips = String::new();
            let l5 = light(s.c1, 0.5);
            let d4 = dark(s.c1, 0.4);
            let cols = [s.c2, s.acc, l5.as_str(), d4.as_str()];
            for i in 0..26 {
                let x = s.box_[0] + s.rnd.next() * s.w;
                let y = s.box_[1] + s.rnd.next() * s.h;
                let r = 0.35 + s.rnd.next() * 0.55;
                let n = 3 + (s.rnd.next() * 3.0).floor() as usize;
                let mut pts = Vec::with_capacity(n);
                for j in 0..n {
                    let a = 2.0 * PI * (j as f64) / (n as f64) + s.rnd.next();
                    pts.push([x + a.cos() * r, y + a.sin() * r]);
                }
                chips.push_str(&poly_svg(&pts, cols[i % cols.len()], ""));
            }
            let cl = s.clip(&chips);
            format!("{}{}", s.base_fill_svg, cl)
        }
        "crackle" => {
            let mut cracks = String::new();
            let dk = dark(s.c1, 0.55);
            for _ in 0..6 {
                let mut pts = Vec::with_capacity(6);
                let mut x = s.box_[0] + s.rnd.next() * s.w;
                let mut y = s.box_[1] + s.rnd.next() * s.h;
                let mut a = s.rnd.next() * std::f64::consts::TAU;
                for _ in 0..6 {
                    pts.push([x, y]);
                    a += (s.rnd.next() - 0.5) * 1.4;
                    x += a.cos() * 1.6;
                    y += a.sin() * 1.6;
                }
                cracks.push_str(&line_svg(&pts, &dk, 0.18, " opacity=\".7\""));
            }
            let gl = s.gloss(0.2);
            let cl = s.clip(&cracks);
            format!("{}{}{}", s.base_fill_svg, gl, cl)
        }
        "topo" => {
            let mut lines = String::new();
            for k in 1..=7 {
                let mut pts = Vec::with_capacity(29);
                for i in 0..=28 {
                    let a = 2.0 * PI * (i as f64) / 28.0;
                    let r = (k as f64)
                        * 1.15
                        * (1.0 + 0.18 * (a * 2.0 + 1.3).sin() + 0.08 * (a * 5.0).sin());
                    pts.push([s.cx + a.cos() * r, s.cy + a.sin() * r * 0.85]);
                }
                lines.push_str(&line_svg(&pts, s.ink, 0.17, " opacity=\".42\""));
            }
            let cl = s.clip(&lines);
            format!("{}{}", s.base_fill_svg, cl)
        }
        "nebula" => {
            let d6 = dark(s.c1, 0.6);
            let l3 = light(s.c1, 0.3);
            let g1 = s.k.rad(
                s.cx - s.w * 0.2,
                s.cy - s.h * 0.15,
                s.w * 0.6,
                &[(0.0, s.c2, 0.8), (1.0, s.c2, 0.0)],
            );
            let g2 = s.k.rad(
                s.cx + s.w * 0.2,
                s.cy + s.h * 0.2,
                s.w * 0.55,
                &[(0.0, s.acc, 0.6), (1.0, s.acc, 0.0)],
            );
            let g3 =
                s.k.rad(s.cx, s.cy, s.w * 0.35, &[(0.0, &l3, 0.5), (1.0, &l3, 0.0)]);
            let mut stars = String::new();
            for _ in 0..14 {
                let x = s.box_[0] + s.rnd.next() * s.w;
                let y = s.box_[1] + s.rnd.next() * s.h;
                let r = 0.08 + s.rnd.next() * 0.16;
                let op = format!(" opacity=\"{}\"", f(0.5 + s.rnd.next() * 0.5));
                stars.push_str(&circle_svg(x, y, r, "#fff", &op));
            }
            let inner = format!(
                "{}{}{}{}",
                circle_svg(
                    s.cx - s.w * 0.2,
                    s.cy - s.h * 0.15,
                    s.w * 0.6,
                    &format!("url(#{})", g1),
                    ""
                ),
                circle_svg(
                    s.cx + s.w * 0.2,
                    s.cy + s.h * 0.2,
                    s.w * 0.55,
                    &format!("url(#{})", g2),
                    ""
                ),
                circle_svg(s.cx, s.cy, s.w * 0.35, &format!("url(#{})", g3), ""),
                stars
            );
            let base = (s.sil_fn)(&d6, 0.0);
            let cl = s.clip(&inner);
            format!("{}{}", base, cl)
        }
        "leather" => {
            let mut grain = String::new();
            for i in 0..70 {
                let x = s.box_[0] + s.rnd.next() * s.w;
                let y = s.box_[1] + s.rnd.next() * s.h;
                let r = 0.25 + s.rnd.next() * 0.3;
                let (col, op) = if i % 2 == 1 {
                    ("#000", " opacity=\".16\"")
                } else {
                    ("#fff", " opacity=\".1\"")
                };
                grain.push_str(&circle_svg(x, y, r, col, op));
            }
            let cl = s.clip(&grain);
            let gl = s.gloss(0.12);
            format!("{}{}{}", s.base_fill_svg, cl, gl)
        }
        "stained" => {
            let l35 = light(s.c1, 0.35);
            let l30 = light(s.c2, 0.3);
            let cols = [s.c1, s.c2, s.acc, l35.as_str(), l30.as_str()];
            let step = 3.4;
            let mut grid: Vec<Vec<Pt>> = Vec::new();
            let mut y = s.box_[1] - step;
            while y <= s.box_[3] + step {
                let mut row = Vec::new();
                let mut x = s.box_[0] - step;
                while x <= s.box_[2] + step {
                    row.push([
                        x + (s.rnd.next() - 0.5) * 1.6,
                        y + (s.rnd.next() - 0.5) * 1.6,
                    ]);
                    x += step;
                }
                grid.push(row);
                y += step;
            }
            let lead = format!(
                " stroke=\"{}\" stroke-width=\".32\" stroke-linejoin=\"round\"",
                dark(s.c1, 0.75)
            );
            let mut panes = String::new();
            for j in 0..grid.len().saturating_sub(1) {
                for i in 0..grid[j].len().saturating_sub(1) {
                    let a = grid[j][i];
                    let b = grid[j][i + 1];
                    let c = grid[j + 1][i + 1];
                    let dd = grid[j + 1][i];
                    panes.push_str(&poly_svg(&[a, b, c], cols[(i + j) % cols.len()], &lead));
                    panes.push_str(&poly_svg(
                        &[a, c, dd],
                        cols[(i + j + 2) % cols.len()],
                        &lead,
                    ));
                }
            }
            let cl = s.clip(&panes);
            let gl = s.gloss(0.2);
            format!("{}{}{}", s.base_fill_svg, cl, gl)
        }
        "shadow" | "clear" => String::new(),
        _ => s.base_fill_svg.to_string(),
    }
}
