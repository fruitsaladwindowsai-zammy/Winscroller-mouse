//! All 14 arrow shapes — 1-to-1 port of `Arrows.js` and `arrows/*.js`.

use super::geo::Pt;
use std::f64::consts::PI;

#[allow(dead_code)]
pub struct ArrowDef {
    pub key: &'static str,
    pub label: &'static str,
    pub upright: bool,
    pub dot: Option<f64>,
    pub points: Vec<Pt>,
}

pub const ARROW_KEYS: &[(&str, &str)] = &[
    ("classic", "Classic"),
    ("slim", "Slim"),
    ("chunky", "Chunky"),
    ("wedge", "Wedge"),
    ("dart", "Dart"),
    ("kite", "Kite"),
    ("needle", "Needle"),
    ("teardrop", "Teardrop"),
    ("leaf", "Leaf"),
    ("bullet", "Bullet"),
    ("crest", "Crest"),
    ("chevron", "Chevron"),
    ("feather", "Feather"),
    ("dot", "Dot"),
];

pub fn arrow_by_key(key: &str) -> ArrowDef {
    match key {
        "slim" => ArrowDef {
            key: "slim",
            label: "Slim",
            upright: false,
            dot: None,
            points: vec![
                [0.0, 0.0],
                [0.0, 20.0],
                [3.9, 16.6],
                [6.3, 22.4],
                [8.6, 21.4],
                [6.3, 15.7],
                [11.3, 15.6],
            ],
        },
        "chunky" => ArrowDef {
            key: "chunky",
            label: "Chunky",
            upright: false,
            dot: None,
            points: vec![
                [0.0, 0.0],
                [0.0, 19.0],
                [5.4, 14.6],
                [8.2, 20.6],
                [12.4, 18.8],
                [9.7, 13.0],
                [15.8, 13.0],
            ],
        },
        "wedge" => ArrowDef {
            key: "wedge",
            label: "Wedge",
            upright: false,
            dot: None,
            points: vec![[0.0, 0.0], [0.0, 19.5], [5.3, 14.6], [14.3, 14.2]],
        },
        "dart" => ArrowDef {
            key: "dart",
            label: "Dart",
            upright: true,
            dot: None,
            points: vec![[0.0, 0.0], [7.2, 19.0], [0.0, 13.4], [-7.2, 19.0]],
        },
        "kite" => ArrowDef {
            key: "kite",
            label: "Kite",
            upright: true,
            dot: None,
            points: vec![[0.0, 0.0], [6.4, 11.5], [0.0, 19.5], [-6.4, 11.5]],
        },
        "needle" => ArrowDef {
            key: "needle",
            label: "Needle",
            upright: true,
            dot: None,
            points: vec![[0.0, 0.0], [3.5, 21.0], [0.0, 18.0], [-3.5, 21.0]],
        },
        "teardrop" => {
            let cy = 12.5f64;
            let r = 7.0f64;
            let a = (r / cy).acos();
            let from = -PI / 2.0 + a;
            let to = 3.0 * PI / 2.0 - a;
            let mut out = vec![[0.0, 0.0]];
            for i in 0..=22 {
                let t = from + (to - from) * (i as f64) / 22.0;
                out.push([r * t.cos(), cy + r * t.sin()]);
            }
            ArrowDef {
                key: "teardrop",
                label: "Teardrop",
                upright: true,
                dot: None,
                points: out,
            }
        }
        "leaf" => {
            let mut out = vec![[0.0, 0.0]];
            for i in 1..16 {
                let t = (i as f64) / 16.0;
                out.push([5.6 * (PI * t).sin().powf(0.85) * (0.85 + 0.3 * t), 20.0 * t]);
            }
            out.push([0.0, 20.0]);
            for i in (1..16).rev() {
                let t = (i as f64) / 16.0;
                out.push([
                    -5.6 * (PI * t).sin().powf(0.85) * (0.85 + 0.3 * t),
                    20.0 * t,
                ]);
            }
            ArrowDef {
                key: "leaf",
                label: "Leaf",
                upright: true,
                dot: None,
                points: out,
            }
        }
        "bullet" => {
            let mut out = vec![[0.0, 0.0]];
            for i in 1..=8 {
                let t = (i as f64) / 8.0;
                out.push([3.6 * (t * PI / 2.0).sin(), 9.0 * t]);
            }
            out.push([3.6, 19.0]);
            out.push([-3.6, 19.0]);
            for i in (1..=8).rev() {
                let t = (i as f64) / 8.0;
                out.push([-3.6 * (t * PI / 2.0).sin(), 9.0 * t]);
            }
            ArrowDef {
                key: "bullet",
                label: "Bullet",
                upright: true,
                dot: None,
                points: out,
            }
        }
        "crest" => ArrowDef {
            key: "crest",
            label: "Crest",
            upright: true,
            dot: None,
            points: vec![
                [0.0, 0.0],
                [6.6, 5.0],
                [6.2, 13.5],
                [3.8, 17.6],
                [0.0, 19.4],
                [-3.8, 17.6],
                [-6.2, 13.5],
                [-6.6, 5.0],
            ],
        },
        "chevron" => ArrowDef {
            key: "chevron",
            label: "Chevron",
            upright: true,
            dot: None,
            points: vec![
                [0.0, 0.0],
                [7.0, 15.5],
                [4.1, 17.0],
                [0.0, 8.6],
                [-4.1, 17.0],
                [-7.0, 15.5],
            ],
        },
        "feather" => ArrowDef {
            key: "feather",
            label: "Feather",
            upright: true,
            dot: None,
            points: vec![
                [0.0, 0.0],
                [3.4, 3.6],
                [5.0, 8.4],
                [4.2, 10.8],
                [5.3, 12.6],
                [4.4, 16.4],
                [1.6, 19.6],
                [0.55, 20.2],
                [0.55, 24.0],
                [-0.55, 24.0],
                [-0.55, 20.2],
                [-1.8, 19.2],
                [-4.2, 15.2],
                [-4.6, 9.6],
                [-3.2, 4.2],
            ],
        },
        "dot" => ArrowDef {
            key: "dot",
            label: "Dot",
            upright: false,
            dot: Some(5.5),
            points: vec![],
        },
        _ => ArrowDef {
            key: "classic",
            label: "Classic",
            upright: false,
            dot: None,
            points: vec![
                [0.0, 0.0],
                [0.0, 20.5],
                [4.8, 16.2],
                [8.1, 23.3],
                [11.2, 21.9],
                [8.0, 15.0],
                [14.3, 14.8],
            ],
        },
    }
}
