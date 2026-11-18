//! Low-allocation 2D draw command stream executed by Direct2D (`ID2D1DeviceContext`).
//! Supports exact Band.js smooth quads (with additive PLUS blending), round caps,
//! circles, radial/linear gradients, and Spotlight DESTINATION_OUT mask.

use crate::colors::{ColorCtx, ColorMode, Rgba, color_at, to_rgba};
use crate::motion::Point;

#[derive(Debug, Clone)]
pub enum DrawCmd {
    /// Filled circle
    FillCircle {
        cx: f32,
        cy: f32,
        r: f32,
        color: Rgba,
    },
    /// Stroked circle
    StrokeCircle {
        cx: f32,
        cy: f32,
        r: f32,
        color: Rgba,
        width: f32,
    },
    /// Solid line with round caps
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: Rgba,
        width: f32,
    },
    /// Linear-gradient line
    LinearGradientLine {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        stops: [(f32, Rgba); 3],
        width: f32,
    },
    /// Radial-gradient filled circle (up to 3 stops)
    RadialGradient {
        cx: f32,
        cy: f32,
        r: f32,
        stops: Vec<(f32, Rgba)>,
    },
    /// Exact Band.js 4-sided tapered quad with linear gradient & additive PLUS compositing
    BandQuad {
        l0: [f32; 2],
        l1: [f32; 2],
        r1: [f32; 2],
        r0: [f32; 2],
        /// Gradient start (older end q) and end (newer end p)
        gx0: f32,
        gy0: f32,
        c0: Rgba,
        gx1: f32,
        gy1: f32,
        c1: Rgba,
    },
    /// Half-circle round cap at the newest end of a Band.js stroke (drawn in additive PLUS mode)
    BandCap {
        r: f32,
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        sweep_cw: bool,
        color: Rgba,
    },
    /// Spotlight: dims the screen except a soft transparent radial hole around (cx, cy), plus subtle ring
    Spotlight {
        cx: f32,
        cy: f32,
        r: f32,
        screen_w: f32,
        screen_h: f32,
        dim_alpha: f32,
        ring_color: Rgba,
    },
}

pub struct DrawCtx {
    pub cmds: Vec<DrawCmd>,
    pub screen_w: f32,
    pub screen_h: f32,
}

impl DrawCtx {
    pub fn new(w: f32, h: f32) -> Self {
        Self {
            cmds: Vec::with_capacity(512),
            screen_w: w,
            screen_h: h,
        }
    }

    pub fn reset(&mut self, w: f32, h: f32) {
        self.cmds.clear();
        self.screen_w = w;
        self.screen_h = h;
    }

    pub fn fill_circle(&mut self, cx: f32, cy: f32, r: f32, color: Rgba) {
        if r > 0.05 && color[3] > 0.002 {
            self.cmds.push(DrawCmd::FillCircle { cx, cy, r, color });
        }
    }

    pub fn stroke_circle(&mut self, cx: f32, cy: f32, r: f32, color: Rgba, width: f32) {
        if r > 0.05 && width > 0.05 && color[3] > 0.002 {
            self.cmds.push(DrawCmd::StrokeCircle {
                cx,
                cy,
                r,
                color,
                width,
            });
        }
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Rgba, width: f32) {
        if width > 0.05 && color[3] > 0.002 {
            self.cmds.push(DrawCmd::Line {
                x1,
                y1,
                x2,
                y2,
                color,
                width,
            });
        }
    }

    pub fn linear_gradient_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        stops: [(f32, Rgba); 3],
        width: f32,
    ) {
        self.cmds.push(DrawCmd::LinearGradientLine {
            x1,
            y1,
            x2,
            y2,
            stops,
            width,
        });
    }

    pub fn radial_gradient(&mut self, cx: f32, cy: f32, r: f32, stops: Vec<(f32, Rgba)>) {
        if r > 0.5 {
            self.cmds.push(DrawCmd::RadialGradient { cx, cy, r, stops });
        }
    }

    pub fn spotlight(&mut self, cx: f32, cy: f32, r: f32, dim_alpha: f32, ring_color: Rgba) {
        self.cmds.push(DrawCmd::Spotlight {
            cx,
            cy,
            r,
            screen_w: self.screen_w,
            screen_h: self.screen_h,
            dim_alpha,
            ring_color,
        });
    }
}

pub struct TrailCtx<'a> {
    pub ctx: &'a mut DrawCtx,
    pub pts: &'a [Point],
    pub size: f32,
    pub strength: f32,
    pub life: f32,
    pub dt: f32,
    pub moved: f32,
    pub color_mode: ColorMode,
    pub color_ctx: &'a ColorCtx,
    pub busy: bool,
}

impl<'a> TrailCtx<'a> {
    #[inline]
    pub fn color(&self, t: f32, alpha: f32) -> Rgba {
        to_rgba(color_at(self.color_mode, t, self.color_ctx), alpha)
    }
}

pub struct HaloCtx<'a> {
    pub ctx: &'a mut DrawCtx,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub size: f32,
    pub strength: f32,
    pub time: f64,
    pub color_mode: ColorMode,
    pub color_ctx: &'a ColorCtx,
    pub busy: bool,
}

impl<'a> HaloCtx<'a> {
    #[inline]
    pub fn color(&self, t: f32, alpha: f32) -> Rgba {
        to_rgba(color_at(self.color_mode, t, self.color_ctx), alpha)
    }
}
