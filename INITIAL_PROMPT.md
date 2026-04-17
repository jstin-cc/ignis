# WinUsage — Claude Code Usage Tracker für Windows

## Projekt-Mission
Ein Windows-nativer, lokaler Usage-Tracker für Claude Code. Liest die JSONL-Logs aus
`%USERPROFILE%\.claude\projects\` und zeigt Token-Verbrauch, Kosten und Session-Status in
drei Oberflächen an:

1. **System-Tray-App** (primär) — klickt man aufs Tray-Icon, klappt ein Panel auf mit
   aktuellem Stand.
2. **Terminal-CLI mit Live-TUI** (`winusage watch`) — für Entwickler, die eh im Terminal
   sind.
3. **Lokaler HTTP-API-Endpunkt** — damit Statuslines, Editor-Plugins oder eigene Scripts
   die Daten konsumieren können.

Inspiriert von OpenUsage (macOS-only), aber **nicht kompatibel** — wir designen das
API-Schema frisch und nehmen uns die Freiheit, es auf Claude Code zu optimieren.

Das Repository ist **zunächst privat**. Wenn es später public werden soll, ist eine
Umstellung trivial — aber bis dahin müssen wir uns um Screenshots, Contributor-Guides
oder Marketing-Material nicht kümmern. Fokus auf Funktionalität und interne Doku.

---

## 🧠 Kontext-Persistenz (KRITISCH — lies das zuerst)

Dieses Projekt ist zu groß für eine einzelne Claude-Code-Session. Der Kontext WIRD
zwischendurch verloren gehen. Damit das nicht zum Problem wird, gilt folgende Disziplin:

### Pflicht-Dateien (werden laufend aktualisiert)

| Datei            | Zweck                                                              |
|------------------|--------------------------------------------------------------------|
| `PROGRESS.md`    | Was ist done / in-progress / blocked pro Milestone.                |
| `NEXT.md`        | Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird. |
| `DECISIONS.md`   | Architecture Decision Records (ADR-light). Jede wichtige Entscheidung mit Datum + Begründung. |
| `CHANGELOG.md`   | Keep-a-Changelog-Format, wird bei jedem Release-Tag aktualisiert.  |
| `CLAUDE.md`      | Permanenter Projekt-Kontext für jede Session. Enthält Stil-Regeln, Architektur-Überblick, aktive Constraints. |

### Session-Regeln

- **Jede neue Session startet mit:** `cat CLAUDE.md PROGRESS.md NEXT.md` — in dieser
  Reihenfolge. Erst dann wird irgendwas anderes gemacht.
- **Bevor der Kontext knapp wird:** aktiv eine Zusammenfassung in `PROGRESS.md` und
  `NEXT.md` schreiben, committen, pushen. **Kein Arbeiten bis zum letzten Token.** Lieber
  früh abschließen als mittendrin den Faden verlieren.
- **Nach jedem abgeschlossenen logischen Schritt:** `PROGRESS.md` updaten + Git-Commit.
- **Wenn eine Entscheidung getroffen wird, die nicht trivial ist** (z.B. Lib-Wahl,
  Datenmodell-Änderung, Architektur-Shift): neuer Eintrag in `DECISIONS.md` mit Datum,
  Kontext, Alternativen, Begründung.

---

## 🔧 Git & GitHub-Workflow

Claude Code legt das Repository selbst an und verwaltet es. Ablauf beim allerersten Start:

1. `gh auth status` prüfen — falls nicht eingeloggt, den Nutzer auffordern `gh auth login`
   auszuführen.
2. `git init` im Projekt-Root.
3. `.gitignore` anlegen (Rust + Node + Tauri + Windows + IDE-Artefakte).
4. Initial-Commit mit der Dokumentation aus Phase 0 (Prompt-Review, docs/, CLAUDE.md etc.).
5. `gh repo create winusage --private --source=. --remote=origin --push`
6. Branch-Schutz ist für ein Solo-Projekt unnötig. Feature-Work kann auf Branches wie
   `feat/jsonl-parser` laufen, muss aber nicht.

### Later-public-Checkliste (erst relevant, wenn Nutzer "mach's public" sagt)

Bis dahin ignorieren. Wenn der Nutzer das Repo später öffentlich machen will:
- LICENSE-File hinzufügen (Nutzer fragen: MIT / Apache-2.0 / andere)
- README mit Screenshots, Installation-Section, Quick-Start
- Sicherstellen, dass keine API-Keys, Pfade oder persönliche Infos in der Git-History sind
- `CONTRIBUTING.md` anlegen
- GitHub Issues-Templates
- `gh repo edit --visibility public`

### Commit-Rhythmus

- Commit nach jedem abgeschlossenen logischen Schritt, **nicht** nach jeder Datei.
- **Push nach jedem Commit.** Lokale Commits, die nicht gepusht sind, gehen bei
  Kontextverlust unter.
- Conventional Commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`, `perf:`.
- `PROGRESS.md` wird im selben Commit aktualisiert wie der Code, auf den er sich bezieht.
- Milestone-Tags: `v0.1.0-mvp`, `v0.2.0`, etc.

### Dokumentations-Rhythmus auf GitHub

- `README.md` bleibt im privaten Repo schlank — eher ein internes Briefing als ein
  Marketing-Dokument.
- `docs/` wächst organisch mit dem Code.
- Keine GitHub Releases im privaten Stadium, nur Tags.

---

## 🛠 Toolchain-Setup (Phase 0, noch vor allem anderen)

Der Nutzer bestätigt: **Rust ist noch nicht installiert.** Node und `gh` sind da.

Claude Code führt den Nutzer durch:

1. Nutzer bittet, `rustup` zu installieren via https://rustup.rs (Windows-Installer).
2. Nach Installation: `rustc --version` und `cargo --version` prüfen.
3. Stable-Toolchain sicherstellen: `rustup default stable`, `rustup update`.
4. Wichtige Components: `rustup component add clippy rustfmt`.
5. Für Tauri werden zusätzlich Microsoft C++ Build Tools benötigt — wenn Tauri-CLI
   installiert wird, gibt `cargo install tauri-cli` einen klaren Fehler aus, falls was
   fehlt. Nutzer ggf. zu Build-Tools-Install führen (Visual Studio Installer →
   "Desktop development with C++").
6. `cargo install tauri-cli --version "^2.0"` für das Tauri-CLI.
7. `node --version` und `gh auth status` nochmal kurz verifizieren, damit die Umgebung
   komplett dokumentiert ist.
8. Installations-Status wird in `PROGRESS.md` unter "Toolchain" festgehalten mit Versionen.

**Wichtig:** Claude Code installiert nichts stillschweigend. Jeder Install-Befehl wird
dem Nutzer vorher angekündigt, der Nutzer führt ihn selbst aus oder bestätigt explizit.

---

## Scope & Entscheidungen (fix)

| Entscheidung         | Wert                                           |
|----------------------|------------------------------------------------|
| MVP-Provider         | **Nur Claude Code**. Keine Cursor/Codex-Vorarbeit. |
| API-Schema           | **Eigenes**, dokumentiert unter `docs/api.md`. |
| API-Port             | `localhost:7337` (frei wählbar via Config).    |
| CLI-Sprache          | Rust + ratatui + crossterm.                    |
| Tray-Sprache         | Tauri 2 + React 19 + TypeScript + Vite.        |
| Core-Sprache         | Rust (Edition 2024).                           |
| Persistenz           | `%APPDATA%\winusage\` — JSON-Configs, SQLite für Cache. |
| Installer            | Tauri Bundler (MSI + NSIS).                    |
| Ziel-Plattform MVP   | Windows 11. Core portabel halten für späteren Linux-Support. |
| Repo-Hosting         | GitHub (private), über `gh` CLI von Claude Code angelegt. |
| Design-Sprache       | Claude-Ästhetik (warmes Terrakotta auf dunklem Grund). |

---

## 🎨 Design-System

### Philosophie

Claude.ai hat eine bewusst **warme, menschliche Ästhetik** — statt klinischem Blau
arbeitet Anthropic mit Terrakotta-Orange auf cremigen oder tiefen, warmen Dunkel-Tönen.
Das wollen wir übernehmen: ein Tool, das sich wie ein vertrauter Arbeitsbegleiter anfühlt,
nicht wie ein Dashboard.

**Leitprinzipien:**
- **Warm statt kalt.** Keine Pure-Black-Backgrounds, keine klinischen Blautöne.
- **Dicht, aber nicht erdrückend.** Zahlen und Details sichtbar, aber mit Luft dazwischen.
- **Ein Akzent, nicht fünf.** Terrakotta ist DIE eine Signalfarbe. Sparsam einsetzen.
- **Typografie trägt.** Kräftige Zahlen, zurückhaltende Labels.
- **Keine Emoji-Ikonografie** in der UI — wir sind kein Konsumenten-Tool.

### Farbpalette (Dark Mode, primäres Theme)

```
# Backgrounds
--bg-base:        #1F1E1B   /* Haupt-Background, warmes Anthrazit */
--bg-elevated:    #292724   /* Karten, Panels */
--bg-overlay:     #34312C   /* Modals, Tooltips */

# Borders & Lines
--border-subtle:  #3D3A34
--border-default: #524E46

# Text
--text-primary:   #F4F3EE   /* Haupt-Text, "Pampas" aus Claude-Palette */
--text-secondary: #B1ADA1   /* Labels, Metadaten, "Cloudy" */
--text-muted:     #7A766D   /* Disabled, sekundäre Info */

# Akzent (Terrakotta)
--accent:         #C15F3C   /* "Crail" — Anthropics Signature-Ton */
--accent-hover:   #D47551
--accent-muted:   #8B4428   /* Für Fortschritts-Balken etc. */

# Status
--success:        #7A9B76   /* Gedämpftes Grün, warm */
--warning:        #D4A574   /* Bernstein */
--danger:         #C06862   /* Gedämpftes Rot, nicht schreiend */

# Charts (Token-Typ-Unterscheidung)
--chart-input:       #C15F3C  /* Akzent */
--chart-output:      #D4A574  /* Warm Amber */
--chart-cache-read:  #7A9B76  /* Sage */
--chart-cache-write: #8B6B9B  /* Gedämpftes Lila */
```

Light Mode kommt später (v0.3+), nicht im MVP.

### Typografie

- **UI-Font:** System-Sans (Segoe UI Variable auf Windows). Kein Web-Font im Tray.
- **Mono-Font:** JetBrains Mono oder Cascadia Code (System) für Zahlen und CLI.
- **Zahlen in Panels:** tabular-nums aktivieren (`font-variant-numeric: tabular-nums`),
  damit sich Stellen nicht verschieben bei Live-Updates.
- **Hierarchie:** 24px Hero-Zahlen, 14px Labels, 12px Metadata. Keine 5 verschiedenen
  Größen.

### Spacing & Geometrie

- Spacing-Scale: 4 / 8 / 12 / 16 / 24 / 32 px.
- Corner-Radius: 6px für Karten, 4px für Buttons, 2px für Chips.
- Border: 1px, `--border-subtle` als Default.

### Tray-Panel-Layout (Wireframe)

```
┌─────────────────────────────────────────┐
│  WinUsage              ⚙  ×             │  Header: 48px
├─────────────────────────────────────────┤
│  TODAY                                  │
│  $2.43              ████████░░░░  62%   │  Session-Block Progress
│  1.2M tokens · 14 sessions              │
├─────────────────────────────────────────┤
│  THIS MONTH                             │
│  $48.17                                 │
│  29.8M tokens · 182 sessions            │
├─────────────────────────────────────────┤
│  ACTIVE SESSION                         │
│  my-project          2h 14m             │
│  312k tokens · $0.71                    │
├─────────────────────────────────────────┤
│  [ Open Dashboard ]  [ CLI: winusage ]  │  Footer: Actions
└─────────────────────────────────────────┘
```

Breite ~360px, Höhe content-fit (max. ~520px). Kein Scroll im MVP — wenn mehr Inhalt
kommt, wird ein Dashboard-Fenster geöffnet.

### CLI-TUI-Layout (ratatui, für `winusage watch`)

```
┌ WinUsage watch ─────────────────────── 14:23:05 ─┐
│                                                  │
│  ╭─ Today ─────────────╮ ╭─ Session Block ────╮  │
│  │  $2.43              │ │  62% used          │  │
│  │  1.2M tokens        │ │  resets in 1h 52m  │  │
│  ╰─────────────────────╯ ╰────────────────────╯  │
│                                                  │
│  ╭─ By Model ──────────────────────────────────╮ │
│  │  claude-opus-4-7     $1.82   890k tokens    │ │
│  │  claude-sonnet-4-6   $0.61   310k tokens    │ │
│  ╰─────────────────────────────────────────────╯ │
│                                                  │
│  ╭─ Burn Rate ─────────────────────────────────╮ │
│  │  ▁▂▃▅▇█▇▅▃▂▁  avg: 14k tok/min              │ │
│  ╰─────────────────────────────────────────────╯ │
│                                                  │
│ [q] quit  [r] refresh  [d] daily  [m] monthly    │
└──────────────────────────────────────────────────┘
```

Farbschema im Terminal nutzt dieselben Akzent-Farben via ANSI-TrueColor.

---

## Architektur

```
winusage/
├── Cargo.toml              # Workspace
├── crates/
│   ├── winusage-core/      # Scanner, Parser, Pricing, Models, Aggregation
│   ├── winusage-api/       # Axum HTTP-Server auf :7337
│   ├── winusage-cli/       # ratatui TUI + Subcommands (daily/monthly/watch)
│   └── winusage-tray/      # Tauri-Host (Rust-Seite der Desktop-App)
├── apps/
│   └── tray-ui/            # React-Frontend für Tauri (Vite)
├── .claude/
│   ├── agents/             # Agent-Definitionen
│   └── CLAUDE.md           # Projekt-Kontext (→ wird zu /CLAUDE.md symlinked oder dupliziert)
├── docs/
│   ├── architecture.md
│   ├── api.md
│   ├── jsonl-format.md
│   ├── pricing.md
│   └── design-system.md    # Design-Tokens, Komponenten-Prinzipien
├── fixtures/               # Anonymisierte JSONL-Samples für Tests
├── CLAUDE.md               # Projekt-Kontext, jede Session liest diese Datei
├── PROGRESS.md
├── NEXT.md
├── DECISIONS.md
├── CHANGELOG.md
├── README.md
└── .gitignore
```

**Design-Prinzip:** `winusage-core` ist die einzige Stelle, an der JSONL gelesen und
Kosten berechnet werden. API, CLI und Tray sind reine Präsentations-/Transport-Schichten.

---

## Kern-Datenmodell (Rust, indikativ)

```rust
pub struct UsageEvent {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub project_path: PathBuf,
    pub model: ModelId,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
}

pub struct UsageSummary {
    pub range: TimeRange,
    pub total_cost_usd: Decimal,
    pub by_model: HashMap<ModelId, ModelUsage>,
    pub by_project: HashMap<PathBuf, ProjectUsage>,
    pub event_count: u64,
}

pub struct ModelPricing {
    pub input_per_mtok: Decimal,
    pub output_per_mtok: Decimal,
    pub cache_write_per_mtok: Decimal,
    pub cache_read_per_mtok: Decimal,
}
```

---

## Feature-Roadmap

### MVP (v0.1.0) — "Es liest und zeigt was an"
- JSONL-Scanner mit File-Watching (`notify` crate)
- Pricing-Engine mit externer `pricing.json`
- Aggregation: heute, diese Woche, dieser Monat, aktuelle Session
- CLI: `winusage daily`, `winusage monthly`, `winusage session`
- Tray-App mit Basis-Panel (Layout siehe oben)
- HTTP-API mit 3 Endpoints: `/health`, `/v1/summary`, `/v1/sessions`
- Installer (MSI)
- Dokumentation

### v0.2.0 — "Live und smart"
- `winusage watch` mit Live-TUI
- 5-Stunden-Billing-Windows (Session-Blocks)
- Burn-Rate + Projektionen
- Tray: Per-Projekt-Breakdown, Chart (Recharts)
- Notifications bei Limit-Schwellen
- Auto-Start bei Windows-Login (optional)

### v1.0.0 — "Plugin-ready"
- Provider-Plugin-Trait extrahiert (Vorbereitung für Cursor/Codex)
- Export: CSV, JSON
- Heatmap-Ansicht im Tray
- Auto-Update via Tauri Updater

Provider für Cursor/Codex kommen **erst nach v1.0**.

---

## Agent-Team (in `.claude/agents/`)

- **lead_engineer** — Architektur, PR-Reviews, Konsistenz. Darf nein sagen zu
  Feature-Creep. Verwaltet `DECISIONS.md`.
- **rust_backend_engineer** — `winusage-core`, `winusage-api`. JSONL-Parser,
  Pricing-Engine, Axum-Server.
- **frontend_engineer** — `apps/tray-ui/`. React 19, TypeScript strict. Setzt
  Design-System aus `docs/design-system.md` technisch um.
- **cli_engineer** — `winusage-cli`. ratatui-TUI, clap-Subcommands.
- **windows_integration** — Tray-Icon, Auto-Start, Notifications, Installer, Pfade.
- **qa_agent** — Test-Strategie, `fixtures/` mit realen JSONL-Samples,
  Integration-Tests.
- **docs_agent** — README, `docs/`, Inline-Rustdoc, `PROGRESS.md`-Pflege nach größeren
  Merges.

**Koordinationsregel:** Kein Agent schreibt Code in einem Crate, das ihm nicht zugeordnet
ist. Ausnahme: `lead_engineer` darf überall lesen und kommentieren, aber selbst nur
Architektur-Dokumente und Workspace-Config editieren.

---

## Wichtige Constraints

- **Read-only auf JSONL-Files.** Wir dürfen Claude Codes Logs niemals schreiben, umbenennen
  oder löschen.
- **Keine Cloud, keine Telemetrie, keine Accounts.** Alle Daten bleiben lokal.
- **Keine Background-Processes außer dem Tray-App-Prozess.** Kein eigener Windows-Service
  im MVP.
- **Graceful Degradation.** Fehlende Files, korrupte Zeilen, unbekannte Modelle: loggen,
  überspringen, weitermachen. Nie crashen.
- **Pricing ist Daten, kein Code.** `pricing.json` beim ersten Start aus embedded Default
  erzeugen, vom Nutzer editierbar.
- **Keine `unwrap()` in Production-Code.** Nur in Tests erlaubt.
- **`cargo clippy -- -D warnings` und `cargo fmt --check` müssen clean sein.**

---

## Development-Umgebung

- OS: Windows 11
- Shell: Git Bash in Cursor-Terminal
- Projekt-Root: `D:\.claude\projects\winusage\`
- Rust-Toolchain: **wird in Phase 0 installiert** (siehe Toolchain-Setup oben)
- Node: LTS (bereits vorhanden)
- GitHub CLI: `gh` (bereits vorhanden und authentifiziert)
- Paste in Claude-Code-Terminal ist unzuverlässig — bei längeren Inputs file-basiert
  arbeiten.

---

## Code-Stil

- **Rust:** `cargo fmt`, `cargo clippy -- -D warnings`, keine `unwrap()` außerhalb Tests,
  `thiserror` für Fehler-Enums, `anyhow` nur an Binary-Grenzen.
- **TypeScript:** strict mode, ESLint + Prettier, keine `any` ohne Kommentar.
- **Commit-Messages:** Conventional Commits.
- **Ein Modul = eine Verantwortung.** Über 300 Zeilen = Refactoring-Signal.

---

## 🚀 Deine erste Aufgabe (als lead_engineer)

Bevor auch nur eine Zeile Produktivcode entsteht, liefere in dieser Reihenfolge:

1. **Kritisches Review dieses Prompts.** Welche Annahmen sind wacklig? Welche
   Entscheidungen fehlen noch? Stelle konkrete Rückfragen an den Nutzer.
2. **Toolchain-Setup.** Rust ist noch nicht installiert — führe den Nutzer durch die
   Installation (siehe Toolchain-Setup-Abschnitt). Dokumentiere die Versionen in
   `PROGRESS.md` unter "Toolchain".
3. **GitHub-Setup.** `gh auth status` prüfen. Repo-Name bestätigen (default: `winusage`).
   Repo wird private angelegt.
4. **Projekt-Struktur-Skelett anlegen** — nur die Ordner und leeren `.gitkeep`-Files,
   plus die 5 Pflicht-Dokumente (`CLAUDE.md`, `PROGRESS.md`, `NEXT.md`, `DECISIONS.md`,
   `CHANGELOG.md`) mit initialem Inhalt.
5. **JSONL-Format-Untersuchung.** Öffne eine echte JSONL-Datei unter
   `%USERPROFILE%\.claude\projects\`, dokumentiere das Schema in `docs/jsonl-format.md`.
6. **Finales Daten-Modell** in `docs/architecture.md` mit Rust-Struct-Signaturen und
   Begründungen.
7. **API-Schema-Entwurf** in `docs/api.md`.
8. **Design-System-Dokumentation** in `docs/design-system.md` — Farbpalette (aus diesem
   Prompt übernehmen), Typografie, Spacing, Komponenten-Prinzipien.
9. **Agent-Definitionen** unter `.claude/agents/*.md`.
10. **Git-Init + Initial-Commit + private GitHub-Repo-Erstellung + Push.**
11. **`PROGRESS.md` updaten**, `NEXT.md` auf "Phase 1: MVP — Start mit winusage-core
    Scanner-Implementation" setzen.

Erst wenn ich (der Nutzer) 1–11 abgenommen habe, starten wir mit dem eigentlichen
MVP-Code. Kein Sprinten, keine Vorab-Implementierung.

Los geht's mit Punkt 1 — sei ehrlich und kritisch.
