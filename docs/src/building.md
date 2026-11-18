# Building from Source

## Prerequisites
- Windows 10 / 11 (64-bit)
- Rust 1.85+ (Edition 2024)
- Visual Studio 2022 Build Tools (Desktop development with C++ / Windows SDK)

## Build Commands

```powershell
git clone https://github.com/fruitsaladwindowsai-zammy/Winscroller-mouse.git
cd Winscroller-mouse

# Format, lint & test workspace
cargo fmt -- --check
cargo clippy --workspace --all-targets
cargo test --workspace

# Build standalone statically-linked release EXE (+crt-static)
cargo build --release -p winscroller-mouse
```

The standalone executable (with embedded multi-res icon) is generated at:
`target\release\winscroller-mouse.exe`
