# Introduction

<p align="center">
  <img src="../assets/brand-banner.svg" alt="Winscroller Mouse" width="100%" />
</p>

**Winscroller Mouse** is a native Windows 10/11 desktop utility written in pure Rust (Edition 2024) that brings the complete experience of [omarchy-plume](https://github.com/qempexe/omarchy-plume) to Windows:

1. **Live GPU Pointer Effects**: Smooth DirectComposition + Direct2D trails, halos, color modes synced live to Windows DWM, and Shake-to-Find radar rings across every monitor.
2. **Procedural Custom Cursor Forge**: Design your own cursor from 14 arrows, 30 surfaces, 22 lines, 18 ornaments, and 5 shadows — compiled in-process via `resvg` into native 32-bit Windows `.cur` and animated `.ani` files with zero external tools.
3. **Engineered for Low Overhead**: Uses a single shared D3D11/D2D1 device, `WS_EX_NOREDIRECTIONBITMAP`, zero per-frame COM brush allocations, adaptive idle sleep (`~0%` CPU), and `IDXGIDevice3::Trim()` (~`9.7 MB` private memory).
