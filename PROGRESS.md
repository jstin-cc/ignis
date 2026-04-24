# PROGRESS.md

Zentrale Projekt-Dokumentation: Fortschritt, anstehende Arbeiten und Release-History
in einer Datei. Updates nach jedem abgeschlossenen logischen Schritt, im selben Commit
wie der Code/Doc-Change.

Legende: `[x]` done · `[~]` in progress · `[ ]` todo · `[!]` blocked

---

## Next — Anstehende Arbeiten

### v1.2.0 — Dashboard in Tray eingebettet ✅

- [x] Eingebettetes Dashboard-Overlay (360px breit, z-index 11, Escape + ← schließt)
- [x] Live-Tab: Burn-Rate-Sparkline, Active Session, Session-Block-Ring, By-Model-Breakdown
- [x] History-Tab: Week-vs-Week-Bars, 30d-Kosten-Trend, Top-Projekte (This Month)
- [x] Pure-SVG Chart-Komponenten: Sparkline, LineChart, BlockRing, TokenTypeBar, WeekBars
- [x] Neuer API-Endpoint `/v1/burn-rate` (30 Minuten-Buckets, Sidechain-ausgeschlossen)
- [x] `ignis-watch` TUI komplett entfernt (Bin-Target, Binary, Tauri-Command; ADR-015)
- [x] BUGFIX #27 gelöst: `ignis-api.exe` als `externalBin` + `beforeBuildCommand` ins Bundle
- [x] WeekSection nutzt echte Wochendaten (`range=week` statt Monats-Proxy)
- [x] Version auf 1.2.0 gebumpt + getaggt

### v1.3.0-Kandidaten

- [ ] Settings als eigener Tab (statt Overlay)
- [ ] Wochen-Heatmap-Ansicht (7-Tage-Ausschnitt, detaillierter als 12-Wochen-Grid)
- [ ] History-Tab: echte 30-Tage-Projektdaten (`range=30days`) statt Monats-Proxy

### Lokale Hotfixes (nicht im Repo — nur Installations-Reparaturen)

- 2026-04-24: `ignis-api.exe` + `ignis-watch.exe` manuell aus `target/release/` nach
  `%LOCALAPPDATA%\Ignis\` kopiert. Symptom: „API nicht erreichbar" im Tray, weil
  Installer-Bundle die beiden Binaries nicht enthielt. BUGFIX #27 jetzt im Repo gelöst.

### Phase v1.1.0 — Tray-UI Überarbeitung ✅

Details und Abhängigkeitsgraph: `PLAN-UEBERARBEITUNG.md`

- [x] Schritt 0 — Design-Tokens vollständig, IBM Plex Fonts geladen.
- [x] Schritt 1 — format.ts: fmt-Objekt nach DESIGN.md-Spec exportiert.
- [x] Schritt 2 — TabBar.tsx erstellt (today/month/projects/heatmap).
- [x] Schritt 3 — App.tsx: TabBar-Layout, Settings-Overlay, kein Scroll.
- [x] Schritt 4 — TodaySection überarbeitet (section-label, fmt).
- [x] Schritt 5 — MonthPanel: WeekSection-Variante + progressClass implementiert.
- [x] Schritt 6 — BlockPanel: progressClass, Token-Ablauf-UX, CSS-Klassen.
- [x] Schritt 7 — SessionSection überarbeitet.
- [x] Schritt 8 — Projects- und HeatmapPanel auf Tab-Layout angepasst.
- [x] Schritt 9 — Footer: .btn--primary + .btn--ghost CSS-Klassen.
- [x] Schritt 10 — Port 7337 Konflikt-Check vor spawn_api().
- [x] Schritt 11 — CHANGELOG v1.1.0, README + NEXT-Abschnitt aktualisiert.
- [x] Schritt 12 — v1.1.0 getaggt — Phase v1.1.0 abgeschlossen.

---

## Abgeschlossen

### Post-v1.1.0 Hotfixes (2026-04-23)

- [x] Dashboard-Button: `cmd /C start` durch direkten Spawn mit `CREATE_NEW_CONSOLE`-Flag
      ersetzt — zuverlässiger auf Windows 11 (kein cmd.exe-Umweg, kein WT-Konflikt).
- [x] `ignis-watch.exe` als `externalBin` ins Installer-Bundle aufgenommen
      (`tauri.conf.json` + `src-tauri/binaries/`); `beforeBuildCommand` baut den
      Release-Binary automatisch vor jedem `tauri build`.
- [x] `find_watch_binary()` extrahiert + robuster: prüft Same-Dir mit/ohne
      Target-Triple (`ignis-watch-x86_64-pc-windows-msvc.exe`), Repo-Target-Dirs,
      und Verzeichnis-Scan als Fallback. Detaillierte Fehlermeldung statt
      stillem PATH-Fallback. Footer loggt Fehler in console und zeigt 8s lang.
- [x] Content-Bereich scrollbar: `overflow-y: auto`, 4px-Scrollbar nur bei Hover sichtbar
      (`background-color: transparent` → `--border-default` on hover).
- [x] App-Icons aus `apps/tray-ui/src/assets/Logo.png` regeneriert — alle Tauri-Größen
      (PNG, ICO, ICNS, AppX, iOS, Android) neu erzeugt via `tauri icon`.
- [x] `ignis-api.exe` Release-Build (`cargo build --release --bin ignis-api`) +
      neben `ignis-tray.exe` kopiert — `find_api_binary()` findet `same_dir` zuerst.
- [x] Fenster vertikal resizable: `resizable: true`, `maxHeight` entfernt, `minHeight: 280px`,
      `maxWidth: 360px` fix. Shell `height: 100vh`, Content `flex: 1`.

### Design-Vorbereitung v1.1.0 (2026-04-22)

- [x] `DESIGN.md` erstellt — vollständiger Design-Handoff (Farben, Typo, Spacing,
      Komponenten-Struktur, Zahlenformat-Spec, Progress-Bar-Logik).
- [x] `apps/tray-ui/src/styles/tokens.css` angelegt — vollständige Design-Token-Basis
      als Referenz für die Überarbeitung.
- [x] `apps/tray-ui/src/assets/` angelegt — Zielordner für App-Assets.
- [x] `tray/src-tauri/icons/Logo_Final.png` hinzugefügt — finales App-Icon.
- [x] `INITIAL_PROMPT.md` nach `archive/` verschoben — Projekt-Root aufgeräumt.
- [x] `PLAN-UEBERARBEITUNG.md` erstellt — 12-Schritte-Plan für Tray-UI-Überarbeitung.

### Anthropic OAuth Usage-Balken (2026-04-21)

- [x] Tauri-Command `get_anthropic_usage`: liest `~/.claude/.credentials.json`, refresht Token
      automatisch (platform.claude.com), pollt `api.anthropic.com/api/oauth/usage`.
- [x] Drei Balken im BlockPanel (USAGE LIMITS): 5h Block / This Week / Extra Usage.
      Fallback auf JSONL-Einzelbalken wenn Credentials fehlen oder offline.
- [x] Extra-Usage: `is_unlimited`-Flag + Dollar-Betrag wenn kein monatliches Limit gesetzt.
- [x] Float-robustes Parsing für `utilization` und `used_credits` (Anthropic liefert Floats).

### Plan-Usage-Feature (2026-04-21)

- [x] Config-Erweiterung: `PlanKind` (pro/max5/max20/custom) + `PlanConfig.token_limit()`;
      Default max5 (88k tokens), serde-default für Rückwärtskompatibilität.
- [x] API: `plan_token_limit: Arc<AtomicU64>` in `ApiState`; `block_token_limit` +
      `block_token_pct` (token-basiert, 0–100) in `ActiveBlockDto`.
- [x] `ignis-api`: Plan-Limit bei Start + nach jedem Re-Scan aus config.json nachladen.
- [x] Tauri: `get_plan_config` + `set_plan_config` Commands; schreiben direkt in config.json.
- [x] Tray UI: `BlockPanel` zeigt Token-%-Balken als Hero + "X% used · resets in Xh Xm";
      Dollar-Betrag als Sekundärinfo.
- [x] Settings-Panel: Plan-Dropdown (pro/max5/max20/custom) + Custom-Eingabefeld.

### Post-v1.0 Hotfixes (2026-04-21)

- [x] Tray-App spawnt `ignis-api` automatisch als Child-Prozess beim Start,
      killt ihn bei Exit (ADR-013).
- [x] CORS-Layer auf der HTTP-API (`tower-http::cors`): OPTIONS-Preflight + `Access-Control-Allow-*`-Header.
- [x] Tauri 2 Release-Build: `custom-protocol`-Feature in `tray/src-tauri/Cargo.toml` aktiviert.
- [x] Capability `core:window:allow-start-dragging` für `data-tauri-drag-region`.
- [x] Dashboard-Button (Footer) startet `ignis-watch.exe` via `open_cli_dashboard`-Tauri-Command;
      CLI-Button kopiert `ignis` in die Zwischenablage.
- [x] Scrollbarer Content-Bereich im Tray-Panel (Header + Footer bleiben sticky).
- [x] Fetch-Timeout (10 s) im Tray-Polling + sichtbares Error-Banner bei API-Ausfall.

### Phase 3 — Plugin-ready (`v1.0.0`) ✅

- [x] Provider-Plugin-Trait — `src/provider.rs`, `ClaudeCodeProvider`, ADR-012; 57 Tests.
- [x] Export: CSV, JSON — `ignis export --format <csv|json> --period <today|week|month>`.
- [x] Heatmap im Tray — `GET /v1/heatmap`; `HeatmapDay`; 84-Tage-Grid (7×n CSS, Terrakotta-Intensität).
- [x] Auto-Update via Tauri Updater — `tauri-plugin-updater`; `check_for_update`-Command;
      Settings-Panel-Button; Platzhalter-Endpoint; App-Icons generiert.

**Phase 3 abgeschlossen am 2026-04-20.**

### Phase 2 — Live & smart (`v0.2.0`) ✅

- [x] `ignis watch` Live-TUI — ratatui 0.29 + crossterm 0.28 + notify 6;
      Layout: Header / Today+Session / By-Model / Burn-Rate / Footer;
      Keys: q quit, r refresh, d daily, m monthly; NO_COLOR-Fallback.
- [x] 5-Stunden-Billing-Windows — `SessionBlock`, `billing_blocks()`, `active_block_at()`;
      Burn-Rate-Panel: Fortschrittsbalken, $/h, verbleibende Zeit, Block-Start-Uhrzeit;
      8 neue Tests (54 gesamt, alle grün).
- [x] Tray: BlockPanel — Fortschrittsbalken (CSS, kein Recharts), $/h Burn Rate,
      verbleibende Zeit; API: `ActiveBlockDto` + `percent_elapsed` in `/v1/summary`.
- [x] Tray: ProjectsPanel — Top-5-Projekte mit Mini-Balken + Kosten (kein Recharts, reine CSS).
- [x] Notifications bei Limit-Schwellen — `useBlockNotifications` feuert bei 80% + 100%.
- [x] Auto-Start bei Windows-Login — `tauri-plugin-autostart`; Settings-Panel via ⚙-Button.

**Phase 2 abgeschlossen am 2026-04-20.**

### Phase 1 — MVP Kern (`v0.1.0`) ✅

- [x] Dependencies: `serde`, `serde_json`, `chrono`, `rust_decimal`, `thiserror`, `pretty_assertions`.
- [x] `src/model.rs`, `src/parser.rs`, `src/pricing.rs`, `src/aggregate.rs`,
      `src/scanner.rs`, `src/config.rs` — Kern-Module mit Tests.
- [x] CLI-Subcommands: `ignis daily`, `ignis monthly`, `ignis session`, `ignis scan`.
- [x] HTTP-API: `/health`, `/v1/summary`, `/v1/sessions` mit Bearer-Auth.
- [x] Tray-App Basis-Panel (Tauri 2 + React 18.3).
- [x] Installer (MSI + NSIS via Tauri Bundler).
- [x] CI-Workflow `.github/workflows/ci.yml` — Windows-Runner, fmt + clippy + test.
- [x] Release-Tag `v0.1.0-mvp`.

**Phase 1 abgeschlossen am 2026-04-18.**

### Phase 0 — Scaffolding & Entscheidungen ✅

- [x] Toolchain verifiziert, Projekt-Skelett + Pflicht-Dokumente angelegt.
- [x] JSONL-Format empirisch untersucht → `docs/jsonl-format.md` + 3 Fixtures.
- [x] `docs/architecture.md`, `docs/api.md`, `docs/design-system.md`, `docs/pricing.md`.
- [x] 3 Agent-Definitionen (`lead_engineer`, `implementer`, `qa_docs`).
- [x] Git-Init + Initial-Commit + `gh repo create jstin-cc/ignis --private` + Push.

**Phase 0 abgeschlossen am 2026-04-17.** Repo: https://github.com/jstin-cc/ignis.

---

## Changelog

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

### [Unreleased]

### [1.2.0] — 2026-04-24

#### Added
- Eingebettetes Dashboard-Overlay im Tray (Footer-Button "Open Dashboard")
- Live-Tab: Burn-Rate-Sparkline (30 Min), Active Session + Token-Typ-Legende,
  Session-Block-Ring (SVG), By-Model-Breakdown mit relativen Balken
- History-Tab: Week-vs-Week-Doppelbalken, 30d-Kostentrendlinie, Top-Projekte (This Month)
- Pure-SVG Chart-Library: Sparkline, LineChart, BlockRing, TokenTypeBar, WeekBars
- Neuer API-Endpoint `GET /v1/burn-rate` — 30 Minuten-Buckets (ADR-014)
- `ignis-api.exe` als `externalBin` im Installer-Bundle (BUGFIX #27)
- `WeekSection` im Today-Tab nutzt echte Wochendaten (`/v1/summary?range=week`)
  statt Monats-Proxy
- Fenster vertikal resizable (Breite 360px fix, Höhe ab 280px frei ziehbar)
- App-Icons aus finalem Logo.png regeneriert

#### Fixed
- Content-Bereich war bei vollem Today-Tab abgeschnitten (`overflow: hidden` → `overflow-y: auto`)
- Minimale 4px-Scrollbar, nur bei Hover sichtbar
- `ignis-api.exe` fehlte im Release-Ordner; Binary liegt jetzt neben `ignis-tray.exe`

#### Removed
- `ignis-watch` TUI-Dashboard: Binary, Bin-Target, Tauri-Command `open_cli_dashboard` (ADR-015)

### [1.1.0] — 2026-04-23

#### Added

- **TabBar-Navigation** im Tray: vier Tabs (Today / Month / Projects / Heatmap),
  Akzent-Unterstrich auf aktivem Tab, kein vertikales Scrollen mehr.
- **Design-System-Tokens** in `tray/src/index.css`: Spacing-Skala, Border-Radii,
  Schatten, `--font-sans` / `--font-mono` (IBM Plex), Typo-Größen-Tokens,
  `--tray-width` / `--tray-header-height`.
- **IBM Plex Sans + Mono** über Google Fonts in `tray/index.html` geladen.
- **CSS-Klassen** für Progress-Bar (`.progress-track`, `.progress-fill`, `--high/--warning/--danger`),
  Buttons (`.btn`, `.btn--primary`, `.btn--secondary`, `.btn--ghost`), `.section-label`, `.extra-usage`, `.badge`.
- **`progressClass()`** in `MonthPanel.tsx`: CSS-Modifier statt Inline-Farben (75 / 90 / 100 %).
- **`fmt`-Objekt** in `format.ts`: `fmt.usd`, `fmt.tok`, `fmt.dur` nach DESIGN.md-Spec.
- **`WeekSection`**-Variante in `MonthPanel`: Monats-Fortschrittsbalken auf Today-Tab.
- **Token-Ablauf-UX** in `BlockPanel`: Auth-Fehler → lesbare Meldung statt rohem Error-String.
- **Settings-Overlay** in `App.tsx`: öffnet sich über dem Content-Bereich (z-index 10), × schließt.
- **Port-7337-Konflikt-Check** vor `spawn_api()`: kein doppelter Spawn wenn Port belegt.
- **Drei Usage-Balken** im Tray `BlockPanel` (USAGE LIMITS): 5h-Block, Woche und Extra Usage —
  Werte direkt von `api.anthropic.com/api/oauth/usage` via OAuth (`anthropic-beta: oauth-2025-04-20`).
- **Anthropic OAuth-Integration** (`tray/src-tauri`): Tauri-Command `get_anthropic_usage`, automatischer
  Token-Refresh (5-min-Buffer), Polling alle 5 Minuten im Frontend.
- **Plan-Konfiguration** (`src/config.rs`): `PlanKind`-Enum (pro/max5/max20/custom) +
  `PlanConfig.token_limit()`; in `config.json` gespeichert.
- **API: `block_token_limit` + `block_token_pct`** in `GET /v1/summary → active_block`.
- **Settings-Panel** im Tray: Plan-Dropdown + Custom-Token-Limit-Eingabe.
- Tray-App spawnt `ignis-api` automatisch als Child-Prozess (ADR-013).
- CORS-Layer auf der HTTP-API (`tower-http::cors`).
- Tauri-Command `open_cli_dashboard`, CLI-Button kopiert `ignis` in Zwischenablage.
- Fetch-Timeout (10 s), Error-Banner bei API-Ausfall.

#### Fixed

- `used_credits` / `monthly_limit` als `f64` geparst (Anthropic liefert Floats).
- `parse_window`: `utilization` als `f64` statt `u64`.
- Tauri 2 Release-Build: `custom-protocol`-Feature fehlte.
- Capability `core:window:allow-start-dragging` fehlte.

### [1.0.0] — 2026-04-20

#### Added

- `ignis export --format <csv|json> --period <today|week|month>`
- `src/provider.rs` — `Provider`-Trait + `ClaudeCodeProvider` (ADR-012)
- `GET /v1/heatmap` — 84-Tage-Tageskostenübersicht
- Tray `HeatmapPanel` — 7×n CSS-Grid (12 Wochen), Terrakotta-Farbintensität
- Tray Auto-Update — `tauri-plugin-updater`; GitHub-Releases-Endpunkt (Platzhalter)
- App-Icons (Terrakotta, alle Plattformgrößen via `tauri icon`)

#### Fixed

- `Image::from_rgba` → `Image::new_owned` (Tauri 2 API-Änderung)

> 57 Tests, `cargo clippy --all-targets -- -D warnings` clean.

### [0.2.0] — 2026-04-20

#### Added

- `ignis watch` — Live-TUI (ratatui 0.29 + crossterm 0.28 + notify 6)
- `SessionBlock`, `billing_blocks()`, `active_block_at()` — 5h-Billing-Windows (ADR-010)
- Tray `BlockPanel`, `ProjectsPanel`, `useBlockNotifications`, `useAutoStart`

#### Fixed

- CI: zwei Clippy-Warnungen in Test-Code behoben

> 54 Tests, `cargo clippy --all-targets -- -D warnings` clean.

### [0.1.0-mvp] — 2026-04-18

#### Added

- Kern-Module: `model`, `parser`, `pricing`, `aggregate`, `scanner`, `config`
- CLI (`ignis daily/monthly/session/scan`), HTTP-API (`/health`, `/v1/summary`, `/v1/sessions`)
- Tray-App Basis (Tauri 2 + React 18.3), MSI + NSIS Installer
- CI-Workflow (Windows-Runner)

> 46 Tests, `cargo clippy -- -D warnings` clean.

### [0.0.1] — 2026-04-17

- Initial scaffolding: Dokumentation, ADR-001–011, Single-Crate-Layout.

[Unreleased]: https://github.com/jstin-cc/ignis/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/jstin-cc/ignis/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/jstin-cc/ignis/compare/v0.2.0...v1.0.0
[0.2.0]: https://github.com/jstin-cc/ignis/compare/v0.1.0-mvp...v0.2.0
[0.1.0-mvp]: https://github.com/jstin-cc/ignis/compare/v0.0.1...v0.1.0-mvp
[0.0.1]: https://github.com/jstin-cc/ignis/releases/tag/v0.0.1
