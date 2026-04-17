# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Phase 1 starten: `winusage-core::parser` + `model` implementieren.**

Gate: Nutzer muss die Phase-0-Doku (`CLAUDE.md`, `DECISIONS.md`, `docs/*.md`) einmal
abnehmen. Falls das noch offen ist, erst dort klären, dann fortfahren.

Konkrete Erstaufgabe (genau in dieser Reihenfolge, jeweils kleiner, eigener Commit):

1. **Dependencies nach `Cargo.toml` aufnehmen** (Phase-1-Minimum):
   - `serde = { version = "1", features = ["derive"] }`
   - `serde_json = "1"`
   - `chrono = { version = "0.4", features = ["serde"] }`
   - `rust_decimal = { version = "1", features = ["serde"] }`
   - `rust_decimal_macros = "1"`
   - `thiserror = "1"`
   - `dev-dependencies`: `pretty_assertions = "1"`
2. **`src/model.rs`** mit den Typen aus `docs/architecture.md` §3. Keine Logik — nur
   Typen, `Default`-Impls, kleine Konstruktoren. Public reexport in `src/lib.rs`.
3. **`src/parser.rs`** — `parse_line` deserialisiert nur die notwendigen Felder einer
   `assistant`-Zeile in `UsageEvent`. Nicht-assistant-Zeilen → `Ok(None)`. Synthetic
   oder `isApiErrorMessage` → `Ok(None)`. Fehlerhafte JSON → `Err(ParseError)` (Scanner
   entscheidet über Skip).
4. **Unit-Tests** für `parser.rs` mit `fixtures/happy-path.jsonl`, `error-synthetic.jsonl`,
   `sidechain.jsonl`. Asserts: Token-Zahlen, Modell-IDs, `is_sidechain`-Flag.
5. **CI-Minimal-Workflow** `.github/workflows/ci.yml` (laut ADR-007) — Windows-Runner,
   drei Steps: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test`.

Commit-Rhythmus: ein Commit pro Punkt 1–5. `PROGRESS.md` im selben Commit aktualisieren.
Push nach jedem Commit.

## Danach

- `src/pricing.rs` + `src/pricing.json` (Platzhalter-Werte, bis reale Preisliste beim
  Release gepflegt wird).
- `src/aggregate.rs` + Integration-Test mit vollständiger Mini-Session.
- `src/scanner.rs` mit Position-Tracking + `notify`-Watcher.
- `examples/scan.rs` als Dev-CLI (liest `%USERPROFILE%\.claude\projects\`, gibt Snapshot
  als JSON auf stdout aus).
- Erst dann: `winusage-cli`-Subcommands beginnen (voraussichtlich Workspace-Split-
  Trigger laut ADR-001).
