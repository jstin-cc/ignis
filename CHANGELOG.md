# Changelog

All notable changes to this project will be documented in this file.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [Unreleased]

## [0.1.0-mvp] — 2026-04-18

### Added

- `src/model.rs` — core data types: `UsageEvent`, `Snapshot`, `Summary`, `SessionState`, `ModelId`, `ModelUsage`, `ProjectUsage`
- `src/parser.rs` — JSONL line parser with graceful skip for synthetic/malformed/non-billing lines
- `src/pricing.rs` + `src/pricing.json` — embedded pricing table, date-suffix fallback, unknown-model warnings
- `src/aggregate.rs` — rolling-window aggregation: today / this week / this month; active-session detection
- `src/scanner.rs` — full scan + delta scan with byte-offset position tracking and NTFS file-identity checks
- `src/config.rs` — path resolution, auth token, JSON persistence
- `src/api.rs` + `src/bin/winusage-api.rs` — Axum HTTP API on `127.0.0.1:7337`: `GET /health`, `GET /v1/summary`, `GET /v1/sessions`; Bearer-token auth, Origin-header block
- `src/bin/winusage.rs` — CLI binary: `daily`, `monthly`, `session`, `scan` subcommands (clap 4)
- `tray/` — Tauri 2 + React 18.3 tray app: 360 px panel (Today / Month / Active Session), frameless toggle-on-click window, MSI + NSIS installer configuration
- `examples/scan.rs` — dev CLI: full scan → JSON dump to stdout
- `.github/workflows/ci.yml` — CI on Windows runner: fmt + clippy + test
- `docs/` — architecture, API spec, design system, JSONL format, pricing docs
- `DECISIONS.md` — ADR-001 through ADR-011

### Notes

- MSI/NSIS installer build requires `npm run tauri build` on Windows with WebView2 and Tauri prerequisites.
- 46 tests, `cargo clippy -- -D warnings` clean, `cargo fmt --check` clean.

## [0.0.1] — 2026-04-17

### Added

- Initial repository scaffolding (Phase 0): documentation, architecture decisions,
  single-crate Rust layout with `winusage-core` lib + `examples/scan.rs`.

[Unreleased]: https://github.com/jstin-cc/winusage/compare/v0.1.0-mvp...HEAD
[0.1.0-mvp]: https://github.com/jstin-cc/winusage/compare/v0.0.1...v0.1.0-mvp
[0.0.1]: https://github.com/jstin-cc/winusage/releases/tag/v0.0.1
