//! All 6 trails + exact `Band.js` Catmull-Rom smooth non-overlapping quad renderer.
//! Ported 1-to-1 from `Band.js` and `trails/*.js`.

use crate::colors::{Rgba, color_at, to_rgba};
use crate::draw::{DrawCmd, DrawCtx, TrailCtx};
use crate::motion::Point;
use crate::settings::TrailKind;

// ---- Band.js exact implementation -------------------------------------------

/// Catmull-Rom curve through `pts`, inserting points every `step` pixels.
fn smooth_catmull_rom(pts: &[Point], step: f32) -> Vec<Point> {
    let n = pts.len();
    if n < 3 {
        return pts.to_vec();
    }
    let mut out = Vec::with_capacity(n * 3);
    for i in 0..(n - 1) {
        let p0 = pts[i.saturating_sub(1)];
        let p1 = pts[i];
        let p2 = pts[i + 1];
        let p3 = pts[(i + 2).min(n - 1)];
        let len = (p2.x - p1.x).hypot(p2.y - p1.y);
        let k = ((len / step).ceil() as usize).clamp(1, 6);
        for j in 0..k {
            let u = j as f32 / k as f32;
            let u2 = u * u;
            let u3 = u2 * u;
            let c = |a: f32, b: f32, cc: f32, d: f32| {
                0.5 * (2.0 * b
                    + (-a + cc) * u
                    + (2.0 * a - 5.0 * b + 4.0 * cc - d) * u2
                    + (-a + 3.0 * b - 3.0 * cc + d) * u3)
            };
            out.push(Point {
                x: c(p0.x, p1.x, p2.x, p3.x),
                y: c(p0.y, p1.y, p2.y, p3.y),
                t: p1.t + (p2.t - p1.t) * u,
            });
        }
    }
    out.push(pts[n - 1]);
    out
}

/// Draw a smooth variable-width band (exact port of `Band.band`).
fn draw_band(
    ctx: &mut DrawCtx,
    raw: &[Point],
    width_at: impl Fn(f32) -> f32,
    color_at_fn: impl Fn(f32) -> Rgba,
) {
    let pts = smooth_catmull_rom(raw, 4.0);
    let n = pts.len();
    if n < 2 {
        return;
    }

    let mut left: Vec<[f32; 2]> = Vec::with_capacity(n);
    let mut right: Vec<[f32; 2]> = Vec::with_capacity(n);

    for i in 0..n {
        let a = pts[i.saturating_sub(1)];
        let b = pts[(i + 1).min(n - 1)];
        let mut dx = b.x - a.x;
        let mut dy = b.y - a.y;
        let len = dx.hypot(dy);
        if len < 1e-6 {
            dx = 1.0;
            dy = 0.0;
        } else {
            dx /= len;
            dy /= len;
        }
        let h = width_at(pts[i].t).max(0.25) * 0.5;
        let mut l = [pts[i].x - dy * h, pts[i].y + dx * h];
        let mut r = [pts[i].x + dy * h, pts[i].y - dx * h];
        // Hold inside edge still on tight turns so quads never fold back
        if i > 0 {
            let fx = pts[i].x - pts[i - 1].x;
            let fy = pts[i].y - pts[i - 1].y;
            if (l[0] - left[i - 1][0]) * fx + (l[1] - left[i - 1][1]) * fy < 0.0 {
                l = left[i - 1];
            }
            if (r[0] - right[i - 1][0]) * fx + (r[1] - right[i - 1][1]) * fy < 0.0 {
                r = right[i - 1];
            }
        }
        left.push(l);
        right.push(r);
    }

    // Emit edge-to-edge quads tail-to-head
    let mut c_a = color_at_fn(pts[n - 1].t);
    for i in (0..(n - 1)).rev() {
        let p = pts[i];
        let q = pts[i + 1];
        let c_b = color_at_fn(p.t);
        if c_a[3] > 0.002 || c_b[3] > 0.002 {
            ctx.cmds.push(DrawCmd::BandQuad {
                l0: left[i],
                l1: left[i + 1],
                r1: right[i + 1],
                r0: right[i],
                gx0: q.x,
                gy0: q.y,
                c0: c_a,
                gx1: p.x,
                gy1: p.y,
                c1: c_b,
            });
        }
        c_a = c_b;
    }

    // Half-circle round cap at the newest end
    let p = pts[0];
    let q = pts[1];
    let r = width_at(p.t).max(0.25) * 0.5;
    let a = (left[0][1] - p.y).atan2(left[0][0] - p.x);
    let fx = p.x - q.x;
    let fy = p.y - q.y;
    let front =
        (a - std::f32::consts::FRAC_PI_2).cos() * fx + (a - std::f32::consts::FRAC_PI_2).sin() * fy;
    let cap_color = color_at_fn(p.t);
    if cap_color[3] > 0.002 {
        ctx.cmds.push(DrawCmd::BandCap {
            r,
            start_x: left[0][0],
            start_y: left[0][1],
            end_x: right[0][0],
            end_y: right[0][1],
            sweep_cw: front <= 0.0,
            color: cap_color,
        });
    }
}

// ---- Comet ------------------------------------------------------------------
pub fn draw_comet(g: &mut TrailCtx) {
    if g.pts.len() < 2 {
        return;
    }
    let size = g.size;
    let strength = g.strength;
    let mode = g.color_mode;
    let cctx = g.color_ctx;
    draw_band(
        g.ctx,
        g.pts,
        |t| size * (0.15 + 0.85 * (1.0 - t)),
        |t| to_rgba(color_at(mode, t, cctx), (1.0 - t).powf(1.3) * strength),
    );
}

// ---- Ribbon -----------------------------------------------------------------
pub fn draw_ribbon(g: &mut TrailCtx) {
    if g.pts.len() < 3 {
        return;
    }
    let size = g.size;
    let strength = g.strength;
    let mode = g.color_mode;
    let cctx = g.color_ctx;
    for &(w_mult, a_mult) in &[(2.2f32, 0.22f32), (0.8, 1.0)] {
        draw_band(
            g.ctx,
            g.pts,
            |_| (size * w_mult).max(1.0),
            |t| to_rgba(color_at(mode, t, cctx), (1.0 - t) * strength * a_mult),
        );
    }
}

// ---- Beads ------------------------------------------------------------------
pub fn draw_beads(g: &mut TrailCtx) {
    if g.pts.len() < 2 {
        return;
    }
    let gap = (g.size * 1.6).max(4.0);
    let mut carry = 0.0f32;
    for i in 0..(g.pts.len() - 1) {
        let a = g.pts[i];
        let b = g.pts[i + 1];
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let len = dx.hypot(dy);
        let mut d = if carry == 0.0 && i == 0 {
            0.0
        } else {
            gap - carry
        };
        while d <= len {
            let f = if len > 0.0 { d / len } else { 0.0 };
            let t = a.t + (b.t - a.t) * f;
            let k = 1.0 - t;
            let r = (g.size * 0.5 * (0.3 + 0.7 * k)).max(0.6);
            let col = g.color(t, k * g.strength);
            g.ctx.fill_circle(a.x + dx * f, a.y + dy * f, r, col);
            d += gap;
        }
        carry = (carry + len).rem_euclid(gap);
    }
}

// ---- Echoes -----------------------------------------------------------------
pub fn draw_echoes(g: &mut TrailCtx) {
    if g.pts.len() < 2 {
        return;
    }
    let gap = (g.size * 2.5).max(10.0);
    let lw = (g.size * 0.14).max(1.0);
    let mut last: Option<Point> = None;
    for &q in g.pts {
        if let Some(l) = last
            && (q.x - l.x).hypot(q.y - l.y) < gap
        {
            continue;
        }
        last = Some(q);
        let k = 1.0 - q.t;
        let r = g.size * (0.4 + q.t * 1.4);
        let col = g.color(q.t, k * g.strength);
        g.ctx.stroke_circle(q.x, q.y, r, col, lw);
    }
}

// ---- Sparks -----------------------------------------------------------------
const SPARKS_MAX: usize = 180;

pub struct SparkParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub age: f32,
    pub life: f32,
}

// ---- Bubbles ----------------------------------------------------------------
const BUBBLES_MAX: usize = 90;

pub struct Bubble {
    pub x: f32,
    pub y: f32,
    pub age: f32,
    pub life: f32,
    pub r: f32,
    pub phase: f32,
}

pub struct TrailMemory {
    pub trail_kind: TrailKind,
    pub sparks: Vec<SparkParticle>,
    pub bubbles: Vec<Bubble>,
    pub bubble_dist: f32,
}

impl Default for TrailMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl TrailMemory {
    pub fn new() -> Self {
        Self {
            trail_kind: TrailKind::None,
            sparks: Vec::new(),
            bubbles: Vec::new(),
            bubble_dist: 0.0,
        }
    }

    pub fn switch_if_needed(&mut self, kind: TrailKind) {
        if self.trail_kind != kind {
            self.sparks.clear();
            self.bubbles.clear();
            self.bubble_dist = 0.0;
            self.trail_kind = kind;
        }
    }

    pub fn has_particles(&self) -> bool {
        !self.sparks.is_empty() || !self.bubbles.is_empty()
    }
}

pub fn draw_sparks(g: &mut TrailCtx, mem: &mut TrailMemory) {
    if g.pts.len() > 1 && g.moved > 0.5 {
        let n = ((g.moved / (18.0 - g.size).max(4.0)).ceil() as usize).min(16);
        for _ in 0..n {
            if mem.sparks.len() >= SPARKS_MAX {
                break;
            }
            let ang = rand_f32() * std::f32::consts::TAU;
            let sp = 20.0 + rand_f32() * 90.0;
            let p = g.pts[0];
            mem.sparks.push(SparkParticle {
                x: p.x,
                y: p.y,
                vx: ang.cos() * sp,
                vy: ang.sin() * sp - 30.0,
                age: 0.0,
                life: g.life * (0.35 + rand_f32() * 0.5),
            });
        }
    }
    let dt = g.dt.min(0.1);
    let size = g.size;
    let strength = g.strength;
    let mode = g.color_mode;
    let cctx = g.color_ctx;
    let mut i = 0;
    while i < mem.sparks.len() {
        let s = &mut mem.sparks[i];
        s.age += dt;
        if s.age >= s.life {
            mem.sparks.swap_remove(i);
            continue;
        }
        s.vy += 160.0 * dt;
        s.x += s.vx * dt;
        s.y += s.vy * dt;
        let t = s.age / s.life;
        let col = to_rgba(color_at(mode, t, cctx), (1.0 - t) * strength);
        let lw = (size * 0.26 * (1.0 - t * 0.6)).max(1.0);
        g.ctx
            .line(s.x, s.y, s.x - s.vx * 0.045, s.y - s.vy * 0.045, col, lw);
        i += 1;
    }
    g.busy = !mem.sparks.is_empty();
}

pub fn draw_bubbles(g: &mut TrailCtx, mem: &mut TrailMemory) {
    mem.bubble_dist += g.moved;
    let gap = (g.size * 2.0).max(8.0);
    while !g.pts.is_empty() && mem.bubble_dist >= gap && mem.bubbles.len() < BUBBLES_MAX {
        mem.bubble_dist -= gap;
        let p = g.pts[0];
        mem.bubbles.push(Bubble {
            x: p.x + (rand_f32() - 0.5) * g.size,
            y: p.y,
            age: 0.0,
            life: g.life * (0.7 + rand_f32() * 0.6),
            r: g.size * (0.25 + rand_f32() * 0.45),
            phase: rand_f32() * std::f32::consts::TAU,
        });
    }
    if mem.bubble_dist > gap * 4.0 {
        mem.bubble_dist = 0.0;
    }
    let dt = g.dt.min(0.1);
    let lw = (g.size * 0.1).max(0.8);
    let mode = g.color_mode;
    let cctx = g.color_ctx;
    let mut i = 0;
    while i < mem.bubbles.len() {
        let b = &mut mem.bubbles[i];
        b.age += dt;
        if b.age >= b.life {
            mem.bubbles.swap_remove(i);
            continue;
        }
        let t = b.age / b.life;
        b.y -= 28.0 * dt;
        let bx = b.x + (b.phase + b.age * 4.0).sin() * g.size * 0.35;
        let r = b.r * (1.0 + t * 0.8);
        let rgb = color_at(mode, t, cctx);
        let col = to_rgba(rgb, (1.0 - t) * g.strength);
        let col_hl = to_rgba(rgb, (1.0 - t) * g.strength * 0.5);
        g.ctx.stroke_circle(bx, b.y, r, col, lw);
        g.ctx
            .fill_circle(bx - r * 0.35, b.y - r * 0.35, (r * 0.18).max(0.6), col_hl);
        i += 1;
    }
    g.busy = !mem.bubbles.is_empty();
}

// Fast lock-free xorshift PRNG
use std::sync::atomic::{AtomicU64, Ordering};
static SEED: AtomicU64 = AtomicU64::new(0x853c_49e6_748f_ea9b);

fn rand_f32() -> f32 {
    let s = SEED.fetch_add(0x9e37_79b9_7f4a_7c15, Ordering::Relaxed);
    let v = s
        .wrapping_mul(0x6c62_272e_07bb_0142)
        .wrapping_add(0x62b8_2175_6295_c58d);
    let bits = 0x3f80_0000u32 | ((v >> 41) as u32 & 0x007f_ffff);
    f32::from_bits(bits) - 1.0
}
