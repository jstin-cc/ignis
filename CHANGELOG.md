# Changelog

All notable changes to this project will be documented in this file.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [Unreleased]

## [1.0.0] — 2026-04-20

### Added

- `winusage export --format <csv|json> --period <today|week|month>` — maschinenlesbare Ausgabe aus der CLI
- `src/provider.rs` — `Provider`-Trait + `ClaudeCodeProvider` als Erweiterungspunkt für künftige Datenquellen (ADR-012)
- `GET /v1/heatmap` — 84-Tage-Tageskostenübersicht als JSON-Array
- Tray `HeatmapPanel` — 7×n CSS-Grid (12 Wochen), relative Terrakotta-Farbintensität, Montag-Ausrichtung
- Tray Auto-Update — `tauri-plugin-updater`; `check_for_update`-Command; "Updates prüfen"-Button im Settings-Panel; GitHub-Releases-Endpunkt als Platzhalter (Pubkey vor echtem Rollout ersetzen)
- App-Icons (Terrakotta, alle Plattformgrößen via `tauri icon`)

### Fixed

- `Image::from_rgba` → `Image::new_owned` (API-Änderung in Tauri 2)

### Notes

- 57 Tests, `cargo clippy --all-targets -- -D warnings` clean, `cargo fmt --check` clean.
- Auto-Update-Endpunkt (`plugins.updater.pubkey` + `endpoints` in `tauri.conf.json`) muss vor echten Releases mit realem Minisign-Keypair belegt werden.

## [0.2.0] — 2026-04-20

### Added

- `winusage watch` — Live-TUI (ratatui 0.29 + crossterm 0.28 + notify 6): Header / Today+Session / By-Model / Burn-Rate / Footer; Keys q/r/d/m; NO_COLOR-Fallback
- `src/aggregate.rs` — `SessionBlock`, `billing_blocks()`, `active_block_at()`: 5-Stunden-Billing-Windows nach Anthropic-Abrechnungslogik (ADR-010)
- `src/api.rs` — `ActiveBlockDto` mit `percent_elapsed: u8`; `active_block`-Feld in `GET /v1/summary`
- Tray `BlockPanel` — CSS-Fortschrittsbalken, $/h Burn-Rate, verbleibende Block-Zeit
- Tray `ProjectsPanel` — Top-5-Projekte mit proportionalen Mini-Balken und Kosten (kein Recharts)
- Tray `useBlockNotifications` — Windows-Notifications bei 80 % und 100 % Block-Auslastung (`tauri-plugin-notification`)
- Tray `useAutoStart` + Settings-Panel — Auto-Start bei Windows-Login per ⚙-Button ein-/ausschaltbar (`tauri-plugin-autostart`)

### Fixed

- CI: `clippy --all-targets` deckte zwei Warnungen in Test-Code auf (`cloned_ref_to_slice_refs`, `field_reassign_with_default`) — beide behoben

### Notes

- 54 Tests, `cargo clippy --all-targets -- -D warnings` clean, `cargo fmt --check` clean.
- Tray-Build erfordert `npm run tauri build` auf Windows mit WebView2 und Tauri-Voraussetzungen.

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

[Unreleased]: https://github.com/jstin-cc/winusage/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/jstin-cc/winusage/compare/v0.2.0...v1.0.0
[0.2.0]: https://github.com/jstin-cc/winusage/compare/v0.1.0-mvp...v0.2.0
[0.1.0-mvp]: https://github.com/jstin-cc/winusage/compare/v0.0.1...v0.1.0-mvp
[0.0.1]: https://github.com/jstin-cc/winusage/releases/tag/v0.0.1
