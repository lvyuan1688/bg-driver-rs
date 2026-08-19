# Changelog

All notable changes to bg-driver-rs are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.4] — 2026-08-20

### Added
- `crates/bg-driver` `ComputerDriver` trait + Windows/macOS/Linux backends.
- `crates/bg-agent` Think/Act/Observe state machine.
- `crates/bg-sandbox` `Sandbox` trait + `Passthrough` impl.
- `crates/bg-bench` micro-benchmark harness.
- `src/main.rs` CLI with `shot` and `info` subcommands.
- `CONTRIBUTING.md`, Issue/PR templates.

## [0.1.3] — 2026-08-15

### Added
- `docs/v0.1.3-patch-notes.md`.

## [0.1.2] — 2026-08-13

### Added
- Initial `ComputerDriver` trait draft.

## [0.1.1] — 2026-08-12

### Added
- Stub `WindowsDriver` returning a black 1280x720 screenshot.

## [0.1.0] — 2026-08-10

Initial public skeleton.
