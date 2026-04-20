# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Phase 3, Schritt 1: Export — CSV und JSON.**

### Kontext

`winusage export` erweitert die CLI um maschinenlesbare Ausgabe. Nützlich für
Eigenanalysen, Skripte und spätere Dashboard-Integrationen.

### Schritte

1. **`src/bin/winusage.rs`** — `Export`-Subcommand mit `--format <csv|json>` und
   `--period <today|week|month>` (Default: `month`) hinzufügen.
   - JSON: Snapshot-Felder `total_cost_usd`, `total_tokens`, `event_count`,
     `by_model` (Array), `by_project` (Array).
   - CSV: Header-Zeile + eine Zeile pro Modell:
     `period,model,input_tokens,output_tokens,cost_usd`

2. Tests in `src/bin/winusage.rs` sind nicht nötig — die Formatierungslogik bleibt
   thin. Ggf. Snapshot-Daten-Test in `aggregate.rs`.

### Danach (Phase 3 Reihenfolge)

- Provider-Plugin-Trait (ADR-012 schreiben, dann `src/provider.rs`)
- Heatmap im Tray (7×n-CSS-Grid, Tages-Granularität)
- Auto-Update via Tauri Updater
