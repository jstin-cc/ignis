# PROGRESS

Statusblick pro Phase. Updates nach jedem abgeschlossenen logischen Schritt, im selben
Commit wie der Code/Doc-Change, auf den er sich bezieht.

Legende: `[x]` done · `[~]` in progress · `[ ]` todo · `[!]` blocked

---

## Phase 0 — Scaffolding & Entscheidungen ✅

- [x] Prompt-Review — 10 offene Punkte vom Nutzer beantwortet (siehe `DECISIONS.md`).
- [x] Toolchain verifiziert: rustc 1.95.0, cargo 1.95.0, clippy 0.1.95, rustfmt 1.9.0,
      node v24.14.1, gh 2.89.0 (jstin-cc). Tauri-CLI: deferred.
- [x] Projekt-Skelett + Pflicht-Dokumente.
- [x] JSONL-Format empirisch untersucht → `docs/jsonl-format.md` + 3 Fixtures.
- [x] `docs/architecture.md`, `docs/api.md`, `docs/design-system.md`, `docs/pricing.md`.
- [x] 3 Agent-Definitionen (`lead_engineer`, `implementer`, `qa_docs`).
- [x] Git-Init + Initial-Commit + `gh repo create jstin-cc/winusage --private` + Push.

**Phase 0 abgeschlossen am 2026-04-17.** Repo: https://github.com/jstin-cc/winusage.

---

## Phase 1 — MVP Kern (`v0.1.0`)

- [x] Dependencies: `serde`, `serde_json`, `chrono`, `rust_decimal`, `thiserror`,
      `pretty_assertions` (dev).
- [x] `src/model.rs` — `UsageEvent`, `Snapshot`, `Summary`, `SessionState`, `ModelId`,
      `ModelUsage`, `ProjectUsage`.
- [x] `src/parser.rs` — `parse_line()` mit 6 Unit-Tests (happy-path, synthetic-skip,
      sidechain, user-line-skip, malformed JSON, no-usage-skip).
- [x] CI-Workflow `.github/workflows/ci.yml` — Windows-Runner, fmt + clippy + test.
- [x] `src/pricing.rs` + `src/pricing.json` (Lookup, Datum-Suffix-Fallback, Warnings).
- [x] `src/aggregate.rs` — Rolling-Windows (today / week / month) + active-session.
- [x] `src/scanner.rs` — Full-Scan + Position-Tracking (Byte-Offset + FileIdentity via NTFS-FFI).
- [x] `src/config.rs` — Pfade, Auth-Token, JSON-Persistenz.
- [x] `examples/scan.rs` — Dev-CLI: Full-Scan → JSON-Dump (verifiziert: 38 Files, 2131 Events).
- [ ] CLI-Subcommands: `winusage daily`, `winusage monthly`, `winusage session`.
- [ ] HTTP-API: `/health`, `/v1/summary`, `/v1/sessions`.
- [ ] Tray-App Basis-Panel (Tauri 2 + React 18.3).
- [ ] Installer (MSI via Tauri Bundler).
- [ ] Release-Tag `v0.1.0-mvp`.

## Phase 2 — Live & smart (`v0.2.0`)

- [ ] `winusage watch` Live-TUI.
- [ ] 5-Stunden-Billing-Windows / Session-Blocks (ADR-010).
- [ ] Burn-Rate + Projektionen.
- [ ] Tray: Per-Projekt-Breakdown + Chart (Recharts).
- [ ] Notifications bei Limit-Schwellen.
- [ ] Auto-Start bei Windows-Login (optional).

## Phase 3 — Plugin-ready (`v1.0.0`)

- [ ] Provider-Plugin-Trait (Vorbereitung für Cursor/Codex).
- [ ] Export: CSV, JSON.
- [ ] Heatmap im Tray.
- [ ] Auto-Update via Tauri Updater.
