# Winscroller Mouse v1.0.1 — Release Notes

**Winscroller Mouse** is a native, GPU-accelerated Windows 10/11 clone of [omarchy-plume](https://github.com/qempexe/omarchy-plume) written in pure Rust — combining a full procedural **Custom Cursor Forge** with silky-smooth **DirectComposition pointer trails, halos, color modes, and Shake-to-Find**.

---

## What's New in v1.0.1

### Fixes & Enhancements
- **Fixed Fullscreen Auto-Hide False Positives**: Resolved an issue where maximized Windows applications (browsers, terminals, code editors) triggered fullscreen auto-hide due to Windows' 8-pixel off-screen border expansion and `QUNS_BUSY` Focus Assist notifications. Trails and halos now remain active over normal maximized windows and only auto-hide when exclusive Direct3D games are playing.
- **DirectComposition Layered Surface Alpha**: Explicitly configured `SetLayeredWindowAttributes` (alpha 255) and `SetWindowPos(HWND_TOPMOST)` during overlay creation, guaranteeing per-pixel alpha rendering and topmost Z-order across all Windows 10 and 11 builds.
- **Interactive First-Run Experience**: Double-clicking `winscroller-mouse.exe` directly launches the 8-tab Settings Panel with live interactive preview, allowing immediate configuration of looks, trails, halos, and custom cursors.
- **Tray Balloon Notifications**: Shows a friendly Windows notification balloon when the daemon starts.
- **Silent Daemon Flags**: Added `--silent` and `--daemon` for silent background execution and Windows startup registry integration.

---

## Highlights

- **Ultra-Light Native Binary**: Single statically linked `winscroller-mouse.exe` (`~1.8 MB`, zero VC++ runtime DLL dependencies, `~9.7 MB` private RAM, `0%` GPU / `<0.1%` CPU when idle).
- **DirectComposition + Direct2D Flip-Model Overlay**: True per-pixel alpha transparency across every monitor with `WS_EX_NOREDIRECTIONBITMAP`, additive Catmull-Rom smooth trails, and automatic `IDXGIDevice3::Trim()` when resting.
- **Full Feature Parity with `omarchy-plume`**:
  - **6 Trails**: Comet, Beads, Ribbon, Sparks, Echoes, Bubbles
  - **6 Halos**: Glow, Ring, Pulse, Spotlight, Crosshair, Target
  - **8 Color Modes**: Theme (live Windows DWM accent), Theme palette, Window border, Custom, Gradient, Rainbow, Drift, Speed
  - **Shake-to-Find**: Rapid shake emits expanding radar rings
  - **Custom Cursor Forge**: 14 Arrows, 30 Surfaces, 22 Lines, 18 Ornaments, 5 Shadows, 3 Fills, 4 Textures, 3 Depths — compiled on the fly into native Windows `.cur` and animated 12-frame `.ani` cursors with zero external tools
  - **12 Ready-Made Looks**, **16 Ready-Made Cursors**, **12 Saved Slots**, **Undo**, and **JSON Export/Import**
- **8-Tab GPU Settings Panel**: Live interactive preview, light & dark cursor showcase strip, instant cross-tab search, and full panel theming/opacity/outline. Drops all UI GPU resources when closed.
- **Smart Windows Integration**: Fullscreen game/video auto-hide, hide cursor when idle or typing, Start with Windows toggle, crash-safe cursor restoration, and local Named-Pipe CLI scripting.

---

## Installation (Zero Build Tools Required)

1. Download **`winscroller-mouse.exe`** (or `winscroller-mouse-x86_64-pc-windows-msvc.zip`) from the release assets below.
2. Verify integrity against `SHA256SUMS.txt` (optional).
3. Double-click **`winscroller-mouse.exe`** — it starts silently in the Windows system tray.
4. **Right-click the tray icon** (or run `winscroller-mouse.exe settings`) to open the 8-tab Settings Panel.

---

## Credits

Faithfully ported to Windows and Rust from [omarchy-plume](https://github.com/qempexe/omarchy-plume) by `qempexe` (MIT License).
