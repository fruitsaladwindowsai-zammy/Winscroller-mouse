//! Geometry helpers for cursor outlines — 1-to-1 port of `Geo.js`.

use std::f64::consts::PI;

pub type Pt = [f64; 2];

#[derive(Clone, Debug)]
pub enum Piece {
    Poly {
        p: Vec<Pt>,
        op: f64,
    },
    Circle {
        cx: f64,
        cy: f64,
        r: f64,
        op: f64,
    },
    Rect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        r: f64,
        rot: f64,
        op: f64,
    },
    Stroke {
        d: String,
        w: f64,
        op: f64,
    },
}

impl Piece {
    pub fn op(&self) -> f64 {
        match self {
            Piece::Poly { op, .. }
            | Piece::Circle { op, .. }
            | Piece::Rect { op, .. }
            | Piece::Stroke { op, .. } => *op,
        }
    }
    pub fn is_stroke(&self) -> bool {
        matches!(self, Piece::Stroke { .. })
    }
}

pub fn f(v: f64) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    if r == 0.0 {
        "0".to_string()
    } else {
        format!("{}", r)
    }
}

pub fn area(p: &[Pt]) -> f64 {
    let mut a = 0.0;
    let n = p.len();
    for i in 0..n {
        let q = p[i];
        let r = p[(i + 1) % n];
        a += q[0] * r[1] - r[0] * q[1];
    }
    a * 0.5
}

pub fn rot(pt: Pt, deg: f64, cx: f64, cy: f64) -> Pt {
    if deg == 0.0 {
        return pt;
    }
    let a = deg * PI / 180.0;
    let (s, c) = a.sin_cos();
    let x = pt[0] - cx;
    let y = pt[1] - cy;
    [cx + x * c - y * s, cy + x * s + y * c]
}

pub fn polygon_of(p: &Piece) -> Option<Vec<Pt>> {
    match p {
        Piece::Poly { p, .. } => Some(p.clone()),
        Piece::Circle { cx, cy, r, .. } => {
            let mut out = Vec::with_capacity(28);
            for i in 0..28 {
                let a = 2.0 * PI * (i as f64) / 28.0;
                out.push([cx + r * a.cos(), cy + r * a.sin()]);
            }
            Some(out)
        }
        Piece::Rect {
            x,
            y,
            w,
            h,
            r,
            rot: rot_deg,
            ..
        } => {
            let rr = r.max(0.0).min(w * 0.5).min(h * 0.5);
            let mut out = Vec::with_capacity(20);
            let corners = [
                (x + w - rr, y + rr, -90.0),
                (x + w - rr, y + h - rr, 0.0),
                (x + rr, y + h - rr, 90.0),
                (x + rr, y + rr, 180.0),
            ];
            for &(cx, cy, a0) in &corners {
                for k in 0..=4 {
                    let a = (a0 + 90.0 * (k as f64) / 4.0) * PI / 180.0;
                    out.push([cx + rr * a.cos(), cy + rr * a.sin()]);
                }
            }
            if *rot_deg != 0.0 {
                let cx = x + w * 0.5;
                let cy = y + h * 0.5;
                Some(out.into_iter().map(|q| rot(q, *rot_deg, cx, cy)).collect())
            } else {
                Some(out)
            }
        }
        Piece::Stroke { .. } => None,
    }
}

pub fn offset(poly: &[Pt], d: f64) -> Vec<Pt> {
    if poly.len() < 3 || d.abs() < 1e-6 {
        return poly.to_vec();
    }
    let n = poly.len();
    let sgn = if area(poly) > 0.0 { 1.0 } else { -1.0 };
    let mut out = Vec::with_capacity(n * 2);
    let normal = |a: Pt, b: Pt| -> Pt {
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let l = dx.hypot(dy);
        let l = if l == 0.0 { 1.0 } else { l };
        [sgn * dy / l, -sgn * dx / l]
    };

    for i in 0..n {
        let p = poly[i];
        let a = poly[(i + n - 1) % n];
        let b = poly[(i + 1) % n];
        let n1 = normal(a, p);
        let n2 = normal(p, b);
        let turn = (p[0] - a[0]) * (b[1] - p[1]) - (p[1] - a[1]) * (b[0] - p[0]);
        let sticks_out = (turn * sgn > 0.0) == (d > 0.0);
        if sticks_out {
            let a1 = n1[1].atan2(n1[0]);
            let a2 = n2[1].atan2(n2[0]);
            let mut da = a2 - a1;
            while da > PI {
                da -= 2.0 * PI;
            }
            while da < -PI {
                da += 2.0 * PI;
            }
            let steps = ((da.abs() / (PI / 8.0)).ceil() as usize).max(1);
            let k = d.abs();
            let sd = d.signum();
            for s in 0..=steps {
                let t = a1 + da * (s as f64) / (steps as f64);
                out.push([p[0] + sd * k * t.cos(), p[1] + sd * k * t.sin()]);
            }
        } else {
            let dot = n1[0] * n2[0] + n1[1] * n2[1];
            let nx = n1[0] + n2[0];
            let ny = n1[1] + n2[1];
            let l = nx.hypot(ny);
            if l < 1e-3 {
                out.push([p[0] + n1[0] * d, p[1] + n1[1] * d]);
                continue;
            }
            let len = d * (2.0 / (1.0 + dot).max(0.02)).sqrt().min(3.0);
            out.push([p[0] + nx / l * len, p[1] + ny / l * len]);
        }
    }
    out
}

pub fn poly_d(poly: &[Pt]) -> String {
    if poly.is_empty() {
        return String::new();
    }
    let mut s = String::with_capacity(poly.len() * 14);
    for (i, q) in poly.iter().enumerate() {
        if i == 0 {
            s.push('M');
        } else {
            s.push_str(" L");
        }
        s.push_str(&f(q[0]));
        s.push(' ');
        s.push_str(&f(q[1]));
    }
    s.push_str(" Z");
    s
}

pub fn perimeter(poly: &[Pt]) -> f64 {
    let n = poly.len();
    let mut s = 0.0;
    for i in 0..n {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        s += (b[0] - a[0]).hypot(b[1] - a[1]);
    }
    s
}

#[derive(Clone, Copy)]
pub struct WalkPt {
    pub x: f64,
    pub y: f64,
    pub tx: f64,
    pub ty: f64,
    pub nx: f64,
    pub ny: f64,
    pub s: f64,
}

pub fn walk(poly: &[Pt], step: f64) -> Vec<WalkPt> {
    let mut out = Vec::new();
    if poly.len() < 3 {
        return out;
    }
    let per = perimeter(poly);
    let n = ((per / step).round() as usize).max(3);
    let st = per / (n as f64);
    let sgn = if area(poly) > 0.0 { 1.0 } else { -1.0 };

    let seg = |k: usize| -> (Pt, Pt, f64) {
        let a = poly[k % poly.len()];
        let b = poly[(k + 1) % poly.len()];
        (a, b, (b[0] - a[0]).hypot(b[1] - a[1]))
    };

    let mut i = 0usize;
    let mut along = 0.0;
    let (mut a, mut b, mut len) = seg(0);

    for k in 0..n {
        let target = (k as f64) * st;
        while along + len < target && i < poly.len() * 2 {
            along += len;
            i += 1;
            let s = seg(i);
            a = s.0;
            b = s.1;
            len = s.2;
        }
        let t = if len > 0.0 {
            (target - along) / len
        } else {
            0.0
        };
        let tx = if len > 0.0 { (b[0] - a[0]) / len } else { 1.0 };
        let ty = if len > 0.0 { (b[1] - a[1]) / len } else { 0.0 };
        out.push(WalkPt {
            x: a[0] + (b[0] - a[0]) * t,
            y: a[1] + (b[1] - a[1]) * t,
            tx,
            ty,
            nx: sgn * ty,
            ny: -sgn * tx,
            s: target,
        });
    }
    out
}

pub fn displaced(poly: &[Pt], step: f64, func: impl Fn(f64) -> f64) -> Vec<Pt> {
    walk(poly, step)
        .into_iter()
        .map(|p| {
            let d = func(p.s);
            [p.x + p.nx * d, p.y + p.ny * d]
        })
        .collect()
}

pub fn wobble(seed: f64) -> impl Fn(f64) -> f64 {
    let a = 1.7 + (seed * 0.37).rem_euclid(1.0);
    let b = 3.1 + (seed * 0.61).rem_euclid(1.0);
    let c = 0.9 + (seed * 0.13).rem_euclid(1.0);
    move |s: f64| {
        0.5 * (s * a + seed).sin()
            + 0.3 * (s * b + seed * 2.0).sin()
            + 0.2 * (s * c + seed * 3.0).sin()
    }
}

pub struct Rng {
    x: u32,
}

impl Rng {
    pub fn new(seed: i64) -> Self {
        let mut x = (seed as u64).wrapping_mul(2654435761) as u32;
        if x == 0 {
            x = 1;
        }
        Self { x }
    }
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> f64 {
        self.x ^= self.x << 13;
        self.x ^= self.x >> 17;
        self.x ^= self.x << 5;
        (self.x as f64) / 4294967296.0
    }
}
