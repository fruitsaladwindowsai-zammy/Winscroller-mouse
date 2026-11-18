//! Every cursor glyph in the theme — 1-to-1 port of `Glyphs.js`.
//! Maps directly onto every Windows system cursor role!

use super::arrows::arrow_by_key;
use super::geo::{Piece, Pt, rot};
use std::f64::consts::PI;

const TIP: Pt = [7.0, 5.5];
const LEAN: f64 = -26.0;
const C: f64 = 16.0;
const BADGE: Pt = [24.2, 24.2];

#[derive(Clone, Debug)]
pub struct Layer {
    pub pieces: Vec<Piece>,
    pub box_: [f64; 4],
    pub badge: bool,
    pub marks: Vec<Piece>,
}

#[derive(Clone, Debug)]
pub struct Glyph {
    pub hot: Pt,
    pub layers: Vec<Layer>,
    pub rot: f64,
    pub tip: Option<Pt>,
    pub inward: Pt,
    pub creases: Vec<Piece>,
}

fn norm(v: Pt) -> Pt {
    let l = v[0].hypot(v[1]);
    let l = if l == 0.0 { 1.0 } else { l };
    [v[0] / l, v[1] / l]
}

fn box_of(pts: &[Pt]) -> [f64; 4] {
    let mut x0 = 1e9f64;
    let mut y0 = 1e9f64;
    let mut x1 = -1e9f64;
    let mut y1 = -1e9f64;
    for &p in pts {
        x0 = x0.min(p[0]);
        y0 = y0.min(p[1]);
        x1 = x1.max(p[0]);
        y1 = y1.max(p[1]);
    }
    [x0, y0, x1, y1]
}

fn n(v: f64) -> String {
    let r = (v * 100.0).round() / 100.0;
    if r == 0.0 {
        "0".into()
    } else {
        format!("{}", r)
    }
}

fn circle_path(cx: f64, cy: f64, r: f64) -> String {
    format!(
        "M{} {} A{} {} 0 1 1 {} {} A{} {} 0 1 1 {} {}",
        n(cx + r),
        n(cy),
        n(r),
        n(r),
        n(cx - r),
        n(cy),
        n(r),
        n(r),
        n(cx + r),
        n(cy)
    )
}

fn arc_path(cx: f64, cy: f64, r: f64, a0: f64, a1: f64) -> String {
    let p0 = [cx + r * a0.cos(), cy + r * a0.sin()];
    let p1 = [cx + r * a1.cos(), cy + r * a1.sin()];
    let large = if (a1 - a0).rem_euclid(2.0 * PI) > PI {
        1
    } else {
        0
    };
    format!(
        "M{} {} A{} {} 0 {} 1 {} {}",
        n(p0[0]),
        n(p0[1]),
        n(r),
        n(r),
        large,
        n(p1[0]),
        n(p1[1])
    )
}

fn rect(x: f64, y: f64, w: f64, h: f64, r: f64, rot_deg: f64) -> Piece {
    Piece::Rect {
        x,
        y,
        w,
        h,
        r,
        rot: rot_deg,
        op: 1.0,
    }
}

fn line(d: impl Into<String>, w: f64, op: f64) -> Piece {
    Piece::Stroke { d: d.into(), w, op }
}

pub fn arrow_glyph(key: &str) -> Glyph {
    let a = arrow_by_key(key);
    if let Some(dot_r) = a.dot {
        let c = [9.5, 9.5];
        return Glyph {
            hot: c,
            layers: vec![Layer {
                pieces: vec![Piece::Circle {
                    cx: c[0],
                    cy: c[1],
                    r: dot_r,
                    op: 1.0,
                }],
                box_: [c[0] - dot_r, c[1] - dot_r, c[0] + dot_r, c[1] + dot_r],
                badge: false,
                marks: vec![],
            }],
            rot: 0.0,
            tip: None,
            inward: [0.0, 0.0],
            creases: vec![],
        };
    }
    let pts: Vec<Pt> = a
        .points
        .iter()
        .map(|&p| {
            let q = if a.upright { rot(p, LEAN, 0.0, 0.0) } else { p };
            [q[0] + TIP[0], q[1] + TIP[1]]
        })
        .collect();
    let tip = pts[0];
    let e1 = norm([pts[1][0] - tip[0], pts[1][1] - tip[1]]);
    let last = pts[pts.len() - 1];
    let e2 = norm([last[0] - tip[0], last[1] - tip[1]]);
    let bx = box_of(&pts);
    Glyph {
        hot: tip,
        tip: Some(tip),
        inward: norm([e1[0] + e2[0], e1[1] + e2[1]]),
        layers: vec![Layer {
            pieces: vec![Piece::Poly { p: pts, op: 1.0 }],
            box_: bx,
            badge: false,
            marks: vec![],
        }],
        rot: 0.0,
        creases: vec![],
    }
}

fn badged(arrow_key: &str, marks: Vec<Piece>) -> Glyph {
    let mut g = arrow_glyph(arrow_key);
    g.layers.push(Layer {
        badge: true,
        pieces: vec![Piece::Circle {
            cx: BADGE[0],
            cy: BADGE[1],
            r: 5.4,
            op: 1.0,
        }],
        box_: [18.8, 18.8, 29.6, 29.6],
        marks,
    });
    g
}

fn badge_mark(kind: &str) -> Vec<Piece> {
    let x = BADGE[0];
    let y = BADGE[1];
    match kind {
        "help" => vec![
            line(
                format!(
                    "M{} {} A1.95 1.95 0 1 1 {} {} Q{} {} {} {}",
                    n(x - 1.9),
                    n(y - 1.6),
                    n(x + 1.0),
                    n(y + 0.1),
                    n(x),
                    n(y + 0.6),
                    n(x),
                    n(y + 1.6)
                ),
                1.3,
                1.0,
            ),
            Piece::Circle {
                cx: x,
                cy: y + 3.3,
                r: 0.8,
                op: 1.0,
            },
        ],
        "copy" => vec![line(
            format!(
                "M{} {} H{} M{} {} V{}",
                n(x - 2.6),
                n(y),
                n(x + 2.6),
                n(x),
                n(y - 2.6),
                n(y + 2.6)
            ),
            1.4,
            1.0,
        )],
        "alias" => vec![
            line(
                format!(
                    "M{} {} V{} Q{} {} {} {} H{}",
                    n(x - 2.4),
                    n(y + 2.6),
                    n(y + 1.0),
                    n(x - 2.4),
                    n(y - 1.4),
                    n(x),
                    n(y - 1.4),
                    n(x + 2.4)
                ),
                1.25,
                1.0,
            ),
            line(
                format!(
                    "M{} {} L{} {} L{} {}",
                    n(x + 1.0),
                    n(y - 2.9),
                    n(x + 2.5),
                    n(y - 1.4),
                    n(x + 1.0),
                    n(y + 0.1)
                ),
                1.25,
                1.0,
            ),
        ],
        "menu" => vec![line(
            format!(
                "M{} {} H{} M{} {} H{} M{} {} H{}",
                n(x - 2.5),
                n(y - 1.9),
                n(x + 2.5),
                n(x - 2.5),
                n(y),
                n(x + 2.5),
                n(x - 2.5),
                n(y + 1.9),
                n(x + 2.5)
            ),
            1.1,
            1.0,
        )],
        _ => vec![],
    }
}

pub fn spinner(
    style: &str,
    cx: f64,
    cy: f64,
    r: f64,
    frame: usize,
    frames: usize,
    w: f64,
) -> Vec<Piece> {
    let turn = 2.0 * PI * (frame as f64) / (frames as f64) - PI / 2.0;
    if style == "dots" {
        let mut out = Vec::with_capacity(8);
        for i in 0..8 {
            let a = 2.0 * PI * (i as f64) / 8.0 - PI / 2.0;
            let age = ((frame as f64 / frames as f64) * 8.0 - i as f64).rem_euclid(8.0);
            out.push(Piece::Circle {
                cx: cx + r * a.cos(),
                cy: cy + r * a.sin(),
                r: r * 0.24,
                op: (1.0 - age / 8.0).max(0.2),
            });
        }
        return out;
    }
    if style == "orbit" {
        return vec![
            Piece::Circle {
                cx,
                cy,
                r: r * 0.22,
                op: 0.55,
            },
            Piece::Circle {
                cx: cx + r * turn.cos(),
                cy: cy + r * turn.sin(),
                r: r * 0.3,
                op: 1.0,
            },
            Piece::Circle {
                cx: cx + r * (turn + PI).cos(),
                cy: cy + r * (turn + PI).sin(),
                r: r * 0.22,
                op: 1.0,
            },
        ];
    }
    vec![
        line(circle_path(cx, cy, r), w, 0.3),
        line(arc_path(cx, cy, r, turn, turn + PI * 0.6), w, 1.0),
    ]
}

pub fn pointing_glyph(style: &str, arrow_key: &str) -> Glyph {
    if style == "arrow" {
        let mut g = arrow_glyph(arrow_key);
        g.layers.push(Layer {
            badge: true,
            pieces: vec![Piece::Circle {
                cx: 22.5,
                cy: 22.5,
                r: 3.4,
                op: 1.0,
            }],
            box_: [19.0, 19.0, 26.0, 26.0],
            marks: vec![],
        });
        return g;
    }
    if style == "ring" {
        return Glyph {
            hot: [C, C],
            layers: vec![Layer {
                pieces: vec![
                    line(circle_path(C, C, 6.6), 2.2, 1.0),
                    Piece::Circle {
                        cx: C,
                        cy: C,
                        r: 1.4,
                        op: 1.0,
                    },
                ],
                box_: [9.0, 9.0, 23.0, 23.0],
                badge: false,
                marks: vec![],
            }],
            rot: 0.0,
            tip: None,
            inward: [0.0, 0.0],
            creases: vec![],
        };
    }
    Glyph {
        hot: [13.4, 3.4],
        layers: vec![Layer {
            box_: [5.0, 3.0, 27.0, 28.0],
            badge: false,
            marks: vec![],
            pieces: vec![
                rect(11.2, 3.2, 4.4, 15.0, 2.2, 0.0),
                rect(15.4, 11.6, 4.1, 8.0, 2.0, 0.0),
                rect(19.3, 12.6, 3.9, 7.5, 1.9, 0.0),
                rect(22.9, 14.2, 3.4, 6.4, 1.7, 0.0),
                rect(11.2, 14.5, 15.1, 13.3, 4.5, 0.0),
                rect(5.6, 15.6, 8.4, 4.0, 2.0, 38.0),
            ],
        }],
        rot: 0.0,
        tip: None,
        inward: [0.0, 0.0],
        creases: vec![line(
            "M15.45 13.4 V17 M19.35 14.4 V17.4 M22.95 16 V18",
            0.7,
            0.45,
        )],
    }
}

pub fn beam_glyph(style: &str, rot_deg: f64) -> Glyph {
    let pieces = match style {
        "slim" => vec![
            line("M16 8 V24", 1.15, 1.0),
            line("M13.9 7.2 H18.1", 1.15, 1.0),
            line("M13.9 24.8 H18.1", 1.15, 1.0),
        ],
        "caret" => vec![line("M16 6.5 V25.5", 2.3, 1.0)],
        _ => vec![
            line("M16 7.8 V24.2", 1.7, 1.0),
            line("M12.8 6.6 Q16 6.6 16 8.4 Q16 6.6 19.2 6.6", 1.5, 1.0),
            line("M12.8 25.4 Q16 25.4 16 23.6 Q16 25.4 19.2 25.4", 1.5, 1.0),
        ],
    };
    Glyph {
        hot: [C, C],
        rot: rot_deg,
        layers: vec![Layer {
            pieces,
            box_: [12.0, 6.0, 20.0, 26.0],
            badge: false,
            marks: vec![],
        }],
        tip: None,
        inward: [0.0, 0.0],
        creases: vec![],
    }
}

pub fn not_allowed_glyph() -> Glyph {
    Glyph {
        hot: [C, C],
        layers: vec![Layer {
            box_: [7.0, 7.0, 25.0, 25.0],
            badge: false,
            marks: vec![],
            pieces: vec![
                line(circle_path(C, C, 8.2), 2.6, 1.0),
                line("M10.2 21.8 L21.8 10.2", 2.6, 1.0),
            ],
        }],
        rot: 0.0,
        tip: None,
        inward: [0.0, 0.0],
        creases: vec![],
    }
}

pub fn crosshair_glyph() -> Glyph {
    Glyph {
        hot: [C, C],
        layers: vec![Layer {
            box_: [4.0, 4.0, 28.0, 28.0],
            badge: false,
            marks: vec![],
            pieces: vec![
                line(
                    "M16 4.5 V12.3 M16 19.7 V27.5 M4.5 16 H12.3 M19.7 16 H27.5",
                    1.6,
                    1.0,
                ),
                Piece::Circle {
                    cx: C,
                    cy: C,
                    r: 1.3,
                    op: 1.0,
                },
            ],
        }],
        rot: 0.0,
        tip: None,
        inward: [0.0, 0.0],
        creases: vec![],
    }
}

pub fn move_glyph() -> Glyph {
    let head: [Pt; 3] = [[16.0, 3.8], [20.4, 9.0], [11.6, 9.0]];
    let mut pieces = vec![line("M16 8.5 V23.5 M8.5 16 H23.5", 2.2, 1.0)];
    for &a in &[0.0, 90.0, 180.0, 270.0] {
        pieces.push(Piece::Poly {
            p: head.iter().map(|&p| rot(p, a, C, C)).collect(),
            op: 1.0,
        });
    }
    Glyph {
        hot: [C, C],
        layers: vec![Layer {
            pieces,
            box_: [3.8, 3.8, 28.2, 28.2],
            badge: false,
            marks: vec![],
        }],
        rot: 0.0,
        tip: None,
        inward: [0.0, 0.0],
        creases: vec![],
    }
}

pub fn double_arrow_glyph(rot_deg: f64) -> Glyph {
    Glyph {
        hot: [C, C],
        rot: rot_deg,
        layers: vec![Layer {
            box_: [10.0, 3.8, 22.0, 28.2],
            badge: false,
            marks: vec![],
            pieces: vec![Piece::Poly {
                p: vec![
                    [16.0, 3.8],
                    [21.6, 10.0],
                    [18.1, 10.0],
                    [18.1, 22.0],
                    [21.6, 22.0],
                    [16.0, 28.2],
                    [10.4, 22.0],
                    [13.9, 22.0],
                    [13.9, 10.0],
                    [10.4, 10.0],
                ],
                op: 1.0,
            }],
        }],
        tip: None,
        inward: [0.0, 0.0],
        creases: vec![],
    }
}

pub fn up_arrow_glyph() -> Glyph {
    Glyph {
        hot: [C, 8.6],
        rot: 0.0,
        layers: vec![Layer {
            box_: [10.4, 8.6, 21.6, 27.4],
            badge: false,
            marks: vec![],
            pieces: vec![Piece::Poly {
                p: vec![
                    [16.0, 8.6],
                    [21.6, 14.6],
                    [18.1, 14.6],
                    [18.1, 27.4],
                    [13.9, 27.4],
                    [13.9, 14.6],
                    [10.4, 14.6],
                ],
                op: 1.0,
            }],
        }],
        tip: None,
        inward: [0.0, 0.0],
        creases: vec![],
    }
}

pub fn wait_glyph(busy_style: &str, frame: usize, frames: usize) -> Glyph {
    Glyph {
        hot: [C, C],
        layers: vec![Layer {
            box_: [7.0, 7.0, 25.0, 25.0],
            badge: false,
            marks: vec![],
            pieces: spinner(busy_style, C, C, 7.6, frame, frames, 2.6),
        }],
        rot: 0.0,
        tip: None,
        inward: [0.0, 0.0],
        creases: vec![],
    }
}

pub fn progress_glyph(arrow_key: &str, busy_style: &str, frame: usize, frames: usize) -> Glyph {
    badged(
        arrow_key,
        spinner(busy_style, BADGE[0], BADGE[1], 2.9, frame, frames, 1.2),
    )
}

pub fn help_glyph(arrow_key: &str) -> Glyph {
    badged(arrow_key, badge_mark("help"))
}

pub fn copy_glyph(arrow_key: &str) -> Glyph {
    badged(arrow_key, badge_mark("copy"))
}

pub fn alias_glyph(arrow_key: &str) -> Glyph {
    badged(arrow_key, badge_mark("alias"))
}

pub fn menu_glyph(arrow_key: &str) -> Glyph {
    badged(arrow_key, badge_mark("menu"))
}
