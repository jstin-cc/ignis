# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**`src/scanner.rs` implementieren.**

Full-Scan + Position-Tracking + optionaler `notify`-Watcher.

1. **Datentypen** (`src/scanner.rs`):
   - `FileIdentity { inode: u64, size: u64 }` — zur Erkennung von Rotationen/Neuanlage.
   - `FilePosition { path: PathBuf, identity: FileIdentity, byte_offset: u64 }` — pro JSONL-File.
   - `ScanResult { events: Vec<UsageEvent>, positions: Vec<FilePosition>, errors: Vec<ScanError> }`.

2. **`scan_all(root: &Path) -> ScanResult`**:
   - Findet alle `%USERPROFILE%\.claude\projects\**\*.jsonl` (via `walkdir` oder `glob`).
   - Liest jede Datei ab Offset 0, parst Zeilen via `parser::parse_line()`.
   - Erfasst Byte-Offsets für jede Datei.
   - Korrupte Zeilen: `ScanError` sammeln, nicht abbrechen.

3. **`scan_delta(positions: &[FilePosition]) -> ScanResult`**:
   - Liest jede Datei nur ab gespeichertem `byte_offset`.
   - Prüft `FileIdentity` — bei Rotation (neue Inode) → Full-Scan dieser Datei.

4. **Dependency**: `walkdir = "2"` in `Cargo.toml` eintragen.
   `notify`-Watcher: **deferred** (kommt mit `scanner::watch()`).

5. **Tests**:
   - Full-Scan gegen `fixtures/happy-path.jsonl` → 2 Events erwartet.
   - Delta-Scan (Offset nach erstem Event) → 1 Event erwartet.
   - Rotation-Erkennung (identity geändert) → Full-Scan-Fallback.

Danach: `src/config.rs` + `examples/scan.rs`.
