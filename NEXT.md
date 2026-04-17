# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**CLI-Subcommands implementieren: `winusage daily`, `winusage monthly`, `winusage session`.**

Diese werden als eigenes Binary `src/bin/winusage.rs` mit `clap` angelegt.

### Schritte

1. `clap = { version = "4", features = ["derive"] }` in `[dependencies]` (auch für spätere API und Tray nützlich).

2. `src/bin/winusage.rs` — Clap-CLI:
   ```
   winusage daily     → today-Summary als Tabelle (stdout)
   winusage monthly   → this_month-Summary
   winusage session   → aktive Session, oder "no active session"
   winusage scan      → alias für cargo run --example scan (JSON-Dump, dev-freundlich)
   ```

3. Ausgabe-Format: einfache, lesbare Tabelle (kein Farb-Crate nötig — nur ASCII-Tabellen).
   Beispiel:
   ```
   Model                  Input      Output     Cost
   claude-sonnet-4-6      1.2M tok   120k tok   $3.61
   ──────────────────────────────────────────────────
   Total                                         $3.61
   ```

4. Binary in `Cargo.toml`:
   ```toml
   [[bin]]
   name = "winusage"
   path = "src/bin/winusage.rs"
   ```

5. Tests: Mindestens `cargo run --bin winusage -- daily` muss exit 0 zurückgeben.

Danach: HTTP-API (`src/api.rs`) mit Axum.
