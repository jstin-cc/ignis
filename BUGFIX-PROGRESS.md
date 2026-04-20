# BUGFIX-PROGRESS

Priorisierte Liste der bei der Code-Review (2026-04-20) gefundenen Bugs,
fehlenden Fehlerbehandlungen und Real-Data-Risiken. Wird wie `PROGRESS.md`
gepflegt: jeder Fix wird im selben Commit wie der Code-Change abgehakt und
mit kurzer Notiz (Commit-Hash, Datum, Modul) versehen.

Legende: `[x]` done · `[~]` in progress · `[ ]` todo · `[!]` blocked · `[-]` won't fix (mit Begründung)

Reihenfolge der Fixes (Empfehlung aus Review):

1. P0-Quick-Wins (#1, #2, #4, #6, #7, #9) — Output falsch / Panic-Risiko
2. P0 #3 + P1 #10 — Tray ist real defekt ohne Token-Auth + API-Polling
3. P0 #5 + P1 #11 — Position-Tracking endlich nutzen
4. P0 #8 + P1 #16 — versteckte Datenverluste sichtbar machen
5. P1 #12, #13, #15 — Sicherheits-/Robustheits-Hygiene
6. P2 — nach Lust

---

## P0 — echte Defekte / Programm liefert falsche Werte

- [x] **#1 `ProjectUsage.session_count` ist konstant 0**
  - Symptom: `/v1/summary` → jedes `by_project[].session_count == 0`.
  - Datei: `src/aggregate.rs:201-207` (`accumulate_summary`).
  - Ursache: `proj.session_count` wird nirgends erhöht.
  - Fix-Skizze: in `build_snapshot` pro Session einen Pass über
    `(session.project_path → unique session_ids)` führen, dann am Ende in die
    `by_project`-Maps der drei Summaries schreiben — oder `accumulate_summary`
    eine `seen_sessions: &mut HashSet<(PathBuf, String)>` mitgeben.

- [x] **#2 UI labelt `event_count` als „sessions"**
  - Symptom: TodayPanel/MonthPanel: „1234 tokens · 47 sessions" — Zahl ist
    aber `event_count` (API-Calls), nicht Sessions.
  - Datei: `tray/src/components/TodayPanel.tsx:18`, `MonthPanel.tsx:18`.
  - Ursache: Falsches Label; `Summary` enthält keine echte Session-Zahl.
  - Fix-Skizze: kurzfristig Label auf „events" ändern. Mittelfristig API
    um `session_count` pro Range erweitern.

- [x] **#4 Cache-Tokens werden in Aggregation doppelt gezählt**
  - Symptom: `cache_creation_tokens` im API-Output zu hoch, sobald ein Event
    sowohl Top-Level `cache_creation_input_tokens` als auch verschachtelte
    `ephemeral_5m/1h` liefert.
  - Datei: `src/aggregate.rs:188-189`.
  - Ursache: `cache_creation_tokens + ephemeral_5m + ephemeral_1h` — laut
    `parse_line` (`docs/pricing.md §4`) ist `cache_creation_input_tokens`
    bereits die Summe der ephemerals.
  - Kosten sind korrekt (Pricing nutzt nur ephemerals).
  - Fix-Skizze: nur `ev.cache_creation_tokens` summieren oder nur
    `(ephemeral_5m + ephemeral_1h)` — exakt **eine** Quelle.

- [x] **#6 Invalide/fehlende Timestamps werden zu `Utc::now()`**
  - Symptom: Kaputte Zeile mit `timestamp: null` oder Müll wandert mit
    aktueller Zeit in „today / week / month / active session" → fälschlich
    aktive Session, Burn-Rate verfälscht.
  - Datei: `src/parser.rs:56-59`.
  - Ursache: `.unwrap_or_else(chrono::Utc::now)`.
  - Fix-Skizze: `Ok(None)` zurückgeben, wenn `timestamp` fehlt/unparsbar
    (graceful degradation, keine Fake-Daten).

- [x] **#7 `truncate(&str, max)` paniert bei Non-ASCII**
  - Symptom: `winusage daily/monthly` und `winusage-watch` panicken, sobald
    ein Modell-Name oder Pfad ein Multi-Byte-Zeichen enthält und
    `s.len() > max`. Beispiel: `D:\projekte\müller\…`.
  - Datei: `src/bin/winusage.rs:178-184`, `src/bin/winusage-watch.rs:488-494`.
  - Ursache: `&s[..max]` schneidet auf Byte-Index, nicht Char-Boundary.
    Verstößt gegen „keine `unwrap()`/Panic in Produktion".
  - Fix-Skizze: char-basierte Truncation (`s.chars().take(max).collect()`)
    oder `floor_char_boundary`-Pattern.

- [x] **#9 `HeatmapPanel` rechnet `Math.max(...[NaN])`**
  - Symptom: Ein einziges nicht-parsbares `cost_usd` → `maxCost = NaN` →
    alle Zellen `rgba(NaN)` → komplett transparente Heatmap.
  - Datei: `tray/src/components/HeatmapPanel.tsx:28`.
  - Ursache: Kein NaN-Filter.
  - Fix-Skizze: NaN-Werte vor `Math.max` herausfiltern; `cellColor` defensiv
    auf 0 setzen, wenn Eingabe NaN.

- [ ] **#3 Tray sendet keinen Bearer-Token → 401 mit Default-Config**
  - Symptom: Tray ist mit Default-Config (Token wird beim ersten Start
    erzeugt) komplett kaputt — alle Fetches scheitern mit 401.
  - Datei: `tray/src/useUsageData.ts:15-37`, `hooks/useUpdater.ts`.
  - Ursache: Kein Mechanismus, den Token aus `%APPDATA%\winusage\config.json`
    zu laden und mitzuschicken.
  - Fix-Skizze: Tauri-Command `get_api_token`, der Config liest und an JS
    übergibt. JS hängt `Authorization: Bearer <token>` an alle Fetches.

- [ ] **#5 TUI macht bei jedem Tick Full-Scan, ignoriert Position-Tracking (ADR-011)**
  - Symptom: `winusage-watch` skaliert linear mit Gesamtdaten (alle 5 s +
    bei Notify-Event). Verstößt gegen ADR-011.
  - Datei: `src/bin/winusage-watch.rs:98-103` (`App::refresh`).
  - Ursache: `scan_all(...)` statt `scan_delta(&previous_positions)`.
  - Fix-Skizze: `App` hält `Vec<FilePosition>`; `refresh` ruft `scan_delta`
    + akkumuliert Events; bei rotierten Files kommt der Delta-Pfad mit.

- [ ] **#8 WalkDir-Errors werden komplett geschluckt**
  - Symptom: Permission-denied auf einem JSONL-File → File ist unsichtbar,
    kein Eintrag in `scan.errors`.
  - Datei: `src/scanner.rs:88-96`.
  - Ursache: `.filter_map(|e| e.ok())` verwirft Walk-Errors.
  - Fix-Skizze: `WalkDir`-Errors als `ScanError::Io` durchreichen, nicht
    droppen.

---

## P1 — Architektur-/Robustheits-Probleme

- [ ] **#10 `winusage-api` re-scannt nach Boot nie wieder**
  - Symptom: Server zeigt nach Stunden weiterhin Boot-Stand;
    `snapshot_age_ms` in `/health` wird permanent größer.
  - Datei: `src/bin/winusage-api.rs:13-30`.
  - Ursache: Es fehlt der Watcher-Loop (analog zur TUI), der periodisch
    `scan_delta` ausführt + `state.update_snapshot()` triggert.
  - Fix-Skizze: `tokio::spawn` mit `notify` + Tick (z. B. 5 s); identische
    Logik wie TUI, aber asynchron.

- [ ] **#11 Race zwischen `file_identity()` und `File::open()`**
  - Symptom: Datei rotiert genau zwischen den beiden Calls → Position wird
    unter falscher Identity gespeichert, nächste Rotation unbemerkt.
  - Datei: `src/scanner.rs:99-130`.
  - Ursache: Zwei separate `File::open`.
  - Fix-Skizze: einmal öffnen, dann `GetFileInformationByHandle` auf dem
    bereits offenen Handle; analog Unix `fstat`.

- [ ] **#16 Sidechain-Events fließen ungefiltert in alle Summen**
  - Symptom: Today/Month/By-Project enthalten Tokens und Kosten von
    Subagent-Calls — User sieht höhere Beträge als sein Hauptthread real
    verbraucht.
  - Datei: `src/aggregate.rs` (`build_snapshot`).
  - Ursache: `is_sidechain` wird gespeichert, aber nirgends als Filter
    oder Sub-Summe genutzt.
  - Fix-Skizze: ADR schreiben: Sidechain explizit ein- oder ausschließen.
    Default vermutlich „einschließen, aber separat ausweisen".

- [ ] **#12 Token-Generator ist nicht crypto-zufällig**
  - Symptom: Lokales API-Token vorhersagbar bei Kenntnis von Boot-Zeit + PID.
    Verstößt gegen ADR-005-Geist.
  - Datei: `src/config.rs:127-137`.
  - Ursache: Eigenbau-LCG aus `subsec_nanos` + PID.
  - Fix-Skizze: `getrandom` (oder `rand::thread_rng().fill`) für 16 echte
    Zufalls-Bytes → hex.

- [ ] **#13 `ALLOWED_ORIGINS` matcht weder Vite-Default noch alle Tauri-Origins**
  - Symptom: Vite Default-Port 5173 → 403. Linux-Tauri-Origin nicht erfasst.
  - Datei: `src/api.rs:110`.
  - Ursache: Hard-codierte 2-er-Liste; ADR-005 sah konfigurierbare
    Allowlist vor.
  - Fix-Skizze: Allowlist aus Config lesen, Defaults sinnvoll erweitern
    (`tauri://localhost`, `http://localhost:1420`, `http://localhost:5173`).

- [ ] **#15 RwLock-Poisoning wird stillschweigend ignoriert**
  - Symptom: Nach Panic in einem Handler bleibt der Lock poisoned, alle
    künftigen `update_snapshot`-Aufrufe sind No-Ops.
  - Datei: `src/api.rs:34-46`.
  - Ursache: `if let Ok(mut guard)` verwirft `Err`-Fall.
  - Fix-Skizze: `into_inner()` bei Poison nutzen — analog zu `read_snapshot`.

- [ ] **#14 `config.json`-Fehlermeldung lügt auf Non-Windows**
  - Symptom: Wenn weder `APPDATA` noch `HOME` gesetzt → Error sagt
    „required env var 'APPDATA' is not set".
  - Datei: `src/config.rs:73-90`.
  - Fix-Skizze: bei Fallback-Branch die andere Variablen-Konstante nutzen.

- [ ] **#17 Polling im Tray ohne `AbortController`**
  - Symptom: Bei langsamen Fetches (>30 s) stapeln sich In-Flight-Requests.
  - Datei: `tray/src/useUsageData.ts:50-70`.
  - Fix-Skizze: `AbortController` pro Run; vorherigen Run abbrechen.

- [ ] **#18 Notification feuert sofort beim Öffnen, wenn Block ≥80 % ist**
  - Symptom: Tray öffnen mitten in altem Block → sofort 80%-Notification.
  - Datei: `tray/src/hooks/useBlockNotifications.ts`.
  - Fix-Skizze: Erste Beobachtung pro Block markieren („baseline"); nur
    feuern, wenn `previousPct < threshold && currentPct >= threshold`.

---

## P2 — Kleinkram

- [ ] **#19 NO_COLOR akzeptiert leeren String**
  - Datei: `src/bin/winusage-watch.rs:42` — Spec verlangt non-empty.

- [ ] **#20 `winusage export` schreibt nur nach stdout**
  - Backlog (steht in `NEXT.md`): `--output <file>`-Flag.

- [ ] **#21 `home_projects_dir()` Fehlerklasse meldet falsches `var`**
  - Datei: `src/config.rs:82-90` — analog #14.

- [ ] **#22 `SessionDto.is_active` mit `active_id == Some(...)`**
  - Funktioniert, ist aber unnötig fragil. Defensive Refaktorierung.

- [ ] **#23 `BlockPanel.computeBurnRate` mischt Client- und Server-Uhr**
  - Datei: `tray/src/components/BlockPanel.tsx:74-81` — Drift → negativ.

- [ ] **#24 `Snapshot.sessions` wächst unbegrenzt**
  - Snapshot enthält alle jemals gesehenen Sessions; `/v1/sessions` clamped
    erst auf 500.

- [x] **#25 Tauri-Tray-Icon ist 1×1 transparenter Pixel**
  - Datei: `tray/src-tauri/src/main.rs:91`.

- [x] **#26 `pricing.json` ist `PLACEHOLDER`, v1.0.0 schon getaggt**
  - Datei: `src/pricing.json:4`. Steht auch in `NEXT.md`.

---

## Done-Log (chronologisch)

> Format: `YYYY-MM-DD · #N · <kurze Beschreibung> · <commit-hash>`

2026-04-20 · #26 · pricing.json PLACEHOLDER-Note entfernt, Datum aktualisiert · 6afb557
2026-04-20 · #25 · Tray-Icon: default_window_icon() statt 1×1-Pixel · a628cc2
2026-04-20 · #2  · TodayPanel/MonthPanel: 'sessions' → 'events' · a856a85
2026-04-20 · #1  · ProjectUsage.session_count via HashSet-Tracking befüllt · 1a65fd8
2026-04-20 · #4  · Cache-Tokens: nur cache_creation_tokens (kein Doppel-Counting) · c1a20bf
2026-04-20 · #6  · parser: invalide Timestamps → Ok(None) statt Utc::now() · 257fed2
2026-04-20 · #7  · truncate(): char_indices().nth() statt Byte-Slice-Panic · be5ddc7
2026-04-20 · #9  · HeatmapPanel: NaN-Filter vor Math.max, cellColor isFinite-Guard · 4d9347e
