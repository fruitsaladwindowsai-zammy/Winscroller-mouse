//! Pure, OS-agnostic core library for Winscroller Mouse.
//! Contains pointer motion tracking, shake detection, color modes, settings schema,
//! 6 trails, 6 halos, per-frame draw command compositor, and the full SVG/CUR/ANI Cursor Forge.

pub mod colors;
pub mod cursor_forge;
pub mod draw;
pub mod halos;
pub mod motion;
pub mod paint;
pub mod settings;
pub mod trails;
