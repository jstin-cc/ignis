# DECISIONS

Architecture Decision Records (ADR-light). Jede nicht-triviale Entscheidung mit Datum,
Kontext, Alternativen und Begründung. Spätere Änderungen werden durch neue ADRs
überschrieben (Status: *Superseded by ADR-NNN*), nicht durch Edit der alten.

Nummerierung aufsteigend. Status: `Accepted` · `Superseded` · `Rejected` · `Proposed`.

---

## ADR-001 — Single-Crate-Start statt Workspace-First

- **Datum:** 2026-04-17
- **Status:** Accepted
- **Kontext:** `INITIAL_PROMPT.md` skizziert eine 4-Crate-Workspace-Struktur
  (`winusage-core`, `-api`, `-cli`, `-tray`). Für den MVP gibt es aber nur einen
  Konsumenten der Kern-Logik.
- **Alternativen:**
  - (A) Workspace-First wie im Prompt — klare Grenzen ab Tag 1, aber viel Boilerplate,
        bevor wir überhaupt wissen, ob die Modul-Schnitte stimmen.
  - (B) **Single-Crate mit `src/lib.rs` + `examples/scan.rs`** — Splitting erst wenn eine
        zweite Konsumenten-Schicht (API oder Tray) konkret gebaut wird.
- **Entscheidung:** (B). `winusage-core` lebt zunächst als flache Lib mit Beispielen.
- **Begründung:** Vermeidet Premature-Abstraction. Modul-Grenzen innerhalb einer Lib sind
  billig zu verschieben, Crate-Grenzen nicht. Re-Organisation zum Workspace ist later ein
  trivialer Refactor (Cargo Workspace + `cargo move` / manueller Split).
- **Folgen:** Architektur-Diagramme müssen klarmachen, dass die im Prompt gezeigte
  Crate-Struktur das **Ziel-Modell** ist, nicht der Phase-0-Stand.

## ADR-002 — Keine SQLite im MVP

- **Datum:** 2026-04-17
- **Status:** Accepted
- **Kontext:** Der Prompt schlägt SQLite als Cache für JSONL-Daten vor.
- **Alternativen:**
  - (A) SQLite-Cache mit Schema und Migrations.
  - (B) **In-Memory-Zustand + Re-Scan-on-Change** via `notify` crate.
  - (C) JSON-Snapshots auf Platte pro Aggregation.
- **Entscheidung:** (B).
- **Begründung:** JSONL-Files sind die einzige Wahrheit. Ein Cache bringt Konsistenz-
  Komplexität (Invalidierung, Schema-Migrationen), die der MVP nicht braucht.
  Position-Tracking pro File (→ ADR-011) liefert inkrementelle Re-Scans ohne DB.
- **Folgen:** Startup-Zeit skaliert mit Gesamt-JSONL-Volumen. Bei spürbaren Ladezeiten
  (> 500 ms) wird in v0.2/v0.3 ein persistenter Snapshot erwogen (neuer ADR).

## ADR-003 — React 18.3 statt React 19

- **Datum:** 2026-04-17
- **Status:** Accepted
- **Kontext:** Der Prompt nennt React 19. React 18.3 ist aber der stabile LTS-Branch mit
  dem breitesten Tauri-/Tooling-Ökosystem.
- **Alternativen:** React 19 · **React 18.3** · Preact / SolidJS.
- **Entscheidung:** React 18.3.
- **Begründung:** MVP soll auf stabilem Fundament stehen. Keine RC-/Migrations-Surprises.
  Upgrade auf 19 ist später ein isolierter Schritt.
- **Folgen:** Abhängigkeiten (z.B. Recharts) werden gegen React-18-Kompatibilität gepinnt.

## ADR-004 — Pricing: Embedded Default + UI-Warning, Updates nur via App-Update

- **Datum:** 2026-04-17
- **Status:** Accepted
- **Kontext:** Pricing-Daten veralten. Drei Optionen:
  (a) externer Endpoint mit Remote-Fetch, (b) lokale `pricing.json` manuell editierbar,
  (c) **embedded Default + UI-Warning bei unbekanntem Modell, Updates via App-Release**.
- **Entscheidung:** (c).
- **Begründung:** Kein Netz-Zugriff zur Laufzeit (Constraint: keine Cloud). Kein Trust-
  /Signing-Problem für eine remote `pricing.json`. UI zeigt klar, welche Modelle nicht
  berücksichtigt werden, sodass der Nutzer ein App-Update ziehen kann.
- **Folgen:** `pricing.json` bleibt *im Binary* eingebettet. Ein Override-Mechanismus
  (lokale `pricing.local.json`) wird separat geprüft (Backlog), ist aber nicht MVP.

## ADR-005 — API-Auth: 127.0.0.1-Bind + Origin-Check + Bearer-Token

- **Datum:** 2026-04-17
- **Status:** Accepted
- **Kontext:** Die HTTP-API exponiert Usage-/Cost-Daten. Drei Maßnahmen sind
  in Kombination schlank und belastbar.
- **Entscheidung:** **Alle drei:**
  1. Server bindet ausschließlich an `127.0.0.1:7337`, nie `0.0.0.0`.
  2. Origin-Header-Check gegen eine konfigurierbare Allowlist
     (Default: leer → nur Same-Origin/No-Origin-Requests, also CLI/Editor-Plugins).
  3. Bearer-Token in Config-Datei unter `%APPDATA%\winusage\auth-token.txt` mit
     restriktiven Windows-ACLs (nur der anlegende User hat Lese-Rechte).
- **Begründung:** Defense-in-Depth. Jede Schicht fängt eine eigene Angriffsklasse ab
  (lokale Netzwerk-Nachbarn, Browser-CSRF, andere User-Accounts auf demselben Host).
- **Folgen:** Token-Erzeugung beim ersten Start; Rotation via CLI-Kommando. ACL-Setup
  via `icacls` im Installer und defensiv beim Runtime-Start.

## ADR-006 — Drei Agenten statt sieben

- **Datum:** 2026-04-17
- **Status:** Accepted
- **Kontext:** Prompt schlägt 7 spezialisierte Agenten vor. Für ein Solo-Projekt in
  frühem Stadium ist das Overhead.
- **Entscheidung:** **Drei Rollen** in `.claude/agents/`:
  - `lead_engineer` — Architektur, Reviews, `DECISIONS.md`.
  - `implementer` — sämtlicher Produktiv-Code (Rust-Core, CLI, API, Tauri-Host, React-UI).
  - `qa_docs` — Tests, Fixtures, Inline-Docs, `docs/`, `PROGRESS.md`-Pflege.
- **Begründung:** Drei reichen, um Zuständigkeiten (Design vs. Build vs. Verify) zu
  trennen. Feinere Spezialisierung ist Premature-Optimization.
- **Folgen:** Der Satz "Kein Agent schreibt Code in einem Crate, das ihm nicht zugeordnet
  ist" aus dem Prompt entfällt; `implementer` ist für alle Crates zuständig.

## ADR-007 — CI-Minimal-Workflow ab Phase 1

- **Datum:** 2026-04-17
- **Status:** Accepted
- **Kontext:** Ein minimaler GitHub-Actions-Workflow ist billig und sichert die
  harten Code-Stil-Constraints automatisch.
- **Entscheidung:** `.github/workflows/ci.yml` mit genau drei Schritten ab Phase 1:
  `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- **Begründung:** Keine Matrix, keine Release-Pipeline im MVP — nur Guards. Windows-
  Runner, da msvc-Toolchain das primäre Ziel ist.
- **Folgen:** Workflow ist nicht Teil des Phase-0-Initial-Commits; kommt im ersten
  Phase-1-PR mit echtem Code rein.

## ADR-008 — Rust-Edition 2021

- **Datum:** 2026-04-17
- **Status:** Accepted
- **Kontext:** Prompt nennt Edition 2024. Edition 2024 ist jung und bringt keine
  MVP-relevanten Features, aber neue Lints/Semantik-Änderungen.
- **Entscheidung:** Edition 2021.
- **Begründung:** Breiteste Crate-Kompatibilität, stabilste Toolchain-Interaktion. Upgrade
  auf 2024 ist `cargo fix --edition` plus neue Lints — machbar, aber ohne Gewinn im MVP.
- **Folgen:** `Cargo.toml` fixiert `edition = "2021"`, `rust-version = "1.75"` als Floor.

## ADR-009 — Genau eine `CLAUDE.md` im Repo-Root

- **Datum:** 2026-04-17
- **Status:** Accepted
- **Kontext:** Prompt diskutiert zusätzlich eine `.claude/CLAUDE.md`.
- **Entscheidung:** Nur `CLAUDE.md` im Repo-Root. `.claude/` enthält ausschließlich
  Agent-Definitionen unter `.claude/agents/`.
- **Begründung:** Eine Quelle der Wahrheit, keine Duplikate, kein Symlink-Trickserei auf
  Windows. Claude Code liest `CLAUDE.md` zuverlässig aus dem Repo-Root.
- **Folgen:** `.claude/CLAUDE.md` existiert nicht und wird auch nicht als Symlink angelegt.

## ADR-010 — Session-Block-Logik erst empirisch dokumentieren

- **Datum:** 2026-04-17
- **Status:** Accepted
- **Kontext:** Der "62% used"-Session-Block-Indikator im MVP-Wireframe ist Spekulation,
  solange wir das JSONL-Format (und Claude Codes tatsächliche Billing-Windows) nicht
  empirisch validiert haben.
- **Entscheidung:** Session-Block-Logik wandert nach **v0.2**. Im MVP erscheint kein
  Prozent-Balken; der Platz bleibt leer oder zeigt nur die aktuelle Session-Dauer.
- **Begründung:** Lieber kein Signal als ein falsches. Vor jeder Implementierung muss
  `docs/jsonl-format.md` dokumentieren, wie Sessions/Blocks aus dem JSONL ableitbar sind.
- **Folgen:** Das Feature ist in `PROGRESS.md` unter Phase 2 geparkt.

## ADR-012 — Provider-Trait als Erweiterungspunkt für künftige Datenquellen

- **Datum:** 2026-04-20
- **Status:** Accepted
- **Kontext:** Phase 3 sieht Vorbereitung für Cursor, Codex und andere KI-Code-Editoren
  vor. Diese Editoren schreiben ggf. Nutzungslogs in einem anderen Format oder an einem
  anderen Pfad. Ohne Abstraktion müsste `scanner.rs` und `parser.rs` direkt angepasst
  oder dupliziert werden.
- **Alternativen:**
  - (A) **Feature-Flags** — je Provider ein Compile-Flag. Einfach, aber nicht erweiterbar
        ohne Quellcode-Änderungen.
  - (B) **`Provider`-Trait** — ein Objekt kapselt `data_root()`, `parse_line()` und ein
        optionales `collect()`-Override. Provider werden zur Laufzeit registriert und
        können aggregiert werden (Multi-Provider-Snapshot).
  - (C) Config-File-Ansatz — externe TOML-Konfiguration beschreibt Pfade und Format.
        Sehr flexibel, aber Over-Engineering für Phase 3.
- **Entscheidung:** (B). Trait in `src/provider.rs`; `ClaudeCodeProvider` als
  Referenz-Implementierung; bestehende `scan_all`-API bleibt unverändert.
- **Begründung:** Ein Trait gibt klare Grenzen vor, ist zero-cost im normalen Fall und
  lässt sich in Tests mock-en. `ClaudeCodeProvider` ist die einzige Implementierung bis
  ein zweiter Provider konkret entsteht — verhindert aber, dass dann die ganze Architektur
  umgebaut werden muss.
- **Folgen:** `src/provider.rs` wird aus `lib.rs` re-exportiert. Scanner und Aggregation
  bleiben unverändert; sie arbeiten weiterhin direkt mit `UsageEvent`-Vektoren.
  Multi-Provider-Fusion (Summe über mehrere `collect()`-Ergebnisse) wird erst dann
  implementiert, wenn ein zweiter Provider real existiert (neuer ADR).

## ADR-011 — Position-Tracking pro File als Design-Anforderung

- **Datum:** 2026-04-17
- **Status:** Accepted
- **Kontext:** Mehrere parallele Claude-Code-Sessions schreiben in verschiedene
  JSONL-Files. Ein naives "letztes-mtime"-Re-Scan verliert Zeilen, wenn zwei Files
  zwischen Scans wachsen.
- **Entscheidung:** Scanner hält pro-File-Position (Byte-Offset + inode/File-ID) **ab
  Phase 1**. Re-Scan liest nur das Delta seit der letzten Position. Wird in
  `docs/architecture.md` als explizite Design-Anforderung verankert.
- **Begründung:** Nachrüsten nach einem naiven Design ist teurer als gleich richtig
  anlegen. Das ist der Fall, wo "ein bisschen mehr Struktur vorne weg" Abgleich-Bugs
  spart, die sonst schwer zu reproduzieren wären.
- **Folgen:** Position-Map wird In-Memory gehalten (konsistent mit ADR-002). Bei Startup
  wird die Map aus einem Full-Scan rekonstruiert; persistente Position-Maps sind optional
  später (neuer ADR, falls Startup-Zeiten problematisch werden).
