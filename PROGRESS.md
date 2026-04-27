# PROGRESS.md

Zentrale Projekt-Dokumentation: Fortschritt, anstehende Arbeiten und Release-History
in einer Datei. Updates nach jedem abgeschlossenen logischen Schritt, im selben Commit
wie der Code/Doc-Change.

Legende: `[x]` done · `[~]` in progress · `[ ]` todo · `[!]` blocked

---

## Roadmap v1.3.0 → v2.0

Strategischer Plan vom aktuellen Stand bis zum Major-Release. Rahmen: keine
neuen Plattformen, keine neuen Provider, v2.0 ist Qualitäts- und Feature-Reife
inkl. Public-Repo.

Bug-Status (Stand 2026-04-24): Alle in `BUGFIX-PROGRESS.md` gelisteten P0- und
P1-Punkte sind abgehakt. Es gibt aktuell **keine offenen P0-Blocker** für
v1.3.0. `#26` (pricing.json) steht trotz Häkchen explizit als Pflege-Aufgabe in
v1.3.0 — die Werte müssen vor jedem Release gegen Anthropic-Preisliste
re-verifiziert werden, das ist Dauer-Aufgabe pro Release.

**Aufwands-Skala:** S = ½–1 Tag · M = 2–4 Tage · L = 1+ Woche

### v1.3.0 — Datenwahrheit & Settings-Reife

> Theme: „Was die App zeigt, stimmt — und Konfiguration fühlt sich nicht mehr
> nach Hack an."

| # | Feature | Akzeptanzkriterium | Aufwand |
|---|---------|--------------------|---------|
| 1 | `pricing.json` re-verifiziert (#26 Pflege) | Alle aktiv genutzten Modelle in `pricing.json` mit Datum 2026-04 abgeglichen, Quelle verlinkt in `docs/pricing.md`, Auto-Reload-Test (File-Mtime) | S |
| 2 | Settings als eigener Tab | Fünfter Tab in `TabBar` (Today/Month/Projects/Heatmap/Settings), Overlay-Code entfernt, Plan-Picker + Token-Limit + Auto-Start + Update-Check + API-Token (read-only) im Tab | M |
| 3 | History-Tab: echte 30-Tage-Projektdaten | Neuer Range `range=30days` in `/v1/summary`, `MonthPanel`/`HistoryTab` zieht Top-Projekte aus echten 30 Tagen statt Monats-Proxy | M |
| 4 | `export --output <file>` Polish (#20 follow-up) | Pfad-Validierung, Overwrite-Schutz (`--force`), atomarer Write (tmp + rename), Dokumentation in `docs/cli.md`, 3 Integrationstests | S |
| 5 | Settings-Migration `config.json` v1 → v2 | Versions-Feld in Config, Migrations-Pfad mit Backup-Datei, kein Daten-Verlust bei Tab-Refactor | S |

**Abhängigkeiten:** keine externen. Settings-Tab (#2) blockt #5; Range `30days`
(#3) ist self-contained.

**Definition of Done v1.3.0:** Alle fünf Punkte abgehakt, `cargo clippy --
-D warnings` clean, Tests grün, `pricing.json` Datum aktuell, `1.3.0` getaggt.

---

### v1.4.0 — Visualisierung & Wahrnehmung

> Theme: „Auf einen Blick sehen, wann du arbeitest."

| # | Feature | Akzeptanzkriterium | Aufwand |
|---|---------|--------------------|---------|
| 1 | Wochen-Heatmap (7-Tage-Ausschnitt) | Neuer Sub-View in HeatmapPanel: 7 Tage × 24 h, Stunden-Buckets, Terrakotta-Intensität nach Tokens, Toggle `12 Wochen` ↔ `Diese Woche` | M |
| 2 | API: `GET /v1/heatmap?granularity=hour&range=week` | 168 Buckets (7×24), korrekte Sidechain-Filterung wie `/v1/burn-rate` | S |
| 3 | Heatmap-Tooltip | Hover-Tooltip mit Tag, Uhrzeit, Tokens, Kosten — pure CSS/SVG, keine Library | S |
| 4 | Today-Tab: Stunden-Sparkline | Mini-Chart unter Today-Hero: 24-h-Verlauf der heutigen Tokens, ergänzt Burn-Rate-Sparkline aus v1.2.0 | S |

**Abhängigkeiten:** v1.3.0 (#3) liefert Range-Architektur für `30days` —
saubere Vorlage für `granularity=hour`.

**Definition of Done v1.4.0:** Heatmap-Toggle funktioniert ohne Layout-Bruch
(Fenster bleibt 360 px breit), Tooltip kein Flackern, `1.4.0` getaggt.

---

### v1.5.0 — Budget-Kontrolle

> Theme: „Du wirst rechtzeitig gewarnt — nicht erst wenn's brennt."

| # | Feature | Akzeptanzkriterium | Aufwand |
|---|---------|--------------------|---------|
| 1 | Konfigurierbare Budget-Schwellen | Settings-Tab: Schwellen als Liste (50/75/90/100 % Default), pro Schwelle Einzel-Toggle, persistiert in `config.json` | M |
| 2 | Schwellen-Notifications | `useBlockNotifications` liest Schwellen aus Config statt hardcoded 80/100, feuert je Schwelle einmal pro Block (Baseline-Logik aus #18 bleibt) | S |
| 3 | Wochen-/Monats-Budget (USD) | Optional: USD-Cap für Woche/Monat in Settings, eigene Notification-Reihe wenn überschritten | M |
| 4 | Budget-Status im BlockPanel | Sichtbare „Next alert at X%" -Zeile, damit User Schwellen versteht ohne Settings öffnen zu müssen | S |

**Abhängigkeiten:** v1.3.0 (#2) — Settings-Tab muss Liste-Editor unterstützen.

**Definition of Done v1.5.0:** Schwellen sind editierbar + werden gefeuert,
keine Doppel-Notifications pro Block, `1.5.0` getaggt.

---

### v1.6.0 — Onboarding & First-Run-Polish

> Theme: „Beim ersten Start ist klar, was die App ist und wo der Token liegt."

| # | Feature | Akzeptanzkriterium | Aufwand |
|---|---------|--------------------|---------|
| 1 | First-Run-Screen | Erkennung über `config.first_run_seen: bool`; 3-Step Wizard im Tray (Willkommen → Plan-Auswahl → Auto-Start-Opt-in) | M |
| 2 | Empty-State, wenn keine JSONL gefunden | Statt blanker `0 tokens` ein Hinweis mit Pfad zu `~/.claude/projects` und Doku-Link | S |
| 3 | API-Token-Anzeige + Copy-Button | Im Settings-Tab: Token + „Copy" + Hinweis „Für CLI/curl-Zugriff" | S |
| 4 | Inline-Hilfe in Settings | Kurze Mikrocopy zu jedem Setting (Plan, Schwellen, Auto-Start) — pure Tooltips, keine Modals | S |

**Abhängigkeiten:** v1.3.0 (#2) Settings-Tab; v1.5.0 (#1) Schwellen-UI.

**Definition of Done v1.6.0:** Frischer Install führt durch Wizard, Empty-State
sichtbar wenn `~/.claude/projects` leer, `1.6.0` getaggt.

---

### v1.7.0 — Auto-Update produktionsreif

> Theme: „Updates kommen automatisch und sind verifizierbar."

| # | Feature | Akzeptanzkriterium | Aufwand |
|---|---------|--------------------|---------|
| 1 | Echter GitHub-Releases-Endpoint | `tauri-plugin-updater` zeigt auf `https://github.com/jstin-cc/ignis/releases/latest/download/latest.json`, Platzhalter aus v1.0.0 ersetzt | S |
| 2 | Update-Manifest-Generator | CI-Job/Script erzeugt `latest.json` (Version, Pub-Datum, Signatur, MSI/NSIS-URL) bei jedem `git tag v*` | M |
| 3 | Code-Signing der Installer | Tauri-Updater-Signing-Key (ed25519) generiert, Public-Key embedded, Private-Key in GitHub Secrets, Installer signiert; ADR über Authenticode (defer auf v2.0 falls Cert-Kosten zu hoch) | M |
| 4 | Release-Notes-Anzeige | Settings-Tab: „Update verfügbar" zeigt Changelog-Auszug aus Release-Body, „Install & Restart"-Button | S |
| 5 | Rollback-Doku | `docs/release.md`: wie ein vergiftetes Release zurückrollen, Tag löschen, Manifest patchen | S |

**Abhängigkeiten:** v1.6.0 (#3) Settings-Tab Update-Sektion.

**Definition of Done v1.7.0:** Tag `v1.7.0-rc1` erzeugt valides Manifest, ein
zweiter Test-Tag `v1.7.0-rc2` updatet `rc1`-Install ohne manuelle Intervention,
`1.7.0` getaggt.

---

### v2.0.0 — Public-Release-Reife

> Theme: „Stabil, dokumentiert, öffentlich. Externe können beitragen."

**Was macht es zu einem Major-Release?**
Bruch der Privatheit (Repo wird öffentlich), formale Stabilitäts-Garantien für
HTTP-API-Schema (`/v1/*`-Endpoints versioniert + Deprecation-Policy), und ein
Contributor-Onboarding-Pfad. Kein Tech-Stack-Wechsel.

#### Pflicht (must-have für v2.0.0)

| # | Feature | Akzeptanzkriterium | Aufwand |
|---|---------|--------------------|---------|
| 1 | Repo public schalten | `gh repo edit jstin-cc/ignis --visibility public`, Secrets-Audit (keine Tokens/Keys/Pfade), Issue-Templates (Bug/Feature/Question), Discussions an | S |
| 2 | LICENSE | MIT oder Apache-2.0; Entscheidung in ADR; Header-Snippet in Quellen optional | S |
| 3 | README mit Screenshots | Hero-Screenshot Tray, Feature-Liste, Install-Steps (MSI-Download), Quick-start CLI/API, Status-Tabelle, Lizenz-Badge, Build-Badge | M |
| 4 | `CONTRIBUTING.md` + `CODE_OF_CONDUCT.md` | Branch-Strategie (main + feature-branches), PR-Template, Review-Checkliste, lokales Build-Setup, ADR-Prozess kurz erklärt | M |
| 5 | API-Schema-Stabilität | `docs/api.md` markiert `/v1/*` als stabil, Deprecation-Policy (1 Minor-Release Vorlauf), `/v2/*`-Pfad-Reservation dokumentiert | S |
| 6 | Stabilitäts-Audit | Manueller Test-Run aller Features auf frischer Win11-VM, Issue-Liste abgearbeitet, keine offenen P0/P1 in `BUGFIX-PROGRESS.md` | M |
| 7 | Authenticode-Signing | Echtes Code-Signing-Cert oder dokumentierter Defer-Pfad mit SmartScreen-Workaround in README | M–L |
| 8 | Versionierungs-Doku | `docs/release.md` erweitert: SemVer-Regeln, Major/Minor/Patch-Trigger, Changelog-Workflow | S |

#### Nice-to-have (v2.0+ Backlog, kein Blocker)

| # | Feature | Aufwand |
|---|---------|---------|
| A | Telemetrie-Opt-in (Crash-Reports lokal sammeln, manuell exportierbar) — bleibt opt-in, no-default | M |
| B | Plugin-API-Stabilisierung — `Provider`-Trait als public extension point dokumentieren | M |
| C | i18n-Vorbereitung (string-Extraktion, ohne zweite Sprache liefern) | M |
| D | Screen-Reader-Audit Tray-UI | S |
| E | MSIX-Bundle als zusätzliches Installer-Format | M |

**Abhängigkeiten:** v1.7.0 muss stabil laufen (Auto-Update wird das primäre
Distributions-Vehikel nach Public-Release).

**Definition of Done v2.0.0:** Repo public, README mit Screenshots live,
CONTRIBUTING merged, eine externe Person kann den Build laut Doku ohne Hilfe
reproduzieren, `2.0.0` getaggt + GitHub Release publiziert.

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

### v1.3.0 — Datenwahrheit & Settings-Reife (in Arbeit)

Vollständiger Scope und Akzeptanzkriterien siehe Roadmap-Abschnitt oben.

- [x] **#1 `pricing.json` re-verifiziert (2026-04-27)** — Werte gegen platform.claude.com
      abgeglichen: Opus 4.7/4.6 von $15/$75 → $5/$25 (−66 %), Haiku 4.5 von $0.80/$4 →
      $1/$5 (+25 %), Sonnet 4.6 unverändert. Cache-Read- und Cache-Write-5m/1h-Preise
      mit aktualisiert. `source`-Feld in `pricing.json` ergänzt. `docs/pricing.md`:
      Beispiel-Snippet aktualisiert, Platzhalter-Note entfernt, §7 um Verifikations-
      Historie erweitert. Drei neue Tests in `src/pricing.rs`: `embedded_default_has_current_models`
      (vier Pflicht-Modelle vorhanden), `opus_47_pricing_matches_anthropic_2026_04`,
      `haiku_45_pricing_matches_anthropic_2026_04`. Auto-Reload als Akzeptanzkriterium
      gestrichen — widerspricht ADR-004 (`include_str!`-embedded). 11 Pricing-Tests grün,
      62 Tests gesamt, Clippy + fmt clean.
- [x] **#2 Settings als eigener Tab (2026-04-27)** — Fünfter Tab in `TabBar`
      (Today/Month/Projects/Heatmap/Settings). Neue Komponente
      `tray/src/components/SettingsTab.tsx` mit Sektionen Allgemein
      (Auto-Start), Plan (Picker + Custom-Token-Limit + Polling-Interval),
      Updates (Update-Check), API-Token (read-only, maskiert mit Copy-Button).
      Settings-Overlay-Markup + ⚙-Header-Button aus `App.tsx` entfernt.
      `TabBar` auf 5 Tabs angepasst (`fontSize` 12 → 11px, `letterSpacing` 0.04
      → 0.03em). Vite-Build + tsc + ESLint clean. Tauri-Runtime-Test pending
      auf User-Seite (Tauri-Bridge nicht headless testbar).
- [x] **#3 History-Tab: echte 30-Tage-Projektdaten (2026-04-27)** — Neuer
      Range `30days` in `/v1/summary` (rolling 30 Tage = heute + 29 vorangegangene,
      midnight-aligned). `Snapshot` um Feld `last_30_days: Summary` erweitert,
      `Windows::for_now` berechnet `last_30_days_start`, `build_snapshot`
      akkumuliert parallel zu today/week/month inkl. session_count-Tracking.
      `useUsageData` fetcht zusätzlich `range=30days`, `UsageData` um
      `last30Days` erweitert. `HistoryTab` zieht Top-Projects jetzt aus echten
      30 Tagen statt Monats-Proxy; Section-Label „TOP PROJECTS THIS MONTH" →
      „TOP PROJECTS 30 DAYS"; ungenutzter `month`-Prop aus
      `Dashboard`/`HistoryTab` entfernt. `docs/api.md` Range-Tabelle ergänzt.
      Test `summary_range_30days_returns_200` neu, 63 Tests grün, clippy +
      fmt + tsc + Vite + ESLint clean.
- [x] **#4 `export --output <file>` Polish (2026-04-27)** — Neues Flag `--force`
      in `ignis export`; `open_output` → `write_output` mit atomarem Write (schreibt
      in `<file>.ignis_tmp`, dann rename), Overwrite-Schutz (Fehler wenn Datei
      existiert und kein `--force`), Pfad-Validierung (Parent-Dir muss existieren).
      3 Unit-Tests in `src/bin/ignis.rs` (#[cfg(test)]): `export_json_creates_new_file`,
      `export_json_refuses_overwrite_without_force`, `export_json_force_overwrites_existing_file`.
      Neue Doku `docs/cli.md` (Subcommand-Übersicht, export-Details, JSON-Schema,
      Beispiele). 66 Tests gesamt (63 lib + 3 bin), clippy + fmt clean.
- [x] **#5 Settings-Migration `config.json` v1 → v2 (2026-04-27)** — `config_version: u32`
      (default 0) zu `StoredConfig` hinzugefügt; `PlanConfig` um
      `usage_poll_interval_secs: u32` (default 60, `#[serde(default = "…")]`) erweitert.
      Migration: `load_file` erkennt `config_version < 2`, erstellt Backup
      `config.json.v0.bak` (oder `.v1.bak`), setzt Version auf 2, schreibt neu —
      Best-effort, kein Crash bei Fehler. `save_file` schreibt immer `config_version: 2`.
      3 neue Tests in `config::tests`: `v1_config_without_version_field_gets_default_plan_fields`,
      `migration_creates_backup_and_upgrades_version`, `v2_config_loads_without_creating_backup`.
      69 Tests gesamt (66 lib + 3 bin), clippy + fmt clean.

### v1.4.0+ Backlog

Reihenfolge und Inhalt siehe Roadmap-Abschnitt oben (v1.4.0 Heatmap-Wochenview,
v1.5.0 Budget-Schwellen, v1.6.0 Onboarding, v1.7.0 Auto-Update prod, v2.0.0 Public).

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

#### Added
- Settings als fünfter Tab im Tray (`SettingsTab.tsx`) mit Sektionen Allgemein,
  Plan, Updates, API-Token (read-only, maskiert + Copy-Button).
- Drei neue Pricing-Tests in `src/pricing.rs`:
  `embedded_default_has_current_models`, `opus_47_pricing_matches_anthropic_2026_04`,
  `haiku_45_pricing_matches_anthropic_2026_04`.
- Neuer Summary-Range `30days` (rolling 30 Tage) in `/v1/summary`;
  `Snapshot.last_30_days`, `Windows::in_last_30_days()`, Test
  `summary_range_30days_returns_200`.
- `ignis export --force`: Overwrite-Schutz + atomarer Write (tmp + rename) +
  Pfad-Validierung; 3 Unit-Tests in `src/bin/ignis.rs`.
- `docs/cli.md`: neues CLI-Referenz-Dokument mit Subcommand-Übersicht,
  export-Details, JSON-Schema und Beispielen.
- `config_version: u32` in `StoredConfig`, `usage_poll_interval_secs: u32` in
  `PlanConfig`; automatische Migration v1 → v2 mit Backup-Datei.

#### Changed
- `pricing.json` re-verifiziert gegen platform.claude.com (Stand 2026-04-27):
  Opus 4.7/4.6 input $15 → $5, output $75 → $25 (cache_read $1.50 → $0.50,
  cache_write_5m $18.75 → $6.25, cache_write_1h $30 → $10);
  Haiku 4.5 input $0.80 → $1.00, output $4 → $5 (cache_read $0.08 → $0.10,
  cache_write_5m $1.00 → $1.25, cache_write_1h $1.60 → $2.00);
  Sonnet 4.6 unverändert. `source`-Feld in `pricing.json` ergänzt.
- `docs/pricing.md` §7: Verifikations-Historie + verlinkte Primärquelle.
- `TabBar.tsx`: 5 Tabs (Today/Month/Projects/Heatmap/Settings); `fontSize` 12 → 11px,
  `letterSpacing` 0.04 → 0.03em, `padding 0 2px` damit alle Labels sauber passen.
- `App.tsx`: Settings-Overlay (`settingsOpen`-State, Overlay-Markup, ⚙-Button im
  Header) entfernt; Settings ausschließlich über den neuen Tab erreichbar.
- `HistoryTab` zieht Top-Projects jetzt aus echten 30 Tagen statt Monats-Proxy;
  Section-Label „TOP PROJECTS THIS MONTH" → „TOP PROJECTS 30 DAYS";
  ungenutzter `month`-Prop aus `Dashboard`/`HistoryTab` entfernt.
- `docs/api.md`: `range`-Tabelle um `30days` erweitert.

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
