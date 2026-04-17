# PROGRESS

Statusblick pro Phase. Updates nach jedem abgeschlossenen logischen Schritt, im selben
Commit wie der Code/Doc-Change, auf den er sich bezieht.

Legende: `[x]` done · `[~]` in progress · `[ ]` todo · `[!]` blocked

---

## Phase 0 — Scaffolding & Entscheidungen ✅

- [x] Prompt-Review — 10 offene Punkte vom Nutzer beantwortet (siehe `DECISIONS.md`).
- [x] Toolchain verifiziert:
  - rustc **1.95.0** stable-x86_64-pc-windows-msvc (2026-04-14)
  - cargo **1.95.0** (2026-03-21)
  - clippy **0.1.95**, rustfmt **1.9.0**
  - node **v24.14.1**, npm **11.11.0**
  - gh **2.89.0**, authenticated as `jstin-cc`
  - _Tauri-CLI: deferred — wird installiert wenn `apps/tray-ui/` begonnen wird._
  - _MSVC Build Tools: präsent (Teil der stable-msvc-Toolchain)._
- [x] Projekt-Skelett + Pflicht-Dokumente (CLAUDE.md, PROGRESS.md, NEXT.md,
      DECISIONS.md, CHANGELOG.md).
- [x] JSONL-Format empirisch untersucht → `docs/jsonl-format.md` (38 Files, 13 MB
      Sample), 3 anonymisierte Fixtures in `fixtures/`.
- [x] Finales Datenmodell → `docs/architecture.md` (inkl. Position-Tracking-Design
      ADR-011).
- [x] API-Schema → `docs/api.md` (`/health`, `/v1/summary`, `/v1/sessions`).
- [x] Design-System → `docs/design-system.md`.
- [x] Pricing-Format → `docs/pricing.md` (ADR-004).
- [x] Agenten-Definitionen: `lead_engineer`, `implementer`, `qa_docs` unter
      `.claude/agents/`.
- [x] Git-Init, `.gitignore`, `.gitattributes` (LF-Normalisierung), Initial-Commit,
      privates GitHub-Repo `jstin-cc/winusage`, Push auf `main`.

**Phase 0 abgeschlossen am 2026-04-17.** Repo: https://github.com/jstin-cc/winusage.

---

## Phase 1 — MVP Kern (`v0.1.0`)

- [ ] CI-Minimal-Workflow `.github/workflows/ci.yml` — `cargo fmt --check && clippy
      -D warnings && cargo test` (Windows-Runner). Laut ADR-007 ab Phase 1.
- [ ] `winusage-core`:
  - [ ] `model.rs` — `UsageEvent`, `Snapshot`, `SessionState`, `ModelId`, `ModelUsage`.
  - [ ] `parser.rs` — `parse_line(&str) -> Result<Option<UsageEvent>>`; Fixtures
        decken happy-path, synthetic-error und sidechain ab.
  - [ ] `pricing.rs` — `PricingTable` mit `include_str!("pricing.json")`, Datum-Suffix-
        Fallback, Warnings.
  - [ ] `scanner.rs` — Full-Scan + Position-Tracking (byte-offset + file-id) + `notify`-
        Watcher, Δ-Scan-Algorithmus laut `docs/architecture.md` §4.
  - [ ] `aggregate.rs` — Rolling-Windows (today / week / month) + active-session.
  - [ ] `config.rs` — Pfade, Auth-Token-Handling.
  - [ ] `examples/scan.rs` liefert ausführbare Dev-CLI: Full-Scan + Dump.
- [ ] CLI-Subcommands: `winusage daily`, `winusage monthly`, `winusage session`.
- [ ] HTTP-API: `/health`, `/v1/summary`, `/v1/sessions` auf `127.0.0.1:7337`,
      Bearer-Token-Auth, Origin-Check.
- [ ] Tray-App Basis-Panel (Tauri 2 + React 18.3) — Layout aus `docs/design-system.md`.
- [ ] Installer (MSI via Tauri Bundler).
- [ ] Release-Tag `v0.1.0-mvp`.

## Phase 2 — Live & smart (`v0.2.0`)

- [ ] `winusage watch` Live-TUI.
- [ ] 5-Stunden-Billing-Windows / Session-Blocks (empirisch validiert, siehe ADR-010).
- [ ] Burn-Rate + Projektionen.
- [ ] Tray: Per-Projekt-Breakdown + Chart (Recharts).
- [ ] Notifications bei Limit-Schwellen.
- [ ] Auto-Start bei Windows-Login (optional).

## Phase 3 — Plugin-ready (`v1.0.0`)

- [ ] Provider-Plugin-Trait (Vorbereitung für Cursor/Codex).
- [ ] Export: CSV, JSON.
- [ ] Heatmap im Tray.
- [ ] Auto-Update via Tauri Updater.
