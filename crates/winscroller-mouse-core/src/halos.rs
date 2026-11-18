//! All 6 halos + shake-to-find rings — 1-to-1 port of `halos/*.js` and `Find.js`.

use crate::colors::Rgba;
use crate::draw::{DrawCtx, HaloCtx};

// ---- Glow -------------------------------------------------------------------
pub fn draw_glow(g: &mut HaloCtx) {
    let r = g.size;
    let stops = vec![
        (0.0, g.color(0.0, 0.55 * g.strength)),
        (0.45, g.color(0.3, 0.22 * g.strength)),
        (1.0, g.color(0.6, 0.0)),
    ];
    g.ctx.radial_gradient(g.x, g.y, r, stops);
}

// ---- Ring -------------------------------------------------------------------
pub fn draw_ring(g: &mut HaloCtx) {
    let r = g.size * 0.5;
    let fill_col = g.color(0.0, 0.12 * g.strength);
    let stroke_col = g.color(0.0, 0.9 * g.strength);
    let lw = (g.size * 0.05).max(1.2);
    g.ctx.fill_circle(g.x, g.y, r, fill_col);
    g.ctx.stroke_circle(g.x, g.y, r, stroke_col, lw);
}

// ---- Pulse ------------------------------------------------------------------
pub fn draw_pulse(g: &mut HaloCtx) {
    let base = g.size * 0.42;
    let breath = 1.0 + 0.08 * ((g.time as f32) * 3.2).sin();
    let lw = (g.size * 0.045).max(1.2);

    let col_ring = g.color(0.0, 0.85 * g.strength);
    g.ctx.stroke_circle(g.x, g.y, base * breath, col_ring, lw);

    let w = (g.time as f32).rem_euclid(1.4) / 1.4;
    let col_wave = g.color(w, (1.0 - w) * 0.6 * g.strength);
    g.ctx
        .stroke_circle(g.x, g.y, base * (1.0 + w * 1.2), col_wave, lw);

    g.busy = true;
}

// ---- Spotlight --------------------------------------------------------------
pub fn draw_spotlight(g: &mut HaloCtx) {
    let r = g.size * 1.4;
    let ring_col = g.color(0.0, 0.35 * g.strength);
    g.ctx.spotlight(g.x, g.y, r, 0.6 * g.strength, ring_col);
}

// ---- Crosshair --------------------------------------------------------------
pub fn draw_crosshair(g: &mut HaloCtx) {
    let gap = g.size * 0.35;
    let reach = g.w.max(g.h);
    let stops = [
        (0.0, g.color(0.0, 0.8 * g.strength)),
        (0.35, g.color(0.5, 0.25 * g.strength)),
        (1.0, g.color(1.0, 0.0)),
    ];
    for &(dx, dy) in &[(1.0f32, 0.0f32), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
        g.ctx.linear_gradient_line(
            g.x + dx * gap,
            g.y + dy * gap,
            g.x + dx * reach,
            g.y + dy * reach,
            stops,
            1.0,
        );
    }
}

// ---- Target -----------------------------------------------------------------
pub fn draw_target(g: &mut HaloCtx) {
    let r = g.size * 0.5;
    let arm = r * 0.45;
    let base_rot = (g.time as f32) * 0.6;
    let lw = (g.size * 0.05).max(1.5);
    let col = g.color(0.0, 0.9 * g.strength);

    // Exact match to target.js: rotated square brackets at (r, -r) -> (r-arm, -r) and (r, -r+arm)
    for i in 0..4 {
        let ang = base_rot + (i as f32) * std::f32::consts::FRAC_PI_2;
        let (s, c) = ang.sin_cos();
        let xform =
            |lx: f32, ly: f32| -> (f32, f32) { (g.x + lx * c - ly * s, g.y + lx * s + ly * c) };
        let (x0, y0) = xform(r - arm, -r);
        let (x1, y1) = xform(r, -r);
        let (x2, y2) = xform(r, -r + arm);
        g.ctx.line(x0, y0, x1, y1, col, lw);
        g.ctx.line(x1, y1, x2, y2, col, lw);
    }
    g.busy = true;
}

// ---- Find (shake-to-find) ---------------------------------------------------
pub const FIND_LENGTH: f32 = 1.4;

pub fn draw_find(
    ctx: &mut DrawCtx,
    x: f32,
    y: f32,
    size: f32,
    age: f32,
    color_fn: impl Fn(f32, f32) -> Rgba,
) -> bool {
    if age < 0.0 || age >= FIND_LENGTH {
        return false;
    }
    for i in 0..3usize {
        let t = (age - (i as f32) * 0.16) / (FIND_LENGTH - 0.32);
        if t <= 0.0 || t >= 1.0 {
            continue;
        }
        let r = size * (0.4 + t * 3.2);
        let col = color_fn(t, (1.0 - t) * 0.95);
        let lw = (size * 0.12 * (1.0 - t)).max(2.0);
        ctx.stroke_circle(x, y, r, col, lw);
    }
    let f = (1.0 - age / 0.5).max(0.0);
    if f > 0.0 {
        let stops = vec![(0.0, color_fn(0.0, 0.6 * f)), (1.0, color_fn(0.5, 0.0))];
        ctx.radial_gradient(x, y, size * 1.6, stops);
    }
    true
}
