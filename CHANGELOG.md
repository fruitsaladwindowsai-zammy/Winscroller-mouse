# Changelog

All notable changes to **Winscroller Mouse** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-11-18

### Added
- **Cargo Workspace Architecture (Rust Edition 2024)**: Split cleanly into `winscroller-mouse-core` (OS-agnostic physics, trails, halos, colors, settings, and SVG/CUR/ANI Cursor Forge) and `winscroller-mouse` (Windows DirectComposition/Direct2D daemon & UI).
- **DirectComposition + DXGI Flip-Model + Direct2D Renderer**: True per-pixel alpha transparency over the Windows desktop with `WS_EX_NOREDIRECTIONBITMAP`, multi-monitor support, cached GPU brushes, adaptive idle sleep, and `IDXGIDevice3::Trim()` memory trimming (`~9.7 MB` private memory).
- **6 Smooth Trails**: Comet, Beads, Ribbon, Sparks (gravity), Echoes, and Bubbles — using exact Catmull-Rom curve smoothing and additive non-overlapping band rendering.
- **6 Pointer Halos**: Glow, Ring, Pulse, Spotlight (radial cutout), Crosshair, and Target with `Always`, `While moving`, and `When still` visibility modes.
- **8 Color Modes**: Theme (live Windows DWM accent sync), Theme palette, Window border, Custom, Gradient, Rainbow, Drift, and Speed.
- **Shake-to-Find**: Configurable sensitivity (`1–10`) and radar ring size (`30–160`).
- **Complete Procedural Cursor Forge ("Made in Winscroller Mouse")**: 14 arrows, 30 surfaces, 3 fills, 4 textures, 3 depths, 22 lines, 18 ornaments, 5 shadows, accent caps, details, and matching hand/beam/busy/resize cursors — rasterized in pure Rust via `resvg` into native `.cur` and 12-frame `.ani` files with automatic Windows system cursor restoration.
- **8-Tab Direct2D/DirectWrite Settings Panel**: Live interactive Trail/Halo preview, light & dark cursor showcase strip, cross-tab search bar, 12 ready-made looks, 16 ready-made cursors, 12 saved slots, 1-step Undo, JSON Copy/Paste, and full panel theming/opacity/outline. Frees all UI resources on close.
- **Windows Integration & CLI IPC**: System tray icon with full context menu, fullscreen game/video auto-hide, idle & typing cursor hiding, Start with Windows option, panic crash logger (`%APPDATA%\WinscrollerMouse\crash.log`), and Named-Pipe CLI (`\\.\pipe\WinscrollerMouse`).
- **Statically Linked Standalone Binary**: `+crt-static` MSVC build with embedded multi-resolution `.ico` resource, GitHub Actions CI/Release/mdBook workflows, Wiki, and interactive landing page.
