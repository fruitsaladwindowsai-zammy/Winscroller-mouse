//! All 18 cursor ornaments — 1-to-1 port of `parts/Ornaments.js`.

use super::geo::{Pt, f, poly_d};
use super::svg::{Kit, dark, light};
use std::f64::consts::PI;

pub const ORNAMENT_KEYS: &[(&str, &str)] = &[
    ("none", "None"),
    ("studs", "Studs"),
    ("rivets", "Rivets"),
    ("pearls", "Pearls"),
    ("crosses", "Crosses"),
    ("stars", "Stars"),
    ("brackets", "Brackets"),
    ("rosettes", "Rosettes"),
    ("diamonds", "Diamonds"),
    ("fleurons", "Fleurons"),
    ("filigree", "Filigree"),
    ("flourish", "Flourish"),
    ("scrolls", "Scrolls"),
    ("chevrons", "Chevrons"),
    ("fans", "Fans"),
    ("deco", "Deco"),
    ("crown", "Crown"),
    ("laurels", "Laurels"),
];

#[derive(Clone, Copy)]
pub struct Side {
    pub x: f64,
    pub y: f64,
    pub tx: f64,
    pub ty: f64,
    pub nx: f64,
    pub ny: f64,
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

fn shape_svg(pts: &[Pt], fill: &str, extra: &str) -> String {
    format!("<path d=\"{}\" fill=\"{}\"{}/>", poly_d(pts), fill, extra)
}

fn path_svg(d: &str, stroke: &str, w: f64, extra: &str) -> String {
    format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"{}/>",
        d,
        stroke,
        f(w),
        extra
    )
}

fn star_svg(x: f64, y: f64, r: f64, k: f64, n: usize, fill: &str, extra: &str) -> String {
    let mut pts = Vec::with_capacity(n * 2);
    for i in 0..(n * 2) {
        let a = PI * (i as f64) / (n as f64) - PI / 2.0;
        let rr = if i % 2 == 1 { r * k } else { r };
        pts.push([x + a.cos() * rr, y + a.sin() * rr]);
    }
    shape_svg(&pts, fill, extra)
}

pub struct OrnamentCtx<'a> {
    pub k: &'a mut Kit,
    pub corners: Vec<Pt>,
    pub sides: Vec<Side>,
    pub tip: Pt,
    pub tail: Pt,
    pub axis: Pt,
    pub box_: [f64; 4],
    pub cx: f64,
    pub cy: f64,
    pub s: f64,
    pub c: &'a str,
    pub inner: Option<Vec<Pt>>,
    pub mask_sil: String,
}

impl<'a> OrnamentCtx<'a> {
    fn edge(&self) -> String {
        format!(" stroke=\"{}\" stroke-width=\".18\"", dark(self.c, 0.55))
    }
    fn inside(&mut self, inner: &str) -> String {
        self.k.mask(&self.mask_sil, inner)
    }
}

pub fn draw_ornament(key: &str, o: &mut OrnamentCtx) -> String {
    match key {
        "none" => String::new(),
        "studs" => {
            let e = o.edge();
            o.corners
                .iter()
                .map(|p| {
                    format!(
                        "{}{}",
                        circle_svg(p[0], p[1], 0.6 * o.s, o.c, &e),
                        circle_svg(
                            p[0] - 0.18 * o.s,
                            p[1] - 0.18 * o.s,
                            0.22 * o.s,
                            "#fff",
                            " opacity=\".6\""
                        )
                    )
                })
                .collect()
        }
        "rivets" => {
            let e = o.edge();
            let mut out = String::new();
            for sd in &o.sides {
                for &k in &[-2.4, 0.0, 2.4] {
                    let px = sd.x + sd.tx * k * o.s - sd.nx * 0.9;
                    let py = sd.y + sd.ty * k * o.s - sd.ny * 0.9;
                    out.push_str(&circle_svg(px, py, 0.36 * o.s, o.c, &e));
                }
            }
            out
        }
        "pearls" => {
            let lc = light(o.c, 0.55);
            let mut out = String::new();
            for sd in &o.sides {
                for &k in &[-2.4, -1.2, 0.0, 1.2, 2.4] {
                    let px = sd.x + sd.tx * k * o.s;
                    let py = sd.y + sd.ty * k * o.s;
                    let r = 0.42 * o.s;
                    out.push_str(&circle_svg(px, py, r, &lc, ""));
                    out.push_str(&circle_svg(
                        px - r * 0.35,
                        py - r * 0.35,
                        r * 0.4,
                        "#fff",
                        " opacity=\".8\"",
                    ));
                }
            }
            out
        }
        "crosses" => o
            .corners
            .iter()
            .map(|p| {
                let r = 0.75 * o.s;
                let d = format!(
                    "M{} {} H{} M{} {} V{}",
                    f(p[0] - r),
                    f(p[1]),
                    f(p[0] + r),
                    f(p[0]),
                    f(p[1] - r),
                    f(p[1] + r)
                );
                path_svg(&d, o.c, 0.38 * o.s, "")
            })
            .collect(),
        "stars" => o
            .corners
            .iter()
            .map(|p| star_svg(p[0], p[1], 0.95 * o.s, 0.35, 4, o.c, ""))
            .collect(),
        "brackets" => {
            let b = o.box_;
            let g = 0.9;
            let l = 2.2 * o.s;
            let x0 = b[0] - g;
            let y0 = b[1] - g;
            let x1 = b[2] + g;
            let y1 = b[3] + g;
            let d = format!(
                "M{} {} V{} H{} M{} {} H{} V{} M{} {} V{} H{} M{} {} H{} V{}",
                f(x0),
                f(y0 + l),
                f(y0),
                f(x0 + l),
                f(x1 - l),
                f(y0),
                f(x1),
                f(y0 + l),
                f(x1),
                f(y1 - l),
                f(y1),
                f(x1 - l),
                f(x0 + l),
                f(y1),
                f(x0),
                f(y1 - l)
            );
            path_svg(&d, o.c, 0.4, "")
        }
        "rosettes" => {
            let lc = light(o.c, 0.6);
            o.corners
                .iter()
                .map(|p| {
                    let mut out = String::new();
                    for i in 0..6 {
                        let a = PI * (i as f64) / 3.0;
                        out.push_str(&circle_svg(
                            p[0] + a.cos() * 0.45 * o.s,
                            p[1] + a.sin() * 0.45 * o.s,
                            0.32 * o.s,
                            o.c,
                            "",
                        ));
                    }
                    out.push_str(&circle_svg(p[0], p[1], 0.28 * o.s, &lc, ""));
                    out
                })
                .collect()
        }
        "diamonds" => {
            let e = o.edge();
            o.sides
                .iter()
                .map(|sd| {
                    let r = 0.85 * o.s;
                    let x = sd.x - sd.nx;
                    let y = sd.y - sd.ny;
                    shape_svg(
                        &[[x, y - r], [x + r * 0.7, y], [x, y + r], [x - r * 0.7, y]],
                        o.c,
                        &e,
                    )
                })
                .collect()
        }
        "fleurons" => {
            let dk = dark(o.c, 0.4);
            o.sides.iter().map(|sd| {
                let x = sd.x - sd.nx * 1.2;
                let y = sd.y - sd.ny * 1.2;
                let r = 0.9 * o.s;
                let a = (-sd.ny).atan2(-sd.nx) * 180.0 / PI;
                format!(
                    "<g transform=\"translate({} {}) rotate({})\"><path d=\"M0 {} Q{} {} 0 {} Q{} {} 0 {} Z\" fill=\"{}\"/><path d=\"M0 {} V{}\" stroke=\"{}\" stroke-width=\".16\"/></g>",
                    f(x), f(y), f(a + 90.0), f(-r), f(r), f(-r * 0.2), f(r), f(-r), f(-r * 0.2), f(-r), o.c, f(-r * 0.6), f(r * 0.6), dk
                )
            }).collect()
        }
        "filigree" => {
            let mut out = if let Some(inner) = o.inner.clone() {
                let st = format!(
                    "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\".2\" stroke-linejoin=\"round\" opacity=\".85\"/>",
                    poly_d(&inner),
                    o.c
                );
                o.inside(&st)
            } else {
                String::new()
            };
            for p in &o.corners {
                out.push_str(&circle_svg(p[0], p[1], 0.3 * o.s, o.c, ""));
            }
            for sd in &o.sides {
                let x = sd.x - sd.nx * 1.3;
                let y = sd.y - sd.ny * 1.3;
                let r = 0.45 * o.s;
                out.push_str(&shape_svg(
                    &[[x, y - r], [x + r, y], [x, y + r], [x - r, y]],
                    o.c,
                    "",
                ));
            }
            out
        }
        "flourish" => {
            let mut d = String::from("M");
            for i in 0..=40 {
                let t = (i as f64) / 40.0;
                let a = t * PI * 3.0;
                let r = 0.2 + t * 2.6 * o.s;
                if i > 0 {
                    d.push_str(" L");
                }
                d.push_str(&format!(
                    "{} {}",
                    f(o.cx + a.cos() * r),
                    f(o.cy + a.sin() * r)
                ));
            }
            let p = path_svg(&d, o.c, 0.3, "");
            o.inside(&p)
        }
        "scrolls" => {
            let mut out = String::new();
            let px = -o.axis[1];
            let py = o.axis[0];
            for &side in &[-1.0, 1.0] {
                let mut d = String::from("M");
                let cx = o.tail[0] - o.axis[0] * 1.6 + px * side * 1.3;
                let cy = o.tail[1] - o.axis[1] * 1.6 + py * side * 1.3;
                for i in 0..=30 {
                    let t = (i as f64) / 30.0;
                    let a = side * t * PI * 2.2;
                    let r = (1.4 - t * 1.1) * o.s;
                    if i > 0 {
                        d.push_str(" L");
                    }
                    d.push_str(&format!("{} {}", f(cx + a.cos() * r), f(cy + a.sin() * r)));
                }
                out.push_str(&path_svg(&d, o.c, 0.32, ""));
            }
            out
        }
        "chevrons" => {
            let chev = |p: Pt, dx: f64, dy: f64| {
                let r = 0.9 * o.s;
                let px = -dy;
                let py = dx;
                let x = p[0] + dx * 1.6;
                let y = p[1] + dy * 1.6;
                let d = format!(
                    "M{} {} L{} {} L{} {}",
                    f(x - dx * r + px * r),
                    f(y - dy * r + py * r),
                    f(x),
                    f(y),
                    f(x - dx * r - px * r),
                    f(y - dy * r - py * r)
                );
                path_svg(&d, o.c, 0.4, "")
            };
            format!(
                "{}{}",
                chev(o.tip, -o.axis[0], -o.axis[1]),
                chev(o.tail, o.axis[0], o.axis[1])
            )
        }
        "fans" => {
            let corners = o.corners.clone();
            let mut total = String::new();
            for p in corners {
                let mut out = String::new();
                let a0 = (o.cy - p[1]).atan2(o.cx - p[0]);
                let r = 1.3 * o.s;
                for i in -2..=2 {
                    let a = a0 + (i as f64) * 0.32;
                    let d = format!(
                        "M{} {} L{} {}",
                        f(p[0]),
                        f(p[1]),
                        f(p[0] + a.cos() * r),
                        f(p[1] + a.sin() * r)
                    );
                    out.push_str(&path_svg(&d, o.c, 0.22, ""));
                }
                total.push_str(&o.inside(&out));
            }
            total
        }
        "deco" => {
            let b = o.box_;
            let mut out = String::new();
            for k in 0..3 {
                let g = 0.5 + (k as f64) * 0.6;
                let l = (1.2 + (k as f64) * 0.7) * o.s;
                let d1 = format!(
                    "M{} {} V{} H{}",
                    f(b[0] - g),
                    f(b[1] - g + l),
                    f(b[1] - g),
                    f(b[0] - g + l)
                );
                let d2 = format!(
                    "M{} {} V{} H{}",
                    f(b[2] + g),
                    f(b[3] + g - l),
                    f(b[3] + g),
                    f(b[2] + g - l)
                );
                out.push_str(&path_svg(&d1, o.c, 0.22, ""));
                out.push_str(&path_svg(&d2, o.c, 0.22, ""));
            }
            out
        }
        "crown" => {
            let r = 1.1 * o.s;
            let x = o.tail[0] - o.axis[0] * 3.6;
            let y = o.tail[1] - o.axis[1] * 3.6;
            let pts = [
                [x - r, y + r * 0.6],
                [x - r, y - r * 0.2],
                [x - r * 0.5, y + r * 0.2],
                [x, y - r * 0.7],
                [x + r * 0.5, y + r * 0.2],
                [x + r, y - r * 0.2],
                [x + r, y + r * 0.6],
            ];
            let e = o.edge();
            format!(
                "{}{}",
                shape_svg(&pts, o.c, &e),
                circle_svg(x, y - r * 0.9, 0.2 * o.s, &light(o.c, 0.5), "")
            )
        }
        "laurels" => {
            let mut out = String::new();
            let big_r = (o.box_[2] - o.box_[0]).max(o.box_[3] - o.box_[1]) * 0.62;
            for &side in &[-1.0, 1.0] {
                for i in 0..6 {
                    let a = PI / 2.0 + side * (0.5 + (i as f64) * 0.36);
                    let x = o.cx + a.cos() * big_r;
                    let y = o.cy + a.sin() * big_r;
                    let ang = a * 180.0 / PI + if side > 0.0 { 60.0 } else { 120.0 };
                    let r = 0.75 * o.s;
                    out.push_str(&format!(
                        "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" transform=\"rotate({} {} {})\" fill=\"{}\"/>",
                        f(x), f(y), f(r), f(r * 0.38), f(ang), f(x), f(y), o.c
                    ));
                }
            }
            out
        }
        _ => String::new(),
    }
}
