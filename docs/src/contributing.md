# Contributing

We welcome pull requests, bug reports, and new trail/halo/cursor ideas!

Before opening a PR, please ensure all 4 CI checks pass cleanly:

```powershell
cargo fmt -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
cargo check --release --workspace --tests
```

See the full guidelines in [`CONTRIBUTING.md`](https://github.com/fruitsaladwindowsai-zammy/Winscroller-mouse/blob/main/CONTRIBUTING.md).
