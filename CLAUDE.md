# CLAUDE.md — Projekt-Kontext Ignis

**Diese Datei wird zu Beginn jeder Session gelesen.** Gemeinsam mit `PROGRESS.md` und
`NEXT.md` bildet sie den Einstiegspunkt bei Kontextverlust.

Lies in dieser Reihenfolge:
1. `CLAUDE.md` (hier) — Was ist das Projekt, welche Regeln gelten?
2. `PROGRESS.md` — Was ist done / in-progress / blocked?
3. `NEXT.md` — Was ist der **eine** konkrete nächste Schritt?
4. `BUGFIX-PROGRESS.md` — offene Bugfixes aus dem Code-Review (priorisiert).
   Wenn `NEXT.md` „mögliche nächste Schritte" listet und kein konkreter
   Auftrag vorliegt, sind Einträge hier mit Status `[ ]` der Default-Workstack.
   Pflege analog zu `PROGRESS.md`: Häkchen + Done-Log-Eintrag im selben
   Commit wie der Fix.

Bei Architektur-Fragen zusätzlich: `DECISIONS.md`, dann `docs/architecture.md`.

---

## Projekt in einem Satz

Windows-nativer, lokaler Claude-Code-Usage-Tracker mit Tray-App, CLI/TUI und HTTP-API —
liest JSONL-Logs aus `%USERPROFILE%\.claude\projects\`, zeigt Tokens/Kosten/Sessions.

## Repository

- GitHub: **privat** (später-public ist möglich, aktuell nicht im Scope).
- Struktur: **Single-Crate-Start** (`ignis-core` als Lib + `examples/scan.rs`).
  Aufteilen in Workspace-Crates erst wenn eine zweite Konsumenten-Schicht konkret entsteht
  (→ ADR-001).

## Tech-Stack (fix)

| Layer          | Wahl                                                        |
|----------------|-------------------------------------------------------------|
| Core           | Rust, Edition **2021** (→ ADR-008)                          |
| CLI/TUI        | ratatui + crossterm + clap                                  |
| HTTP-API       | Axum auf `127.0.0.1:7337`, Bearer-Token-Auth (→ ADR-005)    |
| Tray           | Tauri 2 + React **18.3** (→ ADR-003) + TypeScript + Vite    |
| Persistenz     | In-Memory + Re-Scan-on-Change — **keine SQLite** (→ ADR-002) |
| Installer      | Tauri Bundler (MSI + NSIS)                                  |
| Ziel-Plattform | Windows 11. Core portabel halten für späteren Linux-Support. |

## Constraints (hart)

- **Read-only auf JSONL-Files.** Niemals schreiben, umbenennen, löschen.
- **Keine Cloud, keine Telemetrie, keine Accounts.** Alle Daten bleiben lokal.
- **Keine Background-Processes außer Tray-App.** Kein eigener Windows-Service im MVP.
- **Graceful Degradation.** Fehlende Files, korrupte Zeilen, unbekannte Modelle: loggen,
  überspringen, weitermachen. Nie crashen.
- **Pricing ist Daten, kein Code** — aber embedded Default (→ ADR-004).
- **Keine `unwrap()` in Production-Code.** Nur in Tests erlaubt.
- **`cargo clippy -- -D warnings` und `cargo fmt --check` müssen clean sein.**
- **Position-Tracking pro JSONL-File von Anfang an** (→ ADR-011, Parallel-Sessions-Robustheit).

## Code-Stil

- **Rust:** `cargo fmt`, `cargo clippy -- -D warnings`, `thiserror` für Fehler-Enums,
  `anyhow` nur an Binary-Grenzen.
- **TypeScript:** strict mode, ESLint + Prettier, keine `any` ohne Kommentar.
- **Commits:** Conventional Commits (`feat:`, `fix:`, `refactor:`, `docs:`, `test:`,
  `chore:`, `perf:`).
- **Ein Modul = eine Verantwortung.** Über 300 Zeilen = Refactoring-Signal.

## Session-Disziplin

- Bevor Kontext knapp wird: `PROGRESS.md` + `NEXT.md` schreiben, committen, pushen. Kein
  Arbeiten bis zum letzten Token.
- Nach jedem abgeschlossenen logischen Schritt: `PROGRESS.md` updaten + Commit.
- Nicht-triviale Entscheidung: neuer ADR in `DECISIONS.md` (Datum, Kontext, Alternativen,
  Begründung).
- Push nach jedem Commit.
- Vor jedem PR: `README.md` auf aktuellen Stand bringen (Status-Tabelle, Quick-start,
  neue Features/Commands).

## Agenten-Layout

Nur drei, unter `.claude/agents/` (→ ADR-006):

- **lead_engineer** — Architektur, Reviews, `DECISIONS.md`, Konsistenz, Nein zu Feature-Creep.
- **implementer** — Produktiv-Code (Rust-Core, CLI, API, Tauri-Host, React-UI).
- **qa_docs** — Tests, `fixtures/`, Inline-Rustdoc, `docs/`, `README`, `PROGRESS.md`-Pflege.

## Design-Sprache (Kurzfassung)

Warme Claude-Ästhetik: Terrakotta-Akzent (`#C15F3C`) auf warmem Dunkel-Grund (`#1F1E1B`).
Ein Akzent, nicht fünf. Keine Emoji-Ikonografie. Details in `docs/design-system.md`.

## Pflicht-Dateien

| Datei            | Zweck                                                   |
|------------------|---------------------------------------------------------|
| `CLAUDE.md`         | diese Datei — Projekt-Kontext                          |
| `PROGRESS.md`       | Phasen-/Milestone-Fortschritt                          |
| `NEXT.md`           | Der **eine** nächste konkrete Schritt                  |
| `BUGFIX-PROGRESS.md`| Offene Bugs/Fehlerbehandlungs-Lücken aus Code-Reviews  |
| `DECISIONS.md`      | ADR-light: jede nicht-triviale Entscheidung            |
| `CHANGELOG.md`      | Keep-a-Changelog, wird bei Release-Tags aktualisiert   |

## Nicht-Ziele (MVP)

- Kein Support für Cursor, Codex oder andere Provider (erst nach v1.0).
- Kein Light-Mode (v0.3+).
- Kein OpenUsage-Schema-Kompat — API-Schema ist eigenständig.
- Keine Auto-Update-Logik (kommt v1.0).
