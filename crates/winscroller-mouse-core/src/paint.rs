//! Per-frame effects compositor — 1-to-1 port of `Paint.js`.
//! Used both by the multi-monitor overlay and by the Settings Panel live preview!

use crate::colors::{ColorCtx, color_at, to_rgba};
use crate::draw::{DrawCtx, HaloCtx, TrailCtx};
use crate::halos::{
    draw_crosshair, draw_find, draw_glow, draw_pulse, draw_ring, draw_spotlight, draw_target,
};
use crate::motion::Point;
use crate::settings::{HaloKind, HaloShow, Settings, TrailKind};
use crate::trails::{
    TrailMemory, draw_beads, draw_bubbles, draw_comet, draw_echoes, draw_ribbon, draw_sparks,
};

pub struct FrameState<'a> {
    pub settings: &'a Settings,
    pub pts: &'a [Point],
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub screen_x: f32,
    pub screen_y: f32,
    pub screen_w: f32,
    pub screen_h: f32,
    pub cursor_on_this_screen: bool,
    pub color_ctx: &'a ColorCtx,
    pub time_secs: f64,
    pub dt: f32,
    pub moved: f32,
    pub still_secs: f32,
    pub find_age: f32,
    pub trail_mem: &'a mut TrailMemory,
}

fn halo_alpha(show: HaloShow, still: f32) -> f32 {
    match show {
        HaloShow::Moving => (1.0 - (still - 0.15) / 0.4).clamp(0.0, 1.0),
        HaloShow::Still => ((still - 0.6) / 0.4).clamp(0.0, 1.0),
        HaloShow::Always => 1.0,
    }
}

/// Paint one frame into a reusable `DrawCtx`. Returns `true` while effects keep animating on their own.
pub fn paint_frame_into(ctx: &mut DrawCtx, s: &mut FrameState) -> bool {
    ctx.reset(s.screen_w, s.screen_h);
    let mut busy = false;
    let cfg = s.settings;

    s.trail_mem.switch_if_needed(cfg.trail);

    // --- Trail ---------------------------------------------------------------
    let run_trail =
        cfg.trail != TrailKind::None && (!s.pts.is_empty() || s.trail_mem.has_particles());
    if run_trail {
        let mut tg = TrailCtx {
            ctx,
            pts: s.pts,
            size: cfg.trail_size as f32,
            strength: cfg.trail_strength_f(),
            life: cfg.trail_life_secs(),
            dt: s.dt,
            moved: s.moved,
            color_mode: cfg.color_mode,
            color_ctx: s.color_ctx,
            busy: false,
        };
        match cfg.trail {
            TrailKind::Comet => draw_comet(&mut tg),
            TrailKind::Ribbon => draw_ribbon(&mut tg),
            TrailKind::Beads => draw_beads(&mut tg),
            TrailKind::Echoes => draw_echoes(&mut tg),
            TrailKind::Sparks => draw_sparks(&mut tg, s.trail_mem),
            TrailKind::Bubbles => draw_bubbles(&mut tg, s.trail_mem),
            TrailKind::None => {}
        }
        if tg.busy {
            busy = true;
        }
        // Still-fading path points keep animating until pruned
        if !s.pts.is_empty() {
            busy = true;
        }
    }

    // --- Halo + Find ---------------------------------------------------------
    if s.cursor_on_this_screen {
        let cx = s.cursor_x - s.screen_x;
        let cy = s.cursor_y - s.screen_y;

        if cfg.halo != HaloKind::None {
            let a = halo_alpha(cfg.halo_show, s.still_secs);
            if a > 0.01 {
                let mut hg = HaloCtx {
                    ctx,
                    x: cx,
                    y: cy,
                    w: s.screen_w,
                    h: s.screen_h,
                    size: cfg.halo_size as f32,
                    strength: a * cfg.halo_strength_f(),
                    time: s.time_secs,
                    color_mode: cfg.color_mode,
                    color_ctx: s.color_ctx,
                    busy: false,
                };
                match cfg.halo {
                    HaloKind::Glow => draw_glow(&mut hg),
                    HaloKind::Ring => draw_ring(&mut hg),
                    HaloKind::Pulse => draw_pulse(&mut hg),
                    HaloKind::Spotlight => draw_spotlight(&mut hg),
                    HaloKind::Crosshair => draw_crosshair(&mut hg),
                    HaloKind::Target => draw_target(&mut hg),
                    HaloKind::None => {}
                }
                if hg.busy {
                    busy = true;
                }
            }
            // Keep animating while halo is fading in/out or waiting to fade in when still
            if cfg.halo_show != HaloShow::Always
                && ((a > 0.01 && a < 0.99)
                    || (cfg.halo_show == HaloShow::Still && s.still_secs < 1.05))
            {
                busy = true;
            }
            // Rainbow/Fade color modes animate continuously while a halo is visible
            if a > 0.01
                && matches!(
                    cfg.color_mode,
                    crate::colors::ColorMode::Rainbow | crate::colors::ColorMode::Fade
                )
            {
                busy = true;
            }
        }

        if s.find_age >= 0.0 {
            let mode = cfg.color_mode;
            let cctx = s.color_ctx;
            if draw_find(ctx, cx, cy, cfg.find_size_f(), s.find_age, |t, a| {
                to_rgba(color_at(mode, t, cctx), a)
            }) {
                busy = true;
            }
        }
    }

    busy
}
