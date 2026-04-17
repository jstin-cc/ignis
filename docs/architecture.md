# Architecture

Ziel dieses Dokuments: jeder Implementer kann ohne weitere Rückfragen die Grundstruktur
von `winusage-core`, die öffentlichen Typen und das Scanner-Verhalten bauen.

Verknüpfte Entscheidungen: `DECISIONS.md` (ADR-001 bis ADR-011). Format-Grundlage:
`docs/jsonl-format.md`.

---

## 1. Überblick

```
┌────────────────────────────────────────────────────────────────────┐
│                        winusage-core (Lib)                         │
│                                                                    │
│  ┌────────┐   ┌─────────┐   ┌──────────┐   ┌───────────────────┐   │
│  │ scanner│──▶│  parser │──▶│ aggregate│──▶│ snapshot (public) │   │
│  └────┬───┘   └─────────┘   └────┬─────┘   └─────────┬─────────┘   │
│       │                          │                   │             │
│       │     ┌──────────┐         │                   │             │
│       │     │ pricing  │─────────┘                   │             │
│       │     └──────────┘                             │             │
│       │                                              ▼             │
│  watch │  notify-fs                          consumers:            │
│       └──────────────────────────────────────── CLI · API · Tray   │
└────────────────────────────────────────────────────────────────────┘
```

- **scanner** — findet JSONL-Files, hält Position-Map, liefert Δ-Events.
- **parser** — dekodiert `UsageEvent` aus einer JSONL-Zeile (nur `type: assistant`).
- **aggregate** — baut und inkrementiert `UsageSummary` (pro Session, Tag, Monat, Modell,
  Projekt).
- **pricing** — Modell-ID → `ModelPricing`; Warning-Kanal für unbekannte IDs.
- **snapshot** — immutable, thread-safe Public-API für Consumer.

Konsumenten (CLI / API / Tray) greifen ausschließlich auf `Snapshot`/`SnapshotReader`
zu. Sie parsen weder JSONL noch kennen sie Pricing-Interna.

## 2. Code-Layout (Phase 1, Single-Crate → ADR-001)

```
winusage/
├── Cargo.toml                          # single crate, edition = "2021"
├── src/
│   └── lib.rs                          # Re-exports der Public-API
│       pub mod error;                  # thiserror-Enums
│       pub mod model;                  # UsageEvent, Snapshot, …
│       pub mod scanner;                # Discovery, Positionen, Watch
│       pub mod parser;                 # Line → UsageEvent
│       pub mod pricing;                # ModelPricing + embed-default
│       pub mod aggregate;              # Summaries
│       pub mod config;                 # Pfade, AuthToken, Ports
├── examples/
│   └── scan.rs                         # Dev-CLI: einmaliger Full-Scan + Dump
├── fixtures/
│   ├── happy-path.jsonl
│   ├── error-synthetic.jsonl
│   └── sidechain.jsonl
└── tests/
    └── scanner.rs                      # Integration: fixtures → Snapshot
```

Später (Phase 1 Ende oder Phase 2 Start), wenn die zweite Konsumenten-Schicht konkret
wird, splitten wir in Workspace:

```
winusage/
├── Cargo.toml                          # workspace
└── crates/
    ├── winusage-core/
    ├── winusage-cli/
    ├── winusage-api/
    └── winusage-tray/
```

Die Modulgrenzen in `winusage-core` entsprechen bereits diesem Ziel-Schnitt; ein Split
ist ein mechanischer `cargo mv`-Refactor, kein API-Neudesign.

## 3. Datenmodell (Rust)

```rust
// src/model.rs — stabilisiert vor Phase 1.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Stable identifier for a model. Kept as String — models emerge faster than we can
/// maintain an enum, and unknown models must not crash the parser.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModelId(pub String);

#[derive(Clone, Debug)]
pub struct UsageEvent {
    pub session_id: String,
    pub uuid: String,
    pub timestamp: DateTime<Utc>,
    pub project_path: PathBuf,
    pub git_branch: Option<String>,
    pub model: ModelId,
    pub is_sidechain: bool,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_creation_ephemeral_5m: u64,
    pub cache_creation_ephemeral_1h: u64,
    pub web_search_requests: u64,
    pub web_fetch_requests: u64,
}

#[derive(Clone, Debug, Default)]
pub struct ModelUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cost_usd: Decimal,
    pub event_count: u64,
}

#[derive(Clone, Debug, Default)]
pub struct ProjectUsage {
    pub total_cost_usd: Decimal,
    pub total_tokens: u64,
    pub session_count: u64,
}

#[derive(Clone, Debug)]
pub struct SessionState {
    pub session_id: String,
    pub project_path: PathBuf,
    pub git_branch: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub event_count: u64,
    pub total_cost_usd: Decimal,
    pub by_model: BTreeMap<ModelId, ModelUsage>,
}

#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub total_cost_usd: Decimal,
    pub total_tokens: u64,
    pub event_count: u64,
    pub by_model: BTreeMap<ModelId, ModelUsage>,
    pub by_project: BTreeMap<PathBuf, ProjectUsage>,
}

/// Immutable, cheaply cloneable snapshot handed to consumers (CLI/API/Tray).
/// Implemented behind an `Arc` so readers see a consistent view while the scanner
/// keeps producing new snapshots on write.
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub taken_at: DateTime<Utc>,
    pub today: Summary,
    pub this_week: Summary,
    pub this_month: Summary,
    pub active_session: Option<SessionState>,
    pub sessions: Vec<SessionState>,
    pub pricing_warnings: Vec<ModelId>, // Modelle ohne bekannte Preise
}
```

**Bewusste Designentscheidungen:**

- `ModelId` bleibt ein String, **nicht** ein Enum — unbekannte Modelle müssen Tokens
  erfassen können, auch wenn Pricing fehlt (siehe ADR-004).
- Mengen in `u64`, Kosten in `rust_decimal::Decimal` — Floating-Point ist bei
  µ-Dollar-Beträgen eine Falle; Decimal ist exakt und serialisierbar.
- `BTreeMap` statt `HashMap` für deterministische Iteration (wichtig für stabile
  API-Responses und Tests).
- Snapshots sind komplett immutable; Scanner baut den nächsten Snapshot aus dem alten
  + Δ-Events (Copy-on-write, nicht in-place).

## 4. Scanner-Design mit Position-Tracking (ADR-011)

```rust
// src/scanner.rs

pub struct ScannerConfig {
    pub projects_root: PathBuf,    // Default: %USERPROFILE%\.claude\projects
    pub active_window: Duration,   // Default: 5 min
    pub ignore_subdirs: Vec<&'static str>, // ["memory"] (kein Session-Log)
}

pub struct Scanner {
    config: ScannerConfig,
    pricing: Arc<PricingTable>,
    positions: DashMap<PathBuf, FilePosition>,
    sessions: DashMap<PathBuf, SessionState>, // keyed by file path
    snapshot: Arc<ArcSwap<Snapshot>>,         // lock-free publication
}

#[derive(Clone, Debug)]
pub struct FilePosition {
    pub byte_offset: u64,
    pub file_id: FileIdentity, // (volume_serial, file_index) on Windows
    pub last_modified: SystemTime,
}
```

**Invariante:** JSONL-Files sind **append-only** (siehe `docs/jsonl-format.md` §2). Ein
Scanner darf daher Δ-Scans rein über `byte_offset` machen. `FileIdentity` fängt den
Edge-Case ab, dass Claude Code eine Datei tatsächlich rotiert oder ersetzt (sehr
unwahrscheinlich, aber billig zu erkennen: wenn `file_id` kippt, Position auf 0 setzen).

### 4.1 Scan-Algorithmus (pro File)

```
1. Stat file; read (volume_serial, file_index, mtime, size).
2. If no position entry or file_id changed: seek(0), drop old session state.
   Else if size < stored offset: seek(0) (truncation detected, warn).
   Else: seek(stored offset).
3. Stream-read to EOF, line-by-line.
4. For each line:
     a. Try parse as JSON. On error: warn, skip.
     b. If type != "assistant": skip cheaply.
     c. If message.model == "<synthetic>" OR isApiErrorMessage == true: skip.
     d. Extract UsageEvent. Feed to aggregator.
5. Update position to new size.
6. Update session last_seen / first_seen.
```

Fehlerpfade (I/O, Permission) werden pro File abgefangen: Logging, weiter zum nächsten.
Nie panic.

### 4.2 File-Watching (notify crate)

- Auf Startup: Full-Scan aller `*.jsonl` unter `projects_root` (rekursiv, `memory/` und
  Tool-Backup-Subdirs ausgenommen).
- Danach: `notify::RecommendedWatcher` auf `projects_root`, Debounce 250 ms.
- `Create(*.jsonl)` → Δ-Scan ab Offset 0.
- `Modify(*.jsonl)` → Δ-Scan ab gespeichertem Offset.
- `Remove(*.jsonl)` → Session-State als `archived` markieren (für MVP: einfach
  entfernen; Restore ist nicht Feature).
- Nach jedem Batch: neuen `Snapshot` bauen, `ArcSwap::store`.

### 4.3 Concurrency

- **Scanner-Thread** (single, owns `Scanner`): treibt Watcher-Events, macht I/O.
- **Consumers** (CLI/API/Tray): lesen `Arc<Snapshot>` via `ArcSwap::load`. Keine Locks.
- **Pricing-Table** hinter `Arc`, wird **nicht** zur Laufzeit aktualisiert (ADR-004) —
  keine Synchronisation nötig.

`DashMap` für Position- und Session-Maps ist eine Vorsichtsmaßnahme, falls wir später
concurrent Scans pro Projekt parallelisieren wollen; im MVP reicht `RwLock<HashMap<…>>`.
Die konkrete Wahl wird im ersten Scanner-PR getroffen; beide erfüllen das Contract.

## 5. Parser (`src/parser.rs`)

- `parse_line(&str) -> Result<Option<UsageEvent>, ParseError>` — `Ok(None)` für
  Zeilen, die kein billing-relevantes `assistant` sind; `Ok(Some(…))` für Events;
  `Err(ParseError)` für echte JSON-Struktur-Fehler (Zeile ist übersprung-bar).
- Implementiert mit `serde_json::from_str` und expliziten Struct-Typen (`AssistantLine`,
  `AssistantMessage`, `Usage`). Felder, die nicht benötigt werden, werden ausgelassen —
  `serde` ignoriert unbekannte Felder, was unsere additive Toleranz (siehe
  `jsonl-format.md` Scope-Warnung) direkt erfüllt.

## 6. Pricing (`src/pricing.rs`)

- Embedded-Default über `include_str!("pricing.json")` (ADR-004).
- Struktur siehe `docs/pricing.md`.
- Lookup-Strategie: exakte Model-ID → bei Miss Datum-Suffix strippen (`-YYYYMMDD$`) →
  bei Miss Warning an `Snapshot.pricing_warnings` und Event wird ohne Kosten gezählt.
- `ModelPricing` enthält `input`, `output`, `cache_read`, `cache_write_5m`,
  `cache_write_1h` in USD pro MTok (6-stellige Decimal-Präzision).

## 7. Aggregation (`src/aggregate.rs`)

- Rolling-Window-Aggregation: `today` = `[00:00 lokale Zeit, jetzt]`, `this_week` =
  seit Montag 00:00, `this_month` = seit 1. des Monats 00:00.
- Alle Zeit-Fenster werden bei jedem Snapshot-Build aus `sessions` neu berechnet —
  kein Incremental-Update nötig bei der Größenordnung (Aggregation über wenige tausend
  Events ist sub-Millisekunde).
- Active-Session = Session mit `last_seen > now - active_window`. Es kann mehrere
  geben; wir exponieren im `Snapshot.active_session` diejenige mit dem neuesten
  `last_seen`. Der vollständige Liste-Fall wird über `Snapshot.sessions` bedient.

## 8. Position-Tracking und Parallel-Sessions (ADR-011, Begründung)

Szenario: Zwei Claude-Code-Sessions laufen parallel in unterschiedlichen Projekten.
Beide JSONL-Files wachsen zwischen zwei Scanner-Ticks. Naiv (Full-Scan + "nur letztes
mtime berücksichtigen") würde der zweite Tick Token-Events eines Files doppelt zählen,
wenn er nach einem Full-Scan erneut läuft.

Mit Position-Tracking:

- Jeder File-Scan liest exakt die neuen Bytes seit letztem Tick.
- Neue Events werden exakt einmal aggregiert.
- File-ID-Check fängt ab, falls Claude Code ausnahmsweise eine Datei rotiert.

Kosten: ~32 Byte pro offener Session im Memory. Bei 50 Sessions ≈ 1.6 KB — vernachlässigbar.

## 9. Threading-/Startup-Sequenz

```
main (bin/tray oder cli oder api):
    config = load_config()
    pricing = PricingTable::embedded_default()
    scanner = Scanner::new(config.scanner, pricing)
    scanner.full_scan()                  // populate initial snapshot
    scanner.start_watching()             // spawns its own thread
    consumers.use(scanner.snapshot())    // Arc<ArcSwap<Snapshot>>
```

Keine Tokio/async-Runtime im Scanner: `notify` liefert einen sync-Channel, I/O ist
sequenziell schnell genug. Die API-Crate wird Tokio nutzen (Axum), greift aber nur
lesend auf den Snapshot zu — keine Kreuz-Runtime-Probleme.

## 10. Fehler-Strategie

- `thiserror`-Enums pro Modul: `ScannerError`, `ParseError`, `PricingError`.
- `anyhow` nur in den Binary-Entrypoints (examples/scan.rs, später CLI/API/Tray-Hosts).
- **Niemals `unwrap()`** in `src/` außerhalb von `#[cfg(test)]`.
- Scanner-Fehler eines Files blockieren nie den Rest: Log + skip.

## 11. Offene Architektur-Entscheidungen

- **Persistente Position-Map auf Disk** (für schnellen Startup bei vielen Sessions) —
  erst betrachten, wenn Startup-Zeit messbar ein Problem wird.
- **Async-Scanner** (Tokio) — derzeit kein Bedarf; Re-Visit falls `notify`-Events in
  Bursts verloren gehen.
- **Workspace-Split** — siehe ADR-001; Trigger ist "erste zweite Konsumenten-Schicht
  entsteht konkret".
