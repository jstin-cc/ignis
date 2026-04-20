# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Phase 3, Schritt 3: Heatmap im Tray.**

### Kontext

Eine Aktivitäts-Heatmap zeigt die täglichen Kosten der letzten 12 Wochen als
7×12-CSS-Grid (Spalten = Wochen, Zeilen = Wochentage Mo–So). Farbtiefe entspricht
der Tagesausgabe (leer → leichtes Terrakotta → volles Terrakotta).

### Schritte

1. **`src/api.rs`** — `GET /v1/heatmap` Endpoint: gibt Array von
   `{ date: "YYYY-MM-DD", cost_usd: "0.00" }` für die letzten 84 Tage zurück.
   Neue Funktion `daily_costs(events, pricing, since)` in `aggregate.rs`.

2. **`tray/src/types.ts`** — `HeatmapDay`-Interface + `fetchHeatmap()`-Funktion
   in `useUsageData.ts`.

3. **`tray/src/components/HeatmapPanel.tsx`** — 7×12 CSS-Grid (kein Recharts),
   Tooltip-Titel mit Datum + Kosten (`title`-Attribut reicht).

4. **`tray/src/App.tsx`** — `HeatmapPanel` zwischen ProjectsPanel und Footer einbauen.

### Danach

- Auto-Update via Tauri Updater (Phase 3 Abschluss)
- v1.0.0 Tag vorbereiten
