//! Complete settings, schema, 12 preset looks, 16 preset cursors, and 12 saved slots.
//! 1-to-1 port of `manifest.json` and `Schema.js` for Winscroller Mouse.

use crate::colors::ColorMode;
use serde::{Deserialize, Serialize};

pub const MINE_THEME: &str = "*winscroller";
pub const SLOTS: usize = 12;
pub const PALETTE_PRESETS: &[&str] = &[
    "#7aa2f7", "#bb9af7", "#7dcfff", "#9ece6a", "#e0af68", "#ff9e64", "#f7768e", "#c0caf5",
];

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TrailKind {
    None,
    #[default]
    Comet,
    Beads,
    Ribbon,
    Sparks,
    Echoes,
    Bubbles,
}

impl TrailKind {
    pub const ALL: &'static [TrailKind] = &[
        TrailKind::None,
        TrailKind::Comet,
        TrailKind::Beads,
        TrailKind::Ribbon,
        TrailKind::Sparks,
        TrailKind::Echoes,
        TrailKind::Bubbles,
    ];
    pub fn key(self) -> &'static str {
        match self {
            TrailKind::None => "none",
            TrailKind::Comet => "comet",
            TrailKind::Beads => "beads",
            TrailKind::Ribbon => "ribbon",
            TrailKind::Sparks => "sparks",
            TrailKind::Echoes => "echoes",
            TrailKind::Bubbles => "bubbles",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            TrailKind::None => "None",
            TrailKind::Comet => "Comet",
            TrailKind::Beads => "Beads",
            TrailKind::Ribbon => "Ribbon",
            TrailKind::Sparks => "Sparks",
            TrailKind::Echoes => "Echoes",
            TrailKind::Bubbles => "Bubbles",
        }
    }
    pub fn about(self) -> &'static str {
        match self {
            TrailKind::None => "No trail.",
            TrailKind::Comet => {
                "A tail that tapers away behind the pointer, brightest at its head."
            }
            TrailKind::Beads => "Evenly spaced dots along the path, shrinking toward the end.",
            TrailKind::Ribbon => "A smooth glowing band that follows every curve.",
            TrailKind::Sparks => "Sparks fly off the pointer as it moves, then fall and burn out.",
            TrailKind::Echoes => "Rings left along the path, widening as they fade.",
            TrailKind::Bubbles => "Bubbles let go along the path, swaying as they float up.",
        }
    }
    pub fn from_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(TrailKind::None),
            "comet" => Some(TrailKind::Comet),
            "beads" => Some(TrailKind::Beads),
            "ribbon" => Some(TrailKind::Ribbon),
            "sparks" => Some(TrailKind::Sparks),
            "echoes" => Some(TrailKind::Echoes),
            "bubbles" => Some(TrailKind::Bubbles),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HaloKind {
    None,
    #[default]
    Glow,
    Ring,
    Pulse,
    Spotlight,
    Crosshair,
    Target,
}

impl HaloKind {
    pub const ALL: &'static [HaloKind] = &[
        HaloKind::None,
        HaloKind::Glow,
        HaloKind::Ring,
        HaloKind::Pulse,
        HaloKind::Spotlight,
        HaloKind::Crosshair,
        HaloKind::Target,
    ];
    pub fn key(self) -> &'static str {
        match self {
            HaloKind::None => "none",
            HaloKind::Glow => "glow",
            HaloKind::Ring => "ring",
            HaloKind::Pulse => "pulse",
            HaloKind::Spotlight => "spotlight",
            HaloKind::Crosshair => "crosshair",
            HaloKind::Target => "target",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            HaloKind::None => "None",
            HaloKind::Glow => "Glow",
            HaloKind::Ring => "Ring",
            HaloKind::Pulse => "Pulse",
            HaloKind::Spotlight => "Spotlight",
            HaloKind::Crosshair => "Crosshair",
            HaloKind::Target => "Target",
        }
    }
    pub fn about(self) -> &'static str {
        match self {
            HaloKind::None => "No halo.",
            HaloKind::Glow => "A soft light around the pointer.",
            HaloKind::Ring => "A thin ring with a faint fill, like a touch indicator.",
            HaloKind::Pulse => "A breathing ring that sends out a wave now and then.",
            HaloKind::Spotlight => "Dims the screen except a soft circle around the pointer.",
            HaloKind::Crosshair => {
                "Thin lines across the screen through the pointer, for lining things up."
            }
            HaloKind::Target => "Four corner brackets around the pointer, turning slowly.",
        }
    }
    pub fn from_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(HaloKind::None),
            "glow" => Some(HaloKind::Glow),
            "ring" => Some(HaloKind::Ring),
            "pulse" => Some(HaloKind::Pulse),
            "spotlight" => Some(HaloKind::Spotlight),
            "crosshair" => Some(HaloKind::Crosshair),
            "target" => Some(HaloKind::Target),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HaloShow {
    #[default]
    Always,
    Moving,
    Still,
}

impl HaloShow {
    pub const ALL: &'static [HaloShow] = &[HaloShow::Always, HaloShow::Moving, HaloShow::Still];
    pub fn key(self) -> &'static str {
        match self {
            HaloShow::Always => "always",
            HaloShow::Moving => "moving",
            HaloShow::Still => "still",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            HaloShow::Always => "Always",
            HaloShow::Moving => "While moving",
            HaloShow::Still => "When still",
        }
    }
    pub fn from_key(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "always" => Some(HaloShow::Always),
            "moving" => Some(HaloShow::Moving),
            "still" => Some(HaloShow::Still),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    // Trail
    pub trail: TrailKind,
    pub trail_life: u32,     // 15..150 (hundredths of a second)
    pub trail_size: u32,     // 2..30
    pub trail_strength: u32, // 10..100

    // Halo
    pub halo: HaloKind,
    pub halo_size: u32,     // 20..240
    pub halo_strength: u32, // 10..100
    pub halo_show: HaloShow,

    // Colors
    pub color_mode: ColorMode,
    pub custom_color: String,
    pub custom_color2: String,

    // Panel styling
    pub panel_color: String, // theme | palette | borders | custom | monochrome
    pub panel_bg: String,    // #141414
    pub panel_fg: String,    // #e0e0e0
    pub panel_custom_color: String, // #ff6a1f
    pub panel_text: String,  // auto | theme | custom
    pub panel_frame: bool,
    pub panel_frame_gradient: bool,
    pub panel_frame_color: String, // accent | first | middle | last
    pub panel_tint: u32,           // 0..30
    pub panel_opacity: u32,        // 50..100

    // Cursor theme & designer
    pub pointer_theme: String, // "" (system default), "*winscroller" (made here), or Windows scheme name
    pub pointer_size: u32,     // 16..96
    pub cursor_shape: String,  // 14 arrows
    pub cursor_tilt: i32,      // -30..30
    pub cursor_round: u32,     // 0..100
    pub cursor_surface: String, // 30 surfaces
    pub cursor_fill: String,   // solid | gradient | glass
    pub cursor_gradient_to: String, // second | lighter | darker | accent | palette | border
    pub cursor_gradient_dir: String, // along | across | diagonal | theme
    pub cursor_pattern: String, // none | stripes | dots | grid
    pub cursor_depth: String,  // flat | raised | sunken
    pub cursor_opacity: u32,   // 30..100
    pub cursor_tint: u32,      // 0..40
    pub cursor_color: String, // accent | foreground | background | border | border2 | custom | white | black
    pub cursor_color_custom: String,
    pub cursor_color2: String,
    pub cursor_color2_custom: String,
    pub cursor_line: String,       // 22 lines
    pub cursor_line_color: String, // auto | subtle | ...
    pub cursor_line_custom: String,
    pub cursor_line_width: u32, // 1..4
    pub cursor_line_gradient: bool,
    pub cursor_line_fade_to: String, // lighter | second | accent | palette | border
    pub cursor_line_fade_dir: String, // along | diagonal | across | theme
    pub cursor_ornament: String,     // 18 ornaments
    pub cursor_ornament_size: u32,   // 3..16
    pub cursor_ornaments_on: String, // arrow | all
    pub cursor_cap: String,          // none | tip | tail | both
    pub cursor_detail: String,       // none | tip | spine | spark
    pub cursor_shadow: String,       // none | soft | strong | hard | long
    pub cursor_shadow_color: String, // dark | accent
    pub cursor_shadow_dir: String,   // below | around
    pub cursor_shadow_strength: u32, // 10..100
    pub cursor_hand: String,         // hand | arrow | ring
    pub cursor_beam: String,         // beam | slim | caret
    pub cursor_busy: String,         // ring | dots | orbit
    pub cursor_busy_speed: String,   // slow | normal | fast

    // Hiding
    pub hide_after: String,  // keep | 0 | 2 | 5 | 10 | 30
    pub hide_typing: String, // keep | on | off

    // Find
    pub shake: bool,
    pub shake_sensitivity: u32, // 1..10
    pub find_size: u32,         // 30..160

    // Behavior
    pub enabled: bool,
    pub fullscreen: String, // hide | show
    pub fps: String,        // "30" | "60" | "120" | "auto"
    pub wheel_looks: bool,
    pub start_look: String, // keep | 1..12
    pub start_with_windows: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            trail: TrailKind::Comet,
            trail_life: 45,
            trail_size: 8,
            trail_strength: 85,
            halo: HaloKind::Glow,
            halo_size: 64,
            halo_strength: 70,
            halo_show: HaloShow::Always,
            color_mode: ColorMode::Theme,
            custom_color: "#7aa2f7".into(),
            custom_color2: "#f7768e".into(),
            panel_color: "theme".into(),
            panel_bg: "#141414".into(),
            panel_fg: "#e0e0e0".into(),
            panel_custom_color: "#ff6a1f".into(),
            panel_text: "auto".into(),
            panel_frame: false,
            panel_frame_gradient: true,
            panel_frame_color: "accent".into(),
            panel_tint: 0,
            panel_opacity: 97,
            pointer_theme: String::new(),
            pointer_size: 32,
            cursor_shape: "classic".into(),
            cursor_tilt: 0,
            cursor_round: 30,
            cursor_surface: "filled".into(),
            cursor_fill: "solid".into(),
            cursor_gradient_to: "second".into(),
            cursor_gradient_dir: "along".into(),
            cursor_pattern: "none".into(),
            cursor_depth: "flat".into(),
            cursor_opacity: 100,
            cursor_tint: 0,
            cursor_color: "foreground".into(),
            cursor_color_custom: "#7aa2f7".into(),
            cursor_color2: "accent".into(),
            cursor_color2_custom: "#f7768e".into(),
            cursor_line: "solid".into(),
            cursor_line_color: "auto".into(),
            cursor_line_custom: "#101010".into(),
            cursor_line_width: 2,
            cursor_line_gradient: false,
            cursor_line_fade_to: "lighter".into(),
            cursor_line_fade_dir: "along".into(),
            cursor_ornament: "none".into(),
            cursor_ornament_size: 7,
            cursor_ornaments_on: "arrow".into(),
            cursor_cap: "none".into(),
            cursor_detail: "none".into(),
            cursor_shadow: "soft".into(),
            cursor_shadow_color: "dark".into(),
            cursor_shadow_dir: "below".into(),
            cursor_shadow_strength: 60,
            cursor_hand: "hand".into(),
            cursor_beam: "beam".into(),
            cursor_busy: "ring".into(),
            cursor_busy_speed: "normal".into(),
            hide_after: "keep".into(),
            hide_typing: "keep".into(),
            shake: true,
            shake_sensitivity: 5,
            find_size: 60,
            enabled: true,
            fullscreen: "hide".into(),
            fps: "60".into(),
            wheel_looks: true,
            start_look: "keep".into(),
            start_with_windows: false,
        }
    }
}

impl Settings {
    pub fn trail_life_secs(&self) -> f32 {
        (self.trail_life.clamp(15, 150) as f32) / 100.0
    }
    pub fn trail_strength_f(&self) -> f32 {
        (self.trail_strength.clamp(10, 100) as f32) / 100.0
    }
    pub fn halo_strength_f(&self) -> f32 {
        (self.halo_strength.clamp(10, 100) as f32) / 100.0
    }
    pub fn find_size_f(&self) -> f32 {
        self.find_size.clamp(30, 160) as f32
    }
    pub fn target_fps(&self, monitor_hz: u32) -> u32 {
        match self.fps.as_str() {
            "30" => 30,
            "60" => 60,
            "120" => 120,
            "auto" => monitor_hz.clamp(60, 240),
            _ => 60,
        }
    }
    pub fn hide_after_secs(&self) -> Option<f32> {
        match self.hide_after.as_str() {
            "2" => Some(2.0),
            "5" => Some(5.0),
            "10" => Some(10.0),
            "30" => Some(30.0),
            _ => None,
        }
    }
    pub fn is_custom_cursor(&self) -> bool {
        self.pointer_theme == MINE_THEME || self.pointer_theme == "*plume"
    }

    /// Export only the look keys (trail, halo, colors) as a JSON string.
    pub fn export_look_json(&self) -> String {
        let look = LookData::from_settings(self);
        serde_json::to_string(&look).unwrap_or_default()
    }

    /// Import look keys from JSON string.
    pub fn import_look_json(&mut self, text: &str) -> bool {
        if let Ok(look) = serde_json::from_str::<LookData>(text.trim()) {
            look.apply(self);
            true
        } else {
            false
        }
    }

    /// Set a single setting by its exact omarchy-plume key name (for CLI `set <key> <value>`).
    pub fn set_by_key(&mut self, key: &str, val: &str) -> Result<(), String> {
        let parse_bool = |v: &str| match v.to_lowercase().as_str() {
            "true" | "1" | "on" | "yes" => Ok(true),
            "false" | "0" | "off" | "no" => Ok(false),
            _ => Err(format!("invalid boolean: {}", v)),
        };
        let parse_u32 = |v: &str, min: u32, max: u32| -> Result<u32, String> {
            let n: f32 = v.parse().map_err(|_| format!("invalid integer: {}", v))?;
            Ok((n.round() as i64).clamp(min as i64, max as i64) as u32)
        };
        let parse_i32 = |v: &str, min: i32, max: i32| -> Result<i32, String> {
            let n: f32 = v.parse().map_err(|_| format!("invalid integer: {}", v))?;
            Ok((n.round() as i32).clamp(min, max))
        };

        match key {
            "trail" => {
                self.trail =
                    TrailKind::from_key(val).ok_or_else(|| format!("unknown trail: {}", val))?
            }
            "trailLife" => self.trail_life = parse_u32(val, 15, 150)?,
            "trailSize" => self.trail_size = parse_u32(val, 2, 30)?,
            "trailStrength" => self.trail_strength = parse_u32(val, 10, 100)?,
            "halo" => {
                self.halo =
                    HaloKind::from_key(val).ok_or_else(|| format!("unknown halo: {}", val))?
            }
            "haloSize" => self.halo_size = parse_u32(val, 20, 240)?,
            "haloStrength" => self.halo_strength = parse_u32(val, 10, 100)?,
            "haloShow" => {
                self.halo_show =
                    HaloShow::from_key(val).ok_or_else(|| format!("unknown haloShow: {}", val))?
            }
            "colorMode" => {
                self.color_mode =
                    ColorMode::from_key(val).ok_or_else(|| format!("unknown colorMode: {}", val))?
            }
            "customColor" => self.custom_color = val.to_string(),
            "customColor2" => self.custom_color2 = val.to_string(),
            "panelColor" => self.panel_color = val.to_string(),
            "panelBg" => self.panel_bg = val.to_string(),
            "panelFg" => self.panel_fg = val.to_string(),
            "panelCustomColor" => self.panel_custom_color = val.to_string(),
            "panelText" => self.panel_text = val.to_string(),
            "panelFrame" => self.panel_frame = parse_bool(val)?,
            "panelFrameGradient" => self.panel_frame_gradient = parse_bool(val)?,
            "panelFrameColor" => self.panel_frame_color = val.to_string(),
            "panelTint" => self.panel_tint = parse_u32(val, 0, 30)?,
            "panelOpacity" => self.panel_opacity = parse_u32(val, 50, 100)?,
            "pointerTheme" => {
                self.pointer_theme = if val == "*plume" {
                    MINE_THEME.into()
                } else {
                    val.to_string()
                }
            }
            "pointerSize" => self.pointer_size = parse_u32(val, 16, 96)?,
            "cursorShape" => self.cursor_shape = val.to_string(),
            "cursorTilt" => self.cursor_tilt = parse_i32(val, -30, 30)?,
            "cursorRound" => self.cursor_round = parse_u32(val, 0, 100)?,
            "cursorSurface" => self.cursor_surface = val.to_string(),
            "cursorFill" => self.cursor_fill = val.to_string(),
            "cursorGradientTo" => self.cursor_gradient_to = val.to_string(),
            "cursorGradientDir" => self.cursor_gradient_dir = val.to_string(),
            "cursorPattern" => self.cursor_pattern = val.to_string(),
            "cursorDepth" => self.cursor_depth = val.to_string(),
            "cursorOpacity" => self.cursor_opacity = parse_u32(val, 30, 100)?,
            "cursorTint" => self.cursor_tint = parse_u32(val, 0, 40)?,
            "cursorColor" => self.cursor_color = val.to_string(),
            "cursorColorCustom" => self.cursor_color_custom = val.to_string(),
            "cursorColor2" => self.cursor_color2 = val.to_string(),
            "cursorColor2Custom" => self.cursor_color2_custom = val.to_string(),
            "cursorLine" => self.cursor_line = val.to_string(),
            "cursorLineColor" => self.cursor_line_color = val.to_string(),
            "cursorLineCustom" => self.cursor_line_custom = val.to_string(),
            "cursorLineWidth" => self.cursor_line_width = parse_u32(val, 1, 4)?,
            "cursorLineGradient" => self.cursor_line_gradient = parse_bool(val)?,
            "cursorLineFadeTo" => self.cursor_line_fade_to = val.to_string(),
            "cursorLineFadeDir" => self.cursor_line_fade_dir = val.to_string(),
            "cursorOrnament" => self.cursor_ornament = val.to_string(),
            "cursorOrnamentSize" => self.cursor_ornament_size = parse_u32(val, 3, 16)?,
            "cursorOrnamentsOn" => self.cursor_ornaments_on = val.to_string(),
            "cursorCap" => self.cursor_cap = val.to_string(),
            "cursorDetail" => self.cursor_detail = val.to_string(),
            "cursorShadow" => self.cursor_shadow = val.to_string(),
            "cursorShadowColor" => self.cursor_shadow_color = val.to_string(),
            "cursorShadowDir" => self.cursor_shadow_dir = val.to_string(),
            "cursorShadowStrength" => self.cursor_shadow_strength = parse_u32(val, 10, 100)?,
            "cursorHand" => self.cursor_hand = val.to_string(),
            "cursorBeam" => self.cursor_beam = val.to_string(),
            "cursorBusy" => self.cursor_busy = val.to_string(),
            "cursorBusySpeed" => self.cursor_busy_speed = val.to_string(),
            "hideAfter" => self.hide_after = val.to_string(),
            "hideTyping" => self.hide_typing = val.to_string(),
            "shake" => self.shake = parse_bool(val)?,
            "shakeSensitivity" => self.shake_sensitivity = parse_u32(val, 1, 10)?,
            "findSize" => self.find_size = parse_u32(val, 30, 160)?,
            "enabled" => self.enabled = parse_bool(val)?,
            "fullscreen" => self.fullscreen = val.to_string(),
            "fps" => self.fps = val.to_string(),
            "wheelLooks" => self.wheel_looks = parse_bool(val)?,
            "startLook" => self.start_look = val.to_string(),
            "startWithWindows" => self.start_with_windows = parse_bool(val)?,
            _ => return Err(format!("unknown setting key: {}", key)),
        }
        Ok(())
    }

    /// Returns a fast hash fingerprint of all cursor-builder fields so we only rebuild `.cur`/`.ani` when changed.
    pub fn cursor_fingerprint(
        &self,
        accent_hex: &str,
        fg_hex: &str,
        bg_hex: &str,
        border_hex: &str,
    ) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.pointer_theme.hash(&mut h);
        self.pointer_size.hash(&mut h);
        self.cursor_shape.hash(&mut h);
        self.cursor_tilt.hash(&mut h);
        self.cursor_round.hash(&mut h);
        self.cursor_surface.hash(&mut h);
        self.cursor_fill.hash(&mut h);
        self.cursor_gradient_to.hash(&mut h);
        self.cursor_gradient_dir.hash(&mut h);
        self.cursor_pattern.hash(&mut h);
        self.cursor_depth.hash(&mut h);
        self.cursor_opacity.hash(&mut h);
        self.cursor_tint.hash(&mut h);
        self.cursor_color.hash(&mut h);
        self.cursor_color_custom.hash(&mut h);
        self.cursor_color2.hash(&mut h);
        self.cursor_color2_custom.hash(&mut h);
        self.cursor_line.hash(&mut h);
        self.cursor_line_color.hash(&mut h);
        self.cursor_line_custom.hash(&mut h);
        self.cursor_line_width.hash(&mut h);
        self.cursor_line_gradient.hash(&mut h);
        self.cursor_line_fade_to.hash(&mut h);
        self.cursor_line_fade_dir.hash(&mut h);
        self.cursor_ornament.hash(&mut h);
        self.cursor_ornament_size.hash(&mut h);
        self.cursor_ornaments_on.hash(&mut h);
        self.cursor_cap.hash(&mut h);
        self.cursor_detail.hash(&mut h);
        self.cursor_shadow.hash(&mut h);
        self.cursor_shadow_color.hash(&mut h);
        self.cursor_shadow_dir.hash(&mut h);
        self.cursor_shadow_strength.hash(&mut h);
        self.cursor_hand.hash(&mut h);
        self.cursor_beam.hash(&mut h);
        self.cursor_busy.hash(&mut h);
        self.cursor_busy_speed.hash(&mut h);
        accent_hex.hash(&mut h);
        fg_hex.hash(&mut h);
        bg_hex.hash(&mut h);
        border_hex.hash(&mut h);
        h.finish()
    }
}

/// Persisted document on disk: current settings + 12 saved slots.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AppStateFile {
    pub settings: Settings,
    pub slots: Vec<Option<Settings>>,
}

impl AppStateFile {
    pub fn ensure_slots(&mut self) {
        if self.slots.len() < SLOTS {
            self.slots.resize(SLOTS, None);
        } else if self.slots.len() > SLOTS {
            self.slots.truncate(SLOTS);
        }
    }
}

/// Just the look subset (trail + halo + colors) for export/import & ready-made looks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LookData {
    pub trail: TrailKind,
    pub trail_life: u32,
    pub trail_size: u32,
    pub trail_strength: u32,
    pub halo: HaloKind,
    pub halo_size: u32,
    pub halo_strength: u32,
    pub halo_show: HaloShow,
    pub color_mode: ColorMode,
    pub custom_color: String,
    pub custom_color2: String,
}

impl Default for LookData {
    fn default() -> Self {
        Self::from_settings(&Settings::default())
    }
}

impl LookData {
    pub fn from_settings(s: &Settings) -> Self {
        Self {
            trail: s.trail,
            trail_life: s.trail_life,
            trail_size: s.trail_size,
            trail_strength: s.trail_strength,
            halo: s.halo,
            halo_size: s.halo_size,
            halo_strength: s.halo_strength,
            halo_show: s.halo_show,
            color_mode: s.color_mode,
            custom_color: s.custom_color.clone(),
            custom_color2: s.custom_color2.clone(),
        }
    }

    pub fn apply(&self, s: &mut Settings) {
        s.trail = self.trail;
        s.trail_life = self.trail_life;
        s.trail_size = self.trail_size;
        s.trail_strength = self.trail_strength;
        s.halo = self.halo;
        s.halo_size = self.halo_size;
        s.halo_strength = self.halo_strength;
        s.halo_show = self.halo_show;
        s.color_mode = self.color_mode;
        s.custom_color = self.custom_color.clone();
        s.custom_color2 = self.custom_color2.clone();
    }
}

pub struct BuiltinLook {
    pub key: &'static str,
    pub label: &'static str,
    pub data: LookData,
}

/// Exact 12 ready-made looks from `Schema.js` lines 329–344.
pub fn builtin_looks() -> Vec<BuiltinLook> {
    let d = LookData::default();
    vec![
        BuiltinLook {
            key: "classic",
            label: "Classic",
            data: d.clone(),
        },
        BuiltinLook {
            key: "neon",
            label: "Neon",
            data: LookData {
                trail: TrailKind::Ribbon,
                trail_size: 6,
                halo: HaloKind::Ring,
                halo_size: 44,
                color_mode: ColorMode::Rainbow,
                ..d.clone()
            },
        },
        BuiltinLook {
            key: "ember",
            label: "Ember",
            data: LookData {
                trail: TrailKind::Sparks,
                trail_life: 70,
                trail_size: 10,
                halo: HaloKind::Glow,
                halo_size: 56,
                color_mode: ColorMode::Speed,
                ..d.clone()
            },
        },
        BuiltinLook {
            key: "bubbles",
            label: "Bubbles",
            data: LookData {
                trail: TrailKind::Bubbles,
                trail_life: 110,
                trail_size: 12,
                halo: HaloKind::None,
                color_mode: ColorMode::Palette,
                ..d.clone()
            },
        },
        BuiltinLook {
            key: "echo",
            label: "Echo",
            data: LookData {
                trail: TrailKind::Echoes,
                trail_life: 60,
                trail_size: 10,
                halo: HaloKind::Pulse,
                halo_size: 48,
                color_mode: ColorMode::Border,
                ..d.clone()
            },
        },
        BuiltinLook {
            key: "beads",
            label: "Beads",
            data: LookData {
                trail: TrailKind::Beads,
                trail_life: 60,
                trail_size: 7,
                halo: HaloKind::Ring,
                halo_size: 36,
                color_mode: ColorMode::Gradient,
                custom_color: "#7dcfff".into(),
                custom_color2: "#bb9af7".into(),
                ..d.clone()
            },
        },
        BuiltinLook {
            key: "aurora",
            label: "Aurora",
            data: LookData {
                trail: TrailKind::Ribbon,
                trail_life: 80,
                trail_size: 12,
                trail_strength: 70,
                halo: HaloKind::Glow,
                halo_size: 90,
                color_mode: ColorMode::Border,
                ..d.clone()
            },
        },
        BuiltinLook {
            key: "drift",
            label: "Drift",
            data: LookData {
                trail: TrailKind::Comet,
                trail_life: 70,
                trail_size: 10,
                halo: HaloKind::Glow,
                color_mode: ColorMode::Fade,
                ..d.clone()
            },
        },
        BuiltinLook {
            key: "presenter",
            label: "Presenter",
            data: LookData {
                trail: TrailKind::None,
                halo: HaloKind::Spotlight,
                halo_size: 120,
                halo_strength: 75,
                halo_show: HaloShow::Still,
                ..d.clone()
            },
        },
        BuiltinLook {
            key: "sniper",
            label: "Crosshair",
            data: LookData {
                trail: TrailKind::None,
                halo: HaloKind::Crosshair,
                halo_size: 40,
                color_mode: ColorMode::Custom,
                custom_color: "#f7768e".into(),
                ..d.clone()
            },
        },
        BuiltinLook {
            key: "target",
            label: "Target",
            data: LookData {
                trail: TrailKind::Comet,
                trail_life: 25,
                trail_size: 4,
                halo: HaloKind::Target,
                halo_size: 54,
                ..d.clone()
            },
        },
        BuiltinLook {
            key: "minimal",
            label: "Minimal",
            data: LookData {
                trail: TrailKind::None,
                halo: HaloKind::Ring,
                halo_size: 30,
                halo_strength: 60,
                ..d
            },
        },
    ]
}

pub struct BuiltinCursor {
    pub key: &'static str,
    pub label: &'static str,
    pub apply: fn(&mut Settings),
}

fn reset_cursor_defaults(s: &mut Settings) {
    let d = Settings::default();
    s.pointer_theme = MINE_THEME.into();
    s.cursor_shape = d.cursor_shape;
    s.cursor_tilt = d.cursor_tilt;
    s.cursor_round = d.cursor_round;
    s.cursor_surface = d.cursor_surface;
    s.cursor_fill = d.cursor_fill;
    s.cursor_gradient_to = d.cursor_gradient_to;
    s.cursor_gradient_dir = d.cursor_gradient_dir;
    s.cursor_pattern = d.cursor_pattern;
    s.cursor_depth = d.cursor_depth;
    s.cursor_opacity = d.cursor_opacity;
    s.cursor_tint = d.cursor_tint;
    s.cursor_color = d.cursor_color;
    s.cursor_color_custom = d.cursor_color_custom;
    s.cursor_color2 = d.cursor_color2;
    s.cursor_color2_custom = d.cursor_color2_custom;
    s.cursor_line = d.cursor_line;
    s.cursor_line_color = d.cursor_line_color;
    s.cursor_line_custom = d.cursor_line_custom;
    s.cursor_line_width = d.cursor_line_width;
    s.cursor_line_gradient = d.cursor_line_gradient;
    s.cursor_line_fade_to = d.cursor_line_fade_to;
    s.cursor_line_fade_dir = d.cursor_line_fade_dir;
    s.cursor_ornament = d.cursor_ornament;
    s.cursor_ornament_size = d.cursor_ornament_size;
    s.cursor_ornaments_on = d.cursor_ornaments_on;
    s.cursor_cap = d.cursor_cap;
    s.cursor_detail = d.cursor_detail;
    s.cursor_shadow = d.cursor_shadow;
    s.cursor_shadow_color = d.cursor_shadow_color;
    s.cursor_shadow_dir = d.cursor_shadow_dir;
    s.cursor_shadow_strength = d.cursor_shadow_strength;
    s.cursor_hand = d.cursor_hand;
    s.cursor_beam = d.cursor_beam;
    s.cursor_busy = d.cursor_busy;
    s.cursor_busy_speed = d.cursor_busy_speed;
}

/// Exact 16 ready-made cursors from `Schema.js` lines 393–427.
pub fn builtin_cursors() -> Vec<BuiltinCursor> {
    vec![
        BuiltinCursor {
            key: "plume",
            label: "Winscroller",
            apply: |s| {
                reset_cursor_defaults(s);
            },
        },
        BuiltinCursor {
            key: "ink",
            label: "Ink",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_color = "black".into();
                s.cursor_line_width = 1;
                s.cursor_line_color = "white".into();
            },
        },
        BuiltinCursor {
            key: "paper",
            label: "Paper",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_color = "white".into();
                s.cursor_line_width = 1;
                s.cursor_line_color = "black".into();
                s.cursor_shadow = "hard".into();
                s.cursor_shadow_strength = 45;
                s.cursor_round = 10;
            },
        },
        BuiltinCursor {
            key: "accent",
            label: "Accent",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_shape = "slim".into();
                s.cursor_fill = "gradient".into();
                s.cursor_color = "accent".into();
                s.cursor_gradient_to = "lighter".into();
                s.cursor_line_width = 1;
                s.cursor_line_color = "background".into();
                s.cursor_shadow_color = "accent".into();
                s.cursor_shadow_strength = 50;
            },
        },
        BuiltinCursor {
            key: "neon",
            label: "Neon",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_shape = "needle".into();
                s.cursor_surface = "clear".into();
                s.cursor_color = "accent".into();
                s.cursor_line = "neon".into();
                s.cursor_line_color = "accent".into();
                s.cursor_shadow_color = "accent".into();
                s.cursor_shadow_strength = 70;
                s.cursor_hand = "ring".into();
                s.cursor_beam = "caret".into();
                s.cursor_busy = "orbit".into();
            },
        },
        BuiltinCursor {
            key: "glass",
            label: "Glass",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_shape = "teardrop".into();
                s.cursor_fill = "glass".into();
                s.cursor_color = "accent".into();
                s.cursor_line_width = 1;
                s.cursor_round = 40;
            },
        },
        BuiltinCursor {
            key: "candy",
            label: "Candy",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_shape = "chunky".into();
                s.cursor_surface = "candy".into();
                s.cursor_color = "border".into();
                s.cursor_color2 = "border2".into();
                s.cursor_line_width = 3;
                s.cursor_line_color = "background".into();
                s.cursor_shadow = "long".into();
                s.cursor_shadow_strength = 50;
                s.cursor_round = 60;
            },
        },
        BuiltinCursor {
            key: "gilded",
            label: "Gilded",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_shape = "crest".into();
                s.cursor_surface = "velvet".into();
                s.cursor_color = "background".into();
                s.cursor_color2 = "accent".into();
                s.cursor_line = "gilded".into();
                s.cursor_line_color = "accent".into();
                s.cursor_line_width = 2;
                s.cursor_ornament = "studs".into();
                s.cursor_ornament_size = 6;
                s.cursor_round = 20;
            },
        },
        BuiltinCursor {
            key: "marble",
            label: "Marble",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_shape = "wedge".into();
                s.cursor_surface = "marble".into();
                s.cursor_color = "foreground".into();
                s.cursor_color2 = "accent".into();
                s.cursor_line_color = "background".into();
                s.cursor_depth = "raised".into();
                s.cursor_round = 45;
            },
        },
        BuiltinCursor {
            key: "nebula",
            label: "Nebula",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_shape = "leaf".into();
                s.cursor_surface = "nebula".into();
                s.cursor_color = "border".into();
                s.cursor_color2 = "border2".into();
                s.cursor_line = "glow".into();
                s.cursor_line_color = "foreground".into();
                s.cursor_line_width = 1;
                s.cursor_shadow_color = "accent".into();
            },
        },
        BuiltinCursor {
            key: "stitch",
            label: "Stitched",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_shape = "feather".into();
                s.cursor_surface = "leather".into();
                s.cursor_color = "accent".into();
                s.cursor_line = "stitched".into();
                s.cursor_line_color = "background".into();
                s.cursor_shadow = "hard".into();
                s.cursor_shadow_strength = 50;
            },
        },
        BuiltinCursor {
            key: "dot",
            label: "Dot",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_shape = "dot".into();
                s.cursor_color = "accent".into();
                s.cursor_line_color = "background".into();
                s.cursor_shadow_color = "accent".into();
                s.cursor_hand = "ring".into();
                s.cursor_busy = "dots".into();
            },
        },
        BuiltinCursor {
            key: "sharp",
            label: "Sharp",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_round = 0;
                s.cursor_color = "white".into();
                s.cursor_line_width = 3;
                s.cursor_line_color = "black".into();
                s.cursor_shadow = "none".into();
                s.cursor_detail = "tip".into();
            },
        },
        BuiltinCursor {
            key: "glitch",
            label: "Glitch",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_shape = "chevron".into();
                s.cursor_surface = "crt".into();
                s.cursor_color = "background".into();
                s.cursor_line = "glitch".into();
                s.cursor_line_color = "foreground".into();
                s.cursor_line_width = 2;
                s.cursor_round = 10;
            },
        },
        BuiltinCursor {
            key: "terrazzo",
            label: "Terrazzo",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_shape = "bullet".into();
                s.cursor_surface = "terrazzo".into();
                s.cursor_color = "foreground".into();
                s.cursor_color2 = "accent".into();
                s.cursor_line = "beads".into();
                s.cursor_line_color = "accent".into();
                s.cursor_line_width = 1;
                s.cursor_round = 50;
            },
        },
        BuiltinCursor {
            key: "holo",
            label: "Hologram",
            apply: |s| {
                reset_cursor_defaults(s);
                s.cursor_shape = "kite".into();
                s.cursor_surface = "holo".into();
                s.cursor_color = "foreground".into();
                s.cursor_line = "double".into();
                s.cursor_line_color = "accent".into();
                s.cursor_ornament = "stars".into();
                s.cursor_ornament_size = 5;
            },
        },
    ]
}
