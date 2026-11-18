//! Color helpers and all 8 color modes — 1-to-1 port of `Ink.js`, `Colors.js`, and `colors/*.js`.

use serde::{Deserialize, Serialize};

pub type Rgb = [f32; 3];
pub type Rgba = [f32; 4];

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    #[default]
    Theme,
    Palette,
    Border,
    Custom,
    Gradient,
    Rainbow,
    Fade,
    Speed,
}

impl ColorMode {
    pub const ALL: &'static [ColorMode] = &[
        ColorMode::Theme,
        ColorMode::Palette,
        ColorMode::Border,
        ColorMode::Custom,
        ColorMode::Gradient,
        ColorMode::Rainbow,
        ColorMode::Fade,
        ColorMode::Speed,
    ];

    pub fn key(&self) -> &'static str {
        match self {
            ColorMode::Theme => "theme",
            ColorMode::Palette => "palette",
            ColorMode::Border => "border",
            ColorMode::Custom => "custom",
            ColorMode::Gradient => "gradient",
            ColorMode::Rainbow => "rainbow",
            ColorMode::Fade => "fade",
            ColorMode::Speed => "speed",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ColorMode::Theme => "Theme",
            ColorMode::Palette => "Theme palette",
            ColorMode::Border => "Window border",
            ColorMode::Custom => "Custom",
            ColorMode::Gradient => "Gradient",
            ColorMode::Rainbow => "Rainbow",
            ColorMode::Fade => "Drift",
            ColorMode::Speed => "Speed",
        }
    }

    pub fn about(&self) -> &'static str {
        match self {
            ColorMode::Theme => "Your Windows theme's accent color, all the way.",
            ColorMode::Palette => {
                "Runs through every color of your theme, from the pointer to the tail."
            }
            ColorMode::Border => "The colors of your active window border, as a gradient.",
            ColorMode::Custom => "One color of your choosing.",
            ColorMode::Gradient => {
                "From your first color at the pointer to your second at the end of the tail."
            }
            ColorMode::Rainbow => "Every hue along the tail, slowly turning.",
            ColorMode::Fade => "Starts on your accent and drifts slowly around the color wheel.",
            ColorMode::Speed => "Cool blue when the pointer moves slowly, hot red when it flies.",
        }
    }

    pub fn from_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "theme" => Some(ColorMode::Theme),
            "palette" => Some(ColorMode::Palette),
            "border" => Some(ColorMode::Border),
            "custom" => Some(ColorMode::Custom),
            "gradient" => Some(ColorMode::Gradient),
            "rainbow" => Some(ColorMode::Rainbow),
            "fade" | "drift" => Some(ColorMode::Fade),
            "speed" => Some(ColorMode::Speed),
            _ => None,
        }
    }
}

/// Runtime color context — updated from Windows theme + settings + pointer motion.
#[derive(Debug, Clone)]
pub struct ColorCtx {
    pub accent: Rgb,
    pub foreground: Rgb,
    pub background: Rgb,
    pub palette: Vec<Rgb>,
    pub border: Vec<Rgb>,
    pub border_angle: f32,
    pub custom: Rgb,
    pub custom2: Rgb,
    pub speed: f32,
    pub time: f64,
}

impl Default for ColorCtx {
    fn default() -> Self {
        Self {
            accent: [122.0, 162.0, 247.0],
            foreground: [224.0, 224.0, 224.0],
            background: [20.0, 20.0, 24.0],
            palette: vec![
                [122.0, 162.0, 247.0],
                [187.0, 154.0, 247.0],
                [125.0, 207.0, 255.0],
                [158.0, 206.0, 106.0],
                [224.0, 175.0, 104.0],
                [255.0, 158.0, 100.0],
                [247.0, 118.0, 142.0],
                [192.0, 202.0, 245.0],
            ],
            border: vec![[122.0, 162.0, 247.0], [187.0, 154.0, 247.0]],
            border_angle: 45.0,
            custom: [122.0, 162.0, 247.0],
            custom2: [247.0, 118.0, 142.0],
            speed: 0.0,
            time: 0.0,
        }
    }
}

/// Parse "#rrggbb" or "#rrggbbaa" into [r, g, b] (0..255).
pub fn parse_hex(s: &str) -> Option<Rgb> {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 || s.len() == 8 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()? as f32;
        let g = u8::from_str_radix(&s[2..4], 16).ok()? as f32;
        let b = u8::from_str_radix(&s[4..6], 16).ok()? as f32;
        Some([r, g, b])
    } else {
        None
    }
}

pub fn to_hex(c: Rgb) -> String {
    let r = c[0].round().clamp(0.0, 255.0) as u8;
    let g = c[1].round().clamp(0.0, 255.0) as u8;
    let b = c[2].round().clamp(0.0, 255.0) as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

pub fn mix_rgb(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let k = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * k,
        a[1] + (b[1] - a[1]) * k,
        a[2] + (b[2] - a[2]) * k,
    ]
}

/// HSL (h in 0..360, s & l in 0..1) -> RGB (0..255), exact match to `Ink.hsl`.
pub fn hsl_deg(h: f32, s: f32, l: f32) -> Rgb {
    let h = h.rem_euclid(360.0) / 360.0;
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let f = |mut t: f32| -> f32 {
        t = t.rem_euclid(1.0);
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    [
        f(h + 1.0 / 3.0) * 255.0,
        f(h) * 255.0,
        f(h - 1.0 / 3.0) * 255.0,
    ]
}

/// Hue in degrees (0..360) of an RGB (0..255) color, exact match to `Ink.hueOf`.
pub fn hue_of(c: Rgb) -> f32 {
    let r = c[0] / 255.0;
    let g = c[1] / 255.0;
    let b = c[2] / 255.0;
    let mx = r.max(g).max(b);
    let mn = r.min(g).min(b);
    let d = mx - mn;
    if d == 0.0 {
        return 0.0;
    }
    let h = if mx == r {
        ((g - b) / d) % 6.0
    } else if mx == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h * 60.0).rem_euclid(360.0)
}

pub fn lum(c: Rgb) -> f32 {
    (0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]) / 255.0
}

pub fn contrast(c: Rgb) -> Rgb {
    if lum(c) > 0.55 {
        [24.0, 20.0, 18.0]
    } else {
        [250.0, 247.0, 242.0]
    }
}

/// Interpolate smoothly along a list of colors, exact match to `Ink.along`.
pub fn along(list: &[Rgb], t: f32) -> Option<Rgb> {
    if list.is_empty() {
        return None;
    }
    if list.len() == 1 {
        return Some(list[0]);
    }
    let x = t.clamp(0.0, 1.0) * (list.len() - 1) as f32;
    let i = (x.floor() as usize).min(list.len() - 2);
    Some(mix_rgb(list[i], list[i + 1], x - i as f32))
}

/// Evaluate any of the 8 color modes at position `t` (0=pointer, 1=tail).
/// Exact 1-to-1 port of `colors/*.js`.
pub fn color_at(mode: ColorMode, t: f32, c: &ColorCtx) -> Rgb {
    match mode {
        ColorMode::Theme => c.accent,
        ColorMode::Palette => along(&c.palette, t).unwrap_or(c.accent),
        ColorMode::Border => along(&c.border, t).unwrap_or(c.accent),
        ColorMode::Custom => c.custom,
        ColorMode::Gradient => mix_rgb(c.custom, c.custom2, t),
        ColorMode::Rainbow => hsl_deg((c.time as f32) * 40.0 + t * 300.0, 0.85, 0.62),
        ColorMode::Fade => hsl_deg(
            hue_of(c.accent) + ((c.time as f32) * 0.25).sin() * 70.0 + t * 30.0,
            0.75,
            0.62,
        ),
        ColorMode::Speed => {
            let k = (c.speed * (1.0 - t * 0.5)).clamp(0.0, 1.0);
            hsl_deg(220.0 - 220.0 * k, 0.85, 0.60)
        }
    }
}

/// Convert RGB (0..255) + alpha (0..1) to normalized Rgba (0..1) for Direct2D.
pub fn to_rgba(rgb: Rgb, alpha: f32) -> Rgba {
    [
        (rgb[0] / 255.0).clamp(0.0, 1.0),
        (rgb[1] / 255.0).clamp(0.0, 1.0),
        (rgb[2] / 255.0).clamp(0.0, 1.0),
        alpha.clamp(0.0, 1.0),
    ]
}
