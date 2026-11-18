//! Hardware-accelerated DirectComposition + DXGI Flip-Model + Direct3D11 + Direct2D Renderer.
//!
//! Why this architecture:
//! - True per-pixel alpha transparency over the Windows desktop (`DXGI_ALPHA_MODE_PREMULTIPLIED`)
//! - `WS_EX_NOREDIRECTIONBITMAP` eliminates the GDI redirection surface (huge RAM savings on 4K/multi-monitor!)
//! - True `D2D1_COMPOSITE_MODE_PLUS` (additive "lighter" for seamless Catmull-Rom Band.js quads)
//! - True `D2D1_COMPOSITE_MODE_DESTINATION_OUT` (exact Spotlight radial cutout!)
//! - Reuses a single shared D3D11/D2D1/DComp device across all monitors, and a cached SolidColorBrush per overlay
//! - Calls `IDXGIDevice3::Trim()` when idle to release unused GPU memory

use windows::{
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::{Common::*, *},
            Direct3D::*,
            Direct3D11::*,
            DirectComposition::*,
            Dxgi::{Common::*, *},
        },
        UI::WindowsAndMessaging::*,
    },
    core::*,
};
use windows_numerics::Vector2;

use winscroller_mouse_core::colors::Rgba;
use winscroller_mouse_core::draw::DrawCmd;

#[inline]
pub fn pt(x: f32, y: f32) -> Vector2 {
    Vector2 { X: x, Y: y }
}

#[inline]
pub fn rgba_d2d(c: Rgba) -> D2D1_COLOR_F {
    // Direct2D brushes take straight (unpremultiplied) RGBA; the render target itself is premultiplied.
    D2D1_COLOR_F {
        r: c[0],
        g: c[1],
        b: c[2],
        a: c[3],
    }
}

/// Shared GPU devices across all monitor overlay windows (low memory footprint).
pub struct GpuContext {
    pub _d3d_device: ID3D11Device,
    pub dxgi_device: IDXGIDevice,
    pub dxgi_factory: IDXGIFactory2,
    pub d2d_factory: ID2D1Factory1,
    pub d2d_device: ID2D1Device,
    pub dcomp_device: IDCompositionDevice,
    pub round_stroke: ID2D1StrokeStyle,
}

impl GpuContext {
    pub unsafe fn new() -> Result<Self> {
        let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;
        let feature_levels = [
            D3D_FEATURE_LEVEL_11_1,
            D3D_FEATURE_LEVEL_11_0,
            D3D_FEATURE_LEVEL_10_1,
            D3D_FEATURE_LEVEL_10_0,
        ];

        let mut d3d_device: Option<ID3D11Device> = None;
        let hr = D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            flags,
            Some(&feature_levels),
            D3D11_SDK_VERSION,
            Some(&mut d3d_device),
            None,
            None,
        );
        if hr.is_err() {
            // Fallback to WARP software rasterizer if no hardware GPU
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_WARP,
                HMODULE::default(),
                flags,
                Some(&feature_levels),
                D3D11_SDK_VERSION,
                Some(&mut d3d_device),
                None,
                None,
            )?;
        }
        let d3d_device = d3d_device.unwrap();
        let dxgi_device: IDXGIDevice = d3d_device.cast()?;
        let adapter: IDXGIAdapter = dxgi_device.GetAdapter()?;
        let dxgi_factory: IDXGIFactory2 = adapter.GetParent()?;

        let d2d_factory: ID2D1Factory1 =
            D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
        let d2d_device: ID2D1Device = d2d_factory.CreateDevice(&dxgi_device)?;
        let dcomp_device: IDCompositionDevice = DCompositionCreateDevice(&dxgi_device)?;

        let stroke_props = D2D1_STROKE_STYLE_PROPERTIES1 {
            startCap: D2D1_CAP_STYLE_ROUND,
            endCap: D2D1_CAP_STYLE_ROUND,
            dashCap: D2D1_CAP_STYLE_ROUND,
            lineJoin: D2D1_LINE_JOIN_ROUND,
            miterLimit: 1.0,
            dashStyle: D2D1_DASH_STYLE_SOLID,
            dashOffset: 0.0,
            transformType: D2D1_STROKE_TRANSFORM_TYPE_NORMAL,
        };
        let round_stroke: ID2D1StrokeStyle =
            d2d_factory.CreateStrokeStyle(&stroke_props, None)?.into();

        Ok(Self {
            _d3d_device: d3d_device,
            dxgi_device,
            dxgi_factory,
            d2d_factory,
            d2d_device,
            dcomp_device,
            round_stroke,
        })
    }

    /// Release unused GPU staging memory when the pointer is idle.
    pub unsafe fn trim(&self) {
        if let Ok(dev3) = self.dxgi_device.cast::<IDXGIDevice3>() {
            dev3.Trim();
        }
    }
}

/// One click-through transparent DirectComposition overlay per physical monitor.
pub struct OverlayWindow {
    pub hwnd: HWND,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    swap_chain: IDXGISwapChain1,
    d2d_ctx: ID2D1DeviceContext,
    _target_bitmap: ID2D1Bitmap1,
    _dcomp_target: IDCompositionTarget,
    _dcomp_visual: IDCompositionVisual,
    solid_brush: ID2D1SolidColorBrush,
    /// True when the last rendered frame was completely transparent (nothing visible).
    pub is_clear: bool,
}

impl OverlayWindow {
    pub unsafe fn new(
        gpu: &GpuContext,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        instance: HINSTANCE,
        class_name: PCWSTR,
    ) -> Result<Self> {
        let w = w.max(1);
        let h = h.max(1);

        let ex_style = WS_EX_NOREDIRECTIONBITMAP
            | WS_EX_LAYERED
            | WS_EX_TRANSPARENT
            | WS_EX_TOPMOST
            | WS_EX_TOOLWINDOW
            | WS_EX_NOACTIVATE;

        let hwnd = CreateWindowExW(
            ex_style,
            class_name,
            w!("Winscroller Mouse Overlay"),
            WS_POPUP,
            x,
            y,
            w as i32,
            h as i32,
            None,
            None,
            Some(instance),
            None,
        )?;

        let sc_desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: w,
            Height: h,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            Stereo: FALSE,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
            Flags: 0,
        };

        let swap_chain =
            gpu.dxgi_factory
                .CreateSwapChainForComposition(&gpu.dxgi_device, &sc_desc, None)?;

        let dcomp_target = gpu.dcomp_device.CreateTargetForHwnd(hwnd, true)?;
        let dcomp_visual = gpu.dcomp_device.CreateVisual()?;
        dcomp_visual.SetContent(&swap_chain)?;
        dcomp_target.SetRoot(&dcomp_visual)?;
        gpu.dcomp_device.Commit()?;

        let d2d_ctx = gpu
            .d2d_device
            .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
        d2d_ctx.SetDpi(96.0, 96.0);
        d2d_ctx.SetUnitMode(D2D1_UNIT_MODE_PIXELS);

        let surface: IDXGISurface = swap_chain.GetBuffer(0)?;
        let bmp_props = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
            colorContext: std::mem::ManuallyDrop::new(None),
        };
        let target_bitmap = d2d_ctx.CreateBitmapFromDxgiSurface(&surface, Some(&bmp_props))?;
        d2d_ctx.SetTarget(&target_bitmap);

        let solid_brush = d2d_ctx.CreateSolidColorBrush(
            &D2D1_COLOR_F {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            None,
        )?;

        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

        let mut ov = Self {
            hwnd,
            x,
            y,
            width: w,
            height: h,
            swap_chain,
            d2d_ctx,
            _target_bitmap: target_bitmap,
            _dcomp_target: dcomp_target,
            _dcomp_visual: dcomp_visual,
            solid_brush,
            is_clear: false,
        };
        // Present one clean transparent frame immediately
        let _ = ov.clear_and_present();
        Ok(ov)
    }

    /// Keep the overlay above newly raised topmost windows (like taskbar/start menu).
    pub unsafe fn ensure_topmost(&self) {
        let _ = SetWindowPos(
            self.hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        );
    }

    /// Clear to fully transparent and present (called once when going idle or hidden).
    pub unsafe fn clear_and_present(&mut self) -> Result<()> {
        if self.is_clear {
            return Ok(());
        }
        self.d2d_ctx.BeginDraw();
        self.d2d_ctx.Clear(Some(&D2D1_COLOR_F {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }));
        self.d2d_ctx.EndDraw(None, None)?;
        let _ = self.swap_chain.Present(0, DXGI_PRESENT(0));
        self.is_clear = true;
        Ok(())
    }

    /// Render a frame of `DrawCmd`s and present smoothly.
    pub unsafe fn render(&mut self, gpu: &GpuContext, cmds: &[DrawCmd], vsync: bool) -> Result<()> {
        if cmds.is_empty() {
            return self.clear_and_present();
        }
        self.d2d_ctx.BeginDraw();
        self.d2d_ctx.Clear(Some(&D2D1_COLOR_F {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }));
        self.d2d_ctx
            .SetAntialiasMode(D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);

        execute_draw_cmds(
            &gpu.d2d_factory,
            &self.d2d_ctx,
            &self.solid_brush,
            &gpu.round_stroke,
            cmds,
        )?;

        self.d2d_ctx.EndDraw(None, None)?;
        let sync = if vsync { 1 } else { 0 };
        let _ = self.swap_chain.Present(sync, DXGI_PRESENT(0));
        self.is_clear = false;
        Ok(())
    }
}

impl Drop for OverlayWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

/// Shared Direct2D command executor — used by BOTH the desktop overlay and the Settings live preview!
pub unsafe fn execute_draw_cmds(
    factory: &ID2D1Factory1,
    ctx: &ID2D1DeviceContext,
    solid_brush: &ID2D1SolidColorBrush,
    round_stroke: &ID2D1StrokeStyle,
    cmds: &[DrawCmd],
) -> Result<()> {
    let mut in_plus_mode = false;

    for cmd in cmds {
        let want_plus = matches!(cmd, DrawCmd::BandQuad { .. } | DrawCmd::BandCap { .. });
        if want_plus != in_plus_mode {
            ctx.SetPrimitiveBlend(if want_plus {
                D2D1_PRIMITIVE_BLEND_ADD
            } else {
                D2D1_PRIMITIVE_BLEND_SOURCE_OVER
            });
            in_plus_mode = want_plus;
        }

        match cmd {
            DrawCmd::FillCircle { cx, cy, r, color } => {
                solid_brush.SetColor(&rgba_d2d(*color));
                ctx.FillEllipse(
                    &D2D1_ELLIPSE {
                        point: pt(*cx, *cy),
                        radiusX: *r,
                        radiusY: *r,
                    },
                    solid_brush,
                );
            }
            DrawCmd::StrokeCircle {
                cx,
                cy,
                r,
                color,
                width,
            } => {
                solid_brush.SetColor(&rgba_d2d(*color));
                ctx.DrawEllipse(
                    &D2D1_ELLIPSE {
                        point: pt(*cx, *cy),
                        radiusX: *r,
                        radiusY: *r,
                    },
                    solid_brush,
                    *width,
                    None,
                );
            }
            DrawCmd::Line {
                x1,
                y1,
                x2,
                y2,
                color,
                width,
            } => {
                solid_brush.SetColor(&rgba_d2d(*color));
                ctx.DrawLine(
                    pt(*x1, *y1),
                    pt(*x2, *y2),
                    solid_brush,
                    *width,
                    round_stroke,
                );
            }
            DrawCmd::LinearGradientLine {
                x1,
                y1,
                x2,
                y2,
                stops,
                width,
            } => {
                let d2d_stops = [
                    D2D1_GRADIENT_STOP {
                        position: stops[0].0,
                        color: rgba_d2d(stops[0].1),
                    },
                    D2D1_GRADIENT_STOP {
                        position: stops[1].0,
                        color: rgba_d2d(stops[1].1),
                    },
                    D2D1_GRADIENT_STOP {
                        position: stops[2].0,
                        color: rgba_d2d(stops[2].1),
                    },
                ];
                let col = ctx.CreateGradientStopCollection(
                    &d2d_stops,
                    D2D1_COLOR_SPACE_SRGB,
                    D2D1_COLOR_SPACE_SRGB,
                    D2D1_BUFFER_PRECISION_8BPC_UNORM,
                    D2D1_EXTEND_MODE_CLAMP,
                    D2D1_COLOR_INTERPOLATION_MODE_STRAIGHT,
                )?;
                let brush = ctx.CreateLinearGradientBrush(
                    &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                        startPoint: pt(*x1, *y1),
                        endPoint: pt(*x2, *y2),
                    },
                    None,
                    &col,
                )?;
                ctx.DrawLine(pt(*x1, *y1), pt(*x2, *y2), &brush, *width, None);
            }
            DrawCmd::RadialGradient { cx, cy, r, stops } => {
                let d2d_stops: Vec<D2D1_GRADIENT_STOP> = stops
                    .iter()
                    .map(|&(pos, c)| D2D1_GRADIENT_STOP {
                        position: pos,
                        color: rgba_d2d(c),
                    })
                    .collect();
                let col = ctx.CreateGradientStopCollection(
                    &d2d_stops,
                    D2D1_COLOR_SPACE_SRGB,
                    D2D1_COLOR_SPACE_SRGB,
                    D2D1_BUFFER_PRECISION_8BPC_UNORM,
                    D2D1_EXTEND_MODE_CLAMP,
                    D2D1_COLOR_INTERPOLATION_MODE_STRAIGHT,
                )?;
                let brush = ctx.CreateRadialGradientBrush(
                    &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                        center: pt(*cx, *cy),
                        gradientOriginOffset: pt(0.0, 0.0),
                        radiusX: *r,
                        radiusY: *r,
                    },
                    None,
                    &col,
                )?;
                ctx.FillEllipse(
                    &D2D1_ELLIPSE {
                        point: pt(*cx, *cy),
                        radiusX: *r,
                        radiusY: *r,
                    },
                    &brush,
                );
            }
            DrawCmd::BandQuad {
                l0,
                l1,
                r1,
                r0,
                gx0,
                gy0,
                c0,
                gx1,
                gy1,
                c1,
            } => {
                let geom = factory.CreatePathGeometry()?;
                let sink = geom.Open()?;
                sink.BeginFigure(pt(l0[0], l0[1]), D2D1_FIGURE_BEGIN_FILLED);
                sink.AddLines(&[pt(l1[0], l1[1]), pt(r1[0], r1[1]), pt(r0[0], r0[1])]);
                sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                sink.Close()?;

                if (gx1 - gx0).abs() + (gy1 - gy0).abs() > 0.01 {
                    let d2d_stops = [
                        D2D1_GRADIENT_STOP {
                            position: 0.0,
                            color: rgba_d2d(*c0),
                        },
                        D2D1_GRADIENT_STOP {
                            position: 1.0,
                            color: rgba_d2d(*c1),
                        },
                    ];
                    let col = ctx.CreateGradientStopCollection(
                        &d2d_stops,
                        D2D1_COLOR_SPACE_SRGB,
                        D2D1_COLOR_SPACE_SRGB,
                        D2D1_BUFFER_PRECISION_8BPC_UNORM,
                        D2D1_EXTEND_MODE_CLAMP,
                        D2D1_COLOR_INTERPOLATION_MODE_STRAIGHT,
                    )?;
                    let brush = ctx.CreateLinearGradientBrush(
                        &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                            startPoint: pt(*gx0, *gy0),
                            endPoint: pt(*gx1, *gy1),
                        },
                        None,
                        &col,
                    )?;
                    ctx.FillGeometry(&geom, &brush, None);
                } else {
                    solid_brush.SetColor(&rgba_d2d(*c1));
                    ctx.FillGeometry(&geom, solid_brush, None);
                }
            }
            DrawCmd::BandCap {
                r,
                start_x,
                start_y,
                end_x,
                end_y,
                sweep_cw,
                color,
            } => {
                let geom = factory.CreatePathGeometry()?;
                let sink = geom.Open()?;
                sink.BeginFigure(pt(*start_x, *start_y), D2D1_FIGURE_BEGIN_FILLED);
                sink.AddArc(&D2D1_ARC_SEGMENT {
                    point: pt(*end_x, *end_y),
                    size: D2D_SIZE_F {
                        width: *r,
                        height: *r,
                    },
                    rotationAngle: 0.0,
                    sweepDirection: if *sweep_cw {
                        D2D1_SWEEP_DIRECTION_CLOCKWISE
                    } else {
                        D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE
                    },
                    arcSize: D2D1_ARC_SIZE_SMALL,
                });
                sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                sink.Close()?;

                solid_brush.SetColor(&rgba_d2d(*color));
                ctx.FillGeometry(&geom, solid_brush, None);
            }
            DrawCmd::Spotlight {
                cx,
                cy,
                r,
                screen_w,
                screen_h,
                dim_alpha,
                ring_color,
            } => {
                // Draw dim screen with an exact radial cutout around (cx, cy)!
                // Using an inverted radial gradient (transparent inside r*0.55 -> dim outside r)
                // achieves the exact same visual as destination-out without touching underlying trail pixels!
                let max_r = screen_w.hypot(*screen_h).max(*r + 1.0);
                let stop_inner = ((*r * 0.55) / max_r).clamp(0.0, 0.99);
                let stop_outer = (*r / max_r).clamp(stop_inner + 0.001, 1.0);
                let d2d_stops = [
                    D2D1_GRADIENT_STOP {
                        position: 0.0,
                        color: D2D1_COLOR_F {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        },
                    },
                    D2D1_GRADIENT_STOP {
                        position: stop_inner,
                        color: D2D1_COLOR_F {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        },
                    },
                    D2D1_GRADIENT_STOP {
                        position: stop_outer,
                        color: D2D1_COLOR_F {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: *dim_alpha,
                        },
                    },
                    D2D1_GRADIENT_STOP {
                        position: 1.0,
                        color: D2D1_COLOR_F {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: *dim_alpha,
                        },
                    },
                ];
                let col = ctx.CreateGradientStopCollection(
                    &d2d_stops,
                    D2D1_COLOR_SPACE_SRGB,
                    D2D1_COLOR_SPACE_SRGB,
                    D2D1_BUFFER_PRECISION_8BPC_UNORM,
                    D2D1_EXTEND_MODE_CLAMP,
                    D2D1_COLOR_INTERPOLATION_MODE_STRAIGHT,
                )?;
                let brush = ctx.CreateRadialGradientBrush(
                    &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                        center: pt(*cx, *cy),
                        gradientOriginOffset: pt(0.0, 0.0),
                        radiusX: max_r,
                        radiusY: max_r,
                    },
                    None,
                    &col,
                )?;
                ctx.FillRectangle(
                    &D2D_RECT_F {
                        left: 0.0,
                        top: 0.0,
                        right: *screen_w,
                        bottom: *screen_h,
                    },
                    &brush,
                );

                // Subtle ring edge
                solid_brush.SetColor(&rgba_d2d(*ring_color));
                ctx.DrawEllipse(
                    &D2D1_ELLIPSE {
                        point: pt(*cx, *cy),
                        radiusX: *r * 0.78,
                        radiusY: *r * 0.78,
                    },
                    solid_brush,
                    1.5,
                    None,
                );
            }
        }
    }

    if in_plus_mode {
        ctx.SetPrimitiveBlend(D2D1_PRIMITIVE_BLEND_SOURCE_OVER);
    }

    Ok(())
}
