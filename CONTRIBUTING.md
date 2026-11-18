# Contributing to Winscroller Mouse

Contributions, bug reports, and pull requests are welcome!

---

## Prerequisites

- **Windows 10 or Windows 11** (64-bit) — Winscroller Mouse targets DirectComposition / Direct3D11 / Direct2D / DirectWrite / Win32 APIs natively.
- **Rust toolchain 1.85+** (workspace uses **Rust Edition 2024**).
- **Visual Studio Build Tools** with the **Desktop development with C++** / Windows SDK workload.

---

## Workspace Layout

```text
.
├── Cargo.toml                         # Cargo workspace (edition = "2024")
├── .cargo/config.toml                 # Static CRT linking (+crt-static)
├── winscroller-mouse.example.json     # Reference default config template
├── crates/
│   ├── winscroller-mouse-core/        # Pure OS-agnostic logic & SVG Cursor Forge (unit-tested)
│   │   ├── src/
│   │   │   ├── lib.rs                 # Public API exports
│   │   │   ├── motion.rs              # Pointer path, EMA speed & shake-to-find detector
│   │   │   ├── colors.rs              # 8 color modes + HSL/RGB helpers
│   │   │   ├── settings.rs            # Settings schema, 12 looks, 16 cursors & 12 slots
│   │   │   ├── draw.rs                # Backend-agnostic 2D DrawCmd stream
│   │   │   ├── trails.rs              # Catmull-Rom Band + all 6 trails
│   │   │   ├── halos.rs               # All 6 halos + Find burst
│   │   │   ├── paint.rs               # Per-frame compositor
│   │   │   └── cursor_forge/          # 14 arrows, 30 surfaces, 22 lines, 18 ornaments, SVG/CUR/ANI
│   │   └── tests/
│   │       └── core_tests.rs          # Core, Cursor Forge, CUR/ANI & example config tests
│   └── winscroller-mouse/             # Native Windows daemon, GPU renderer & UI
│       ├── build.rs                   # Pure-Rust .res icon embedder
│       ├── assets/winscroller-mouse.ico
│       └── src/
│           ├── main.rs                # Event loop, panic crash logger, CLI & lifecycle
│           ├── renderer.rs            # DirectComposition + DXGI Flip-Model + D2D1 renderer
│           ├── settings_ui.rs         # 8-tab Direct2D/DirectWrite Settings Panel + live preview
│           ├── win_cursor.rs          # Native Windows .cur/.ani applier & system cursor restore
│           ├── win_theme.rs           # DWM accent sync, fullscreen detection, autostart, clipboard
│           ├── tray.rs                # System tray icon & context menu
│           ├── ipc.rs                 # Local Named-Pipe IPC server & CLI client
│           └── store.rs               # Atomic settings.json persistence
├── docs/                              # mdBook documentation for GitHub Pages
├── wiki/                              # GitHub Wiki pages & assets
└── website/                           # Interactive landing page & local dev server
```

---

## Development & Testing Workflow

1. **Fork and clone** the repository, then create a feature branch:
   ```powershell
   git checkout -b feature/my-improvement
   ```
2. **Keep core logic testable**: Place pure rendering math, color modes, settings validation, and SVG/CUR/ANI generation inside `crates/winscroller-mouse-core` so it can be unit tested without a live Win32 desktop.
3. **Run all CI checks before committing**:
   ```powershell
   cargo fmt -- --check
   cargo clippy --workspace --all-targets
   cargo test --workspace
   cargo check --release --workspace --tests
   ```
   > If rebuilding `--release` while running locally, quit the running instance first with `winscroller-mouse.exe quit`.
4. **Update docs when changing user-facing behavior**: Update `winscroller-mouse.example.json`, `USAGE.md`, `CONFIGURATION.md`, `docs/src/`, and `CHANGELOG.md` as needed.
5. **Open a Pull Request** describing what changed, why, and how it was verified.

---

## Coding Standards

- Follow `rustfmt` formatting (`cargo fmt`).
- Avoid duplicating logic; keep functions focused and shallow.
- Keep idle memory and CPU usage minimal — never allocate per-frame COM brushes or leave background timers spinning when the pointer rests.
- Validate any config schema changes against `example_json_parses_cleanly_and_validates` in `crates/winscroller-mouse-core/tests/core_tests.rs`.

## License

By contributing, you agree that your contributions are licensed under the [MIT License](LICENSE).
