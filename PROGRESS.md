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
- [x] CLI-Subcommands: `winusage daily`, `winusage monthly`, `winusage session`, `winusage scan`.
- [x] HTTP-API: `/health`, `/v1/summary`, `/v1/sessions`.
- [x] Tray-App Basis-Panel (Tauri 2 + React 18.3).
- [x] Installer (MSI via Tauri Bundler) — konfiguriert in `tray/src-tauri/tauri.conf.json` (targets: msi + nsis); Build läuft via `tauri build` auf Windows.
- [x] Release-Tag `v0.1.0-mvp`.

**Phase 1 abgeschlossen am 2026-04-18.**

## Phase 2 — Live & smart (`v0.2.0`)

- [x] `winusage watch` Live-TUI — ratatui 0.29 + crossterm 0.28 + notify 6;
      Layout: Header / Today+Session / By-Model / Burn-Rate / Footer;
      Keys: q quit, r refresh, d daily, m monthly; NO_COLOR-Fallback.
- [x] 5-Stunden-Billing-Windows — `SessionBlock`, `billing_blocks()`, `active_block_at()`;
      Burn-Rate-Panel: Fortschrittsbalken, $/h, verbleibende Zeit, Block-Start-Uhrzeit;
      8 neue Tests (54 gesamt, alle grün).
- [ ] 5-Stunden-Billing-Windows / Session-Blocks (ADR-010).
- [ ] Burn-Rate + Projektionen.
- [x] Tray: BlockPanel — Fortschrittsbalken (CSS, kein Recharts), $/h Burn Rate,
      verbleibende Zeit; API: ActiveBlockDto + percent_elapsed in /v1/summary.
- [x] Tray: ProjectsPanel — Top-5-Projekte mit Mini-Balken + Kosten (kein Recharts, reine CSS).
- [x] Notifications bei Limit-Schwellen — `useBlockNotifications` feuert bei 80% + 100%;
      `tauri-plugin-notification`; Capabilities-Datei; Fallback außerhalb Tauri.
- [ ] Auto-Start bei Windows-Login (optional).

## Phase 3 — Plugin-ready (`v1.0.0`)

- [ ] Provider-Plugin-Trait (Vorbereitung für Cursor/Codex).
- [ ] Export: CSV, JSON.
- [ ] Heatmap im Tray.
- [ ] Auto-Update via Tauri Updater.
