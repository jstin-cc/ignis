# AUDIT-PLAN.md — Post-v2.0-Audit

**Erstellt:** 2026-04-28 · **Rolle:** lead_engineer · **Phase:** 1 (nur Doku)

Ergebnis eines vollständigen Read-Through aller Quellen, Konfigurationen und
Dokumente nach dem v2.0.0-Tag. Pro Punkt: Bereich · Datei/Zeile · Problem ·
Fix-Vorschlag · Aufwand (S = ½ Tag · M = 1–2 Tage · L = >2 Tage).

Phase 2 wird **erst nach expliziter Freigabe** gestartet. Dann pro Item: Fix
+ Häkchen + Commit + Push, kein Batch.

Legende: `[ ]` offen · `[x]` erledigt · `[-]` won't fix.

---

## P0 — Defekte, falsche Daten oder kaputte UX

### [x] A1 — Tray schreibt Plan/Schwellen/Budget in falsche Config-Datei

- **Bereich:** Bug · Tray-Host
- **Datei:** `tray/src-tauri/src/main.rs:377-384` (`config_path()`)
- **Problem:** `get_api_token` liest aus `%APPDATA%\ignis\config.json`
  (Zeile 357), aber `config_path()` für `get_plan_config`/`set_plan_config`
  /`set_alert_thresholds`/`set_budget_caps`/`get_first_run_seen`/
  `set_first_run_seen` schreibt nach `%APPDATA%\winusage\config.json`. Core
  (`src/config.rs:131-132`) liest ausschließlich `ignis/config.json`. Folge:
  **Sämtliche Plan-, Threshold- und Budget-Änderungen aus dem Settings-Tab
  erreichen den Core nie.** UI zeigt zwar gespeicherte Werte (liest dieselbe
  Datei), aber `ignis-api` rechnet weiter mit Default-Plan und ignoriert
  Schwellen. Auch der First-Run-Wizard markiert in der falschen Datei.
- **Fix:** In `config_path()` `"winusage"` → `"ignis"`. Optional einmalige
  Migrations-Logik: wenn `winusage/config.json` existiert und
  `ignis/config.json` nicht, kopieren + alte löschen. Test: Plan ändern →
  in Datei prüfen → Core neu starten → `/v1/summary` zeigt neues Limit.
- **Aufwand:** S
- **Reihenfolge:** Zuerst (P0, blockiert valide Settings).

### [x] A2 — `ignis-api` re-scannt jede 30 s das gesamte Logverzeichnis

- **Bereich:** Performance · Backend
- **Datei:** `src/bin/ignis-api.rs:70` (Background-Tick)
- **Problem:** `scan_all` statt `scan_incremental`/`scan_delta`. Jeder
  Notify-Event und jeder 30-s-Tick liest **alle** JSONL-Files ab Byte 0,
  parsed alles erneut. Verstößt gegen ADR-011 (Position-Tracking als
  Design-Anforderung) und macht den BUGFIX #5/#11-Aufwand zunichte. Skaliert
  linear mit Gesamt-JSONL-Volumen — bei großen Logs spürbarer CPU-Verbrauch
  des Hintergrundprozesses.
- **Fix:** Hintergrund-Task hält `Vec<FilePosition>` zwischen Ticks; ruft
  `scan_incremental(&dir, &mut positions)` auf; akkumuliert Events analog
  zur ehemaligen TUI-Logik. Bei Boot bleibt `scan_all`. Test: zwei
  aufeinander folgende Scans, zweiter darf 0 neue Bytes lesen.
- **Aufwand:** M
- **Reihenfolge:** Nach A1 (unabhängig, aber höchste Wirkung).

### [!] A3 — Auto-Updater pubkey ist ein Platzhalter (blocked: User-Aktion)

- **Bereich:** Bug · Release-Kette
- **Datei:** `tray/src-tauri/tauri.conf.json:39`
- **Problem:** `"pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXkKUlVRa1FBQUE="`
  decodiert zu `untrusted comment: minisign public key\nRUQkQAAA` — kein
  echter ed25519-Schlüssel. Die `[x] v1.7.0 #1+#2+#3`-Notiz in `PROGRESS.md`
  bestätigt explizit „Schlüssel noch nicht generiert". Auto-Update wird
  beim ersten Roll-out in der Signatur-Verifikation scheitern → Update
  schlägt still fehl, Nutzer hängen auf alter Version.
- **Fix:** `cargo tauri signer generate -w ~/.tauri/ignis.key` ausführen,
  Public-Key in `tauri.conf.json` eintragen, Private-Key + Passwort als
  GitHub-Secrets `TAURI_SIGNING_PRIVATE_KEY` /
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` setzen, Test-Tag `v2.0.1-rc1` →
  `v2.0.1-rc2` Update-Pfad verifizieren.
- **Aufwand:** S (Generation) + M (Verifikations-Test)
- **Reihenfolge:** Vor nächstem Tag, kann parallel zu A1/A2.
- **Status (2026-04-28):** Blocked. Schlüsselgenerierung mit
  Passphrase ist eine User-Aktion (Private-Key darf nie durch den
  Agent erzeugt werden). `docs/release.md` (Z. 23-34) beschreibt den
  Workflow vollständig: `cargo tauri signer generate -w
  ~/.tauri/ignis.key` → Pubkey in `tauri.conf.json:39` eintragen →
  GitHub-Secrets `TAURI_SIGNING_PRIVATE_KEY` /
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` setzen → Test-Tag.

### [x] A4 — Versions-Drift über vier Manifeste

- **Bereich:** Projektstruktur · Release
- **Dateien:**
  - `Cargo.toml:3` → `ignis-core 1.3.0`
  - `tray/src-tauri/Cargo.toml` → `ignis-tray 0.1.0`
  - `tray/package.json` → `winusage-tray 0.1.0`
  - `tray/src-tauri/tauri.conf.json:3` → `2.0.0`
  - Git-Tag → `v2.0.0`
- **Problem:** Das Repo wurde als v2.0.0 getaggt, aber drei der vier
  Versionsfelder sind veraltet. `package.json` heißt zudem noch
  `winusage-tray`. Konsequenzen: Build-Output, `ignis-api --version` (falls
  jemals exposed), Auto-Update-Vergleiche, Crash-Logs werden inkonsistent.
- **Fix:** Alle vier Versionen auf `2.0.0` setzen, `package.json#name` auf
  `ignis-tray` umbenennen. Eine kleine `scripts/bump-version.ps1` (oder
  `.sh`) die alle vier Stellen synchron setzt — für künftige Releases. ADR
  optional: „Single source of version" festschreiben.
- **Aufwand:** S
- **Reihenfolge:** Nach A1/A2/A3 (kosmetisch aber wichtig vor v2.0.1).

---

## P1 — Stale Dokumentation, Code-Hygiene, Robustheit

### [x] B1 — Doppelte ADR-Nummer 012 in `DECISIONS.md`

- **Bereich:** Dokumentation
- **Datei:** `DECISIONS.md:152` (Provider-Trait, 2026-04-20) und
  `DECISIONS.md:196` (Sidechain-Events, 2026-04-20)
- **Problem:** Zwei verschiedene ADRs tragen Nummer 012. Reihenfolge ist
  zudem nicht-numerisch (010 → 012 → 011 → 012 → 013 → 014 → 017 → 016 →
  015). ADR-Verweise im Code (`build_burn_rate`-Doc, `aggregate.rs`)
  zeigen damit auf einen mehrdeutigen Anker.
- **Fix:** Sidechain-Block zu ADR-012b umbenennen (oder neuer ADR-018
  mit Status `Supersedes ADR-012` und korrektem Inhalt). Datei in
  numerischer Reihenfolge sortieren. Kreuz-Verweise in Code/Docs prüfen
  (grep auf `ADR-012`, `ADR-015`, `ADR-016`, `ADR-017`).
- **Aufwand:** S
- **Reihenfolge:** Sofort umsetzbar, blockiert nichts.

### [x] B2 — `BUGFIX-PROGRESS.md` referenziert tote Pfade

- **Bereich:** Dokumentation
- **Datei:** `BUGFIX-PROGRESS.md` (mehrere Stellen, u. a. #7, #19, #5, #10)
- **Problem:** Bug-Beschreibungen nennen `src/bin/winusage.rs`,
  `src/bin/winusage-watch.rs`, `src/bin/winusage-api.rs` und
  `winusage daily/monthly`. Diese Dateien wurden umbenannt (`ignis*.rs`)
  oder gelöscht (TUI per ADR-015). Done-Log #19 sagt
  „`src/bin/winusage-watch.rs:42`" — Datei existiert nicht mehr.
- **Fix:** Bug-Liste auf `src/bin/ignis*.rs` aktualisieren oder einen
  „Pfad-Migrations"-Hinweis-Block am Datei-Ende einfügen
  („Pfade vor 2026-04-24 hießen `winusage*`, gilt für #5–#27"). Letzteres
  ist billiger und ehrlicher.
- **Aufwand:** S
- **Reihenfolge:** Nach B1.

### [x] B3 — Tote `apps/tray-ui/`-Doku-Spuren

- **Bereich:** Projektstruktur
- **Dateien:** `apps/tray-ui/` (nur `.gitkeep`, `tokens.css`, untracktes
  Logo); Verweise in `DESIGN.md`, `PLAN-UEBERARBEITUNG.md`,
  `.claude/agents/implementer.md`, `.gitignore`.
- **Problem:** Die echte Tray lebt unter `tray/`. `apps/tray-ui/` ist
  ein verwaister Plan-Stub und sorgt für Verwirrung („wo ist die
  React-App?"). `.gitignore` hat noch zwei Einträge dafür.
- **Fix:** `apps/tray-ui/` löschen (nur `tokens.css` ist potentiell
  relevant — vor Löschen prüfen, ob er aus `tray/src/index.css`
  importiert wird). Doku-Verweise auf `apps/tray-ui/` durch `tray/src/`
  ersetzen. `.gitignore`-Einträge entfernen.
- **Aufwand:** S
- **Reihenfolge:** Nach B2 (rein doku-/dateibewegend).

### [x] B4 — Stale `.gitignore`-Einträge + fehlende Build-Artefakt-Regel

- **Bereich:** Projektstruktur
- **Datei:** `.gitignore`
- **Problem:**
  - Zeile 50: `/winusage-data/` — alter Name, Verzeichnis existiert nicht
    mehr.
  - `tray/src-tauri/binaries/` ist *nicht* ignoriert (untracked, enthält
    aber 2 MB-Build-Artefakt `ignis-api-x86_64-pc-windows-msvc.exe`).
  - `apps/tray-ui/node_modules/` und `apps/tray-ui/dist/` referenzieren
    Verzeichnis das laut B3 verschwindet.
- **Fix:** `/winusage-data/` und `apps/tray-ui/`-Einträge entfernen.
  `tray/src-tauri/binaries/` (außer `.gitkeep`) hinzufügen. Optional
  Repo nach `.exe`-Dateien durchsuchen, die versehentlich committet
  sind (`git ls-files | rg '\.exe$'`).
- **Aufwand:** S
- **Reihenfolge:** Nach B3.

### [x] B5 — Tote Dependencies in `Cargo.toml`

- **Bereich:** Code-Qualität
- **Datei:** `Cargo.toml:40-41`
- **Problem:** `ratatui = "0.29"` und `crossterm = "0.28"` sind
  Dependencies, obwohl die TUI per ADR-015 (2026-04-24) entfernt wurde.
  Erhöht Build-Zeit + Compile-Footprint ohne Nutzen.
- **Fix:** Beide entfernen, `cargo build` + Tests müssen weiterhin
  bestehen. CLAUDE.md-Tabelle „CLI/TUI" enthält noch
  „ratatui + crossterm + clap" — auf nur `clap` reduzieren.
- **Aufwand:** S
- **Reihenfolge:** Nach B4.

### [x] B6 — `ignis-api`-Pfad-Bin-Output-Ort hardcoded auf Windows

- **Bereich:** Build · Cross-Platform
- **Datei:** `tray/src-tauri/tauri.conf.json:7`
- **Problem:** `beforeBuildCommand` enthält
  `... && copy ..\\target\\release\\ignis-api.exe ...`. `copy` ist eine
  cmd.exe-Builtin; auf Linux/macOS-Buildern (z. B. künftiger
  Cross-Build) failt das. CLAUDE.md sagt explizit „Core portabel halten
  für späteren Linux-Support".
- **Fix:** Plattform-agnostisches Skript (`scripts/copy-sidecar.cjs` oder
  `node`-One-Liner) oder bedingt via `npm run build:sidecar`. Alternativ
  `tauri-action`-Workflow übernimmt das schon — also ggf. einfach
  `beforeBuildCommand` reduzieren auf `npm run build && cargo build
  --release --bin ignis-api --manifest-path ../Cargo.toml` und Sidecar-
  Copy in eine `.js`-Datei auslagern, die `node` plattformneutral
  ausführt.
- **Aufwand:** S
- **Reihenfolge:** Optional, kann nach B5.

### [x] B7 — Panic-Pfade in Produktions-Code

- **Bereich:** Code-Qualität · Robustheit
- **Dateien:**
  - `src/aggregate.rs:297` → `.expect("midnight is always valid")` in
    `build_hourly_heatmap`.
  - `src/bin/ignis-api.rs:13` → `.expect("static addr is valid")` (ok,
    static).
  - `src/bin/ignis-api.rs:103` → `.expect("failed to install Ctrl+C
    handler")`.
- **Problem:** CLAUDE.md verbietet `unwrap()`/Panic in Produktions-Code.
  `expect("midnight is always valid")` ist riskant: an
  DST-Übergangstagen (Sommer/Winterzeit) liefert
  `Local.with_ymd_and_hms(...)` `None` für die nicht-existierende Stunde.
  Für 00:00 zwar harmlos, aber stilistisch verboten.
- **Fix:** `.single().or_else(|| Local.with_ymd_and_hms(... 1, 0, 0).single())
  .unwrap_or_else(|| Utc::now().with_timezone(&Local))` oder explizit
  fehlertolerant: bei `None` → leerer Heatmap-Vec zurückgeben (graceful
  degradation). Ctrl+C-Handler: `if let Err(e) = ... { eprintln!(...) }`.
- **Aufwand:** S
- **Reihenfolge:** Nach B6.

### [x] B8 — `unwrap_or_default()` schluckt `spawn_blocking`-Fehler still

- **Bereich:** Code-Qualität · Beobachtbarkeit
- **Datei:** `src/bin/ignis-api.rs:71-72`
- **Problem:** `tokio::task::spawn_blocking(... scan_all ...).await
  .unwrap_or_default()` setzt bei Join-Fehler einen leeren `ScanResult`.
  Symptom: Wenn der Scanner-Thread paniciert, sieht die UI einfach „0
  Events" und niemand erfährt davon.
- **Fix:** Bei `Err(e)` `eprintln!("scanner task panicked: {e:?}")` und
  alten Snapshot beibehalten (`continue`), nicht überschreiben.
- **Aufwand:** S
- **Reihenfolge:** Im selben Commit wie A2.

### [x] B9 — `ALLOWED_ORIGINS` weiterhin hardcoded

- **Bereich:** Architektur · ADR-Konformität
- **Datei:** `src/api.rs:131-137`
- **Problem:** ADR-005 fordert „konfigurierbare Allowlist". Aktuell
  4-er-Liste hart einkompiliert — Bug #13 hat erweitert, aber nicht
  konfigurierbar gemacht. Nutzer mit anderem Vite-Port oder Browser-
  Plugin-Setup können nichts ändern.
- **Fix:** `Config` um `Vec<String> allowed_origins` erweitern (default =
  bisherige Liste). `ApiState::new` nimmt Slice; `check_origin` liest aus
  State. Dokumentieren in `docs/api.md`.
- **Aufwand:** S
- **Reihenfolge:** Nach B8.

---

## P2 — Cleanup, Konsistenz, Future-Proofing

### [ ] C1 — `archive/INITIAL_PROMPT.md` und `PLAN-UEBERARBEITUNG.md`

- **Bereich:** Projektstruktur · Doku-Hygiene
- **Dateien:** `archive/INITIAL_PROMPT.md` (20 kB), `PLAN-UEBERARBEITUNG.md`
- **Problem:** Beide sind erledigte Plan-Dokumente; Inhalt ist via
  `git log` rekonstruierbar. Verwirrt neue Contributor.
- **Fix:** Inhalt verifizieren (kein Wissen, das nicht in `DECISIONS.md`
  oder `PROGRESS.md` steht), löschen. Wenn Wissen fehlt → in passenden
  Dauer-Dokument migrieren.
- **Aufwand:** S
- **Reihenfolge:** Nach B-Kette.

### [ ] C2 — `rust-toolchain.toml` (1.95.0) vs. CI-Action `@stable`

- **Bereich:** CI · Reproducibility
- **Datei:** `rust-toolchain.toml`, `.github/workflows/ci.yml:21`
- **Problem:** `dtolnay/rust-toolchain@stable` installiert *neueste*
  stable; `rust-toolchain.toml` pinnt 1.95.0. Action respektiert
  `rust-toolchain.toml` zwar (Cargo aktiviert sie), aber Komponenten
  (`clippy`, `rustfmt`) werden für die Action-Version installiert. Wenn
  die Versionen auseinander driften, fehlen Komponenten oder es gibt
  Subtle-Inkonsistenzen.
- **Fix:** Action auf `dtolnay/rust-toolchain@1.95.0` pinnen oder
  `rust-toolchain.toml` löschen und einzig `@stable` als Quelle. ADR
  empfehlen, was der „source of truth" ist.
- **Aufwand:** S
- **Reihenfolge:** Nach C1.

### [x] C3 — `BUGFIX-PROGRESS.md` Done-Log endet auf `(pending)`

- **Bereich:** Dokumentation
- **Datei:** `BUGFIX-PROGRESS.md:239`
- **Problem:** Letzte Zeile sagt
  `2026-04-24 · #27 · ignis-api als externalBin … · (pending)` — der
  Commit-Hash fehlt, obwohl Bug abgehakt ist.
- **Fix:** Echten Hash eintragen (`git log --oneline | rg externalBin`)
  oder Note zu „mehrere Commits" mit Hash-Liste.
- **Aufwand:** S
- **Reihenfolge:** Beliebig.

### [ ] C4 — `Snapshot.sessions` und Polling-Interval als Config

- **Bereich:** Performance · Konfigurierbarkeit
- **Datei:** `src/aggregate.rs` (`Snapshot::sessions`), `src/api.rs`
  (Sessions-Cap), `src/bin/ignis-api.rs:57` (30 s Tick)
- **Problem:** Bug #24 cappt API-Output auf 500, aber `Snapshot.sessions`
  selbst hält alle jemals gesehenen Sessions im Speicher. Bei sehr
  großen `.claude/projects/`-Verzeichnissen wächst das unbegrenzt.
  30-s-Tick ist hardcoded — `PlanConfig.usage_poll_interval_secs` gibt es
  schon, wird aber nur fürs Anthropic-Usage-API verwendet, nicht für den
  Re-Scan-Tick.
- **Fix:** Optional: nach Konfig-Wert (`scan_interval_secs`, default 30)
  pollen. `Snapshot.sessions` cappen + ältere droppen, oder hash-basiert
  deduplicieren. Beides nice-to-have.
- **Aufwand:** M (zusammen)
- **Reihenfolge:** Niedrige Prio.

### [x] C5 — Lange Tauri-Commands für Config-Mutation

- **Bereich:** Code-Qualität
- **Datei:** `tray/src-tauri/src/main.rs` (mehrere `set_*`-Commands)
- **Problem:** `set_plan_config`, `set_alert_thresholds`, `set_budget_caps`,
  `set_first_run_seen` haben sehr ähnliche Struktur (read JSON → mutate
  field → write). Datei nähert sich der CLAUDE.md-Grenze von 300 Zeilen.
- **Fix:** Helper `mutate_config_json<F>(path, f: F)`-Pattern; Reduktion
  um ~30 Zeilen. Achtung: aus dem A1-Fix folgt sowieso, dass alle dasselbe
  Pfad-Konstrukt nutzen — ideal um beides zusammen zu refactor.
- **Aufwand:** S
- **Reihenfolge:** Im selben Commit wie A1.

### [x] C6 — ADR-005-Doku referenziert `winusage`-Pfad

- **Bereich:** Dokumentation
- **Datei:** `DECISIONS.md:82`
- **Problem:** ADR-005 nennt
  `%APPDATA%\winusage\auth-token.txt` — aber Token liegt jetzt in
  `%APPDATA%\ignis\config.json#api_token` (kein eigenes Token-File mehr).
  Konkretisierung an die Realität fehlt.
- **Fix:** ADR mit „Updated 2026-04-…"-Note ergänzen oder Folge-ADR
  schreiben. Inhaltlich nur Naming-Änderung.
- **Aufwand:** S
- **Reihenfolge:** Mit B1.

### [x] C7 — README-Status-Tabelle auf 2.0.0 hieven

- **Bereich:** Dokumentation
- **Datei:** `README.md` (Status-Tabelle, laut `PROGRESS.md` auf
  „v1.7.0-Stand")
- **Problem:** Vor `v2.0`-Tag wurde der Auftrag „README für PR
  aktualisieren" laut CLAUDE.md erfüllt, Status-Tabelle laut PROGRESS.md
  spiegelt aber v1.7.0. Kontrolllesen vor Public-Marketing nötig.
- **Fix:** README + Screenshots-Abschnitt + Feature-Tabelle gegen
  v2.0.0-Stand verifizieren.
- **Aufwand:** S
- **Reihenfolge:** Vor v2.0.1-Patch.

---

## Reihenfolge der Abarbeitung

```
Phase 2.1 — kritische Defekte
  A1 (config-path) → C5 (Refactor im selben Commit)
  A2 (scan_incremental) + B8 (spawn_blocking-Log) → ein Commit
  A3 (pubkey)
  A4 (versions)

Phase 2.2 — Doku & Struktur
  B1 (ADR-012) → B2 → C6
  B3 (apps/tray-ui weg) → B4 (.gitignore)
  C3 (commit-hash Bugfix #27)
  C7 (README v2.0.0)

Phase 2.3 — Code-Hygiene
  B5 (ratatui/crossterm raus)
  B6 (cross-platform copy)
  B7 (Panics → graceful)
  B9 (ALLOWED_ORIGINS configurierbar)

Phase 2.4 — Optional / Backlog
  C1 (archive/ + PLAN-UEBERARBEITUNG.md löschen)
  C2 (toolchain pin)
  C4 (snapshot.sessions cap + scan_interval)
```

**Abhängigkeiten:**
- A1 vor C5 (selber Commit).
- A2 vor B8 (selber Commit).
- B1 vor B2/C6 (Cross-Refs konsistent halten).
- B3 vor B4 (`.gitignore` reflektiert tatsächlichen Zustand).
- A4 nach allen funktionalen Fixes (sonst sind Versionen schon getauscht
  bevor Bugs raus sind).

**Aufwand insgesamt:** ~2 Tage konzentriert (alle S/M Items).

---

## Out of Scope für dieses Audit

Bewusst nicht behandelt (entweder vorhandene Backlog-Items oder
v2.x-Features):

- Linux/macOS-Support (CLAUDE.md sagt portabel-halten, aber kein Build).
- Authenticode-Signing (ADR-016 — Defer auf eigenes Cert).
- Telemetrie-Opt-in / Plugin-API-Stabilisierung / i18n / MSIX (alle
  v2.0+-Backlog laut `PROGRESS.md`).
- Dauerhaft persistente Position-Maps (laut ADR-011 nur bei spürbaren
  Startup-Zeiten — nicht beobachtet).
