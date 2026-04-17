# PROGRESS

Statusblick pro Phase. Updates nach jedem abgeschlossenen logischen Schritt, im selben
Commit wie der Code/Doc-Change, auf den er sich bezieht.

Legende: `[x]` done · `[~]` in progress · `[ ]` todo · `[!]` blocked

---

## Phase 0 — Scaffolding & Entscheidungen

- [x] Prompt-Review — 10 offene Punkte vom Nutzer beantwortet (siehe `DECISIONS.md`).
- [x] Toolchain verifiziert:
  - rustc **1.95.0** stable-x86_64-pc-windows-msvc (2026-04-14)
  - cargo **1.95.0** (2026-03-21)
  - clippy **0.1.95**, rustfmt **1.9.0**
  - node **v24.14.1**, npm **11.11.0**
  - gh **2.89.0**, authenticated as `jstin-cc`
  - _Tauri-CLI: deferred — wird installiert wenn `apps/tray-ui/` begonnen wird._
  - _MSVC Build Tools: präsent (Teil der stable-msvc-Toolchain)._
- [~] Projekt-Skelett + Pflicht-Dokumente
- [ ] JSONL-Format empirisch untersucht → `docs/jsonl-format.md`
- [ ] Finales Datenmodell → `docs/architecture.md` (inkl. Position-Tracking-Design)
- [ ] API-Schema → `docs/api.md`
- [ ] Design-System → `docs/design-system.md`
- [ ] Agenten-Definitionen (`lead_engineer`, `implementer`, `qa_docs`)
- [ ] Git-Init + Initial-Commit + `gh repo create` + Push

## Phase 1 — MVP Kern (`v0.1.0`)

- [ ] CI-Minimal-Workflow (`cargo fmt --check && clippy -D warnings && test`) — laut ADR-007
      ab Phase 1.
- [ ] `winusage-core`: JSONL-Scanner mit `notify`, Position-Tracking pro File.
- [ ] Pricing-Engine (embedded Default, Warning bei unbekanntem Modell).
- [ ] Aggregation: today, week, month, active session.
- [ ] CLI-Subcommands: `daily`, `monthly`, `session`.
- [ ] HTTP-API: `/health`, `/v1/summary`, `/v1/sessions` (Bearer-Token-Auth, 127.0.0.1-Bind,
      Origin-Check).
- [ ] Tray-App Basis-Panel (Tauri 2 + React 18.3).
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
