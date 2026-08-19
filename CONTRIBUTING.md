# Contributing to bg-driver-rs

Thanks for your interest! This is a community-driven, open-source computer-use
driver. Contributions of all sizes are welcome.

## Quick start

```bash
git clone https://github.com/lvyuan1688/bg-driver-rs
cd bg-driver-rs
cargo build
cargo test
```

The skeleton ships a stub driver that returns a 1280x720 black screenshot, so
the agent loop can be exercised without a real desktop.

## Ways to contribute

- **Bugs**: open an issue with OS, Rust version, command, and stack trace.
- **Backends**: add or improve a per-OS backend in
  `crates/bg-driver/src/<os>.rs`.
- **Sandbox**: extend `crates/bg-sandbox` with real isolation (seccomp, Job
  Object, sandbox-exec).
- **Benchmarks**: add new micro-benchmarks in `crates/bg-bench`.
- **Docs**: typos, clarifications, and new guides are all welcome.

## Pull request checklist

- [ ] `cargo fmt` is clean
- [ ] `cargo clippy -- -D warnings` is clean
- [ ] `cargo test` passes
- [ ] `CHANGELOG.md` updated (if user-visible)

## Code of conduct

Be kind. Personal attacks, harassment, or discriminatory behavior will not be
tolerated.

## License

By contributing, you agree your contributions are licensed under the MIT
license (see `LICENSE`).
