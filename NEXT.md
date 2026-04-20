# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Phase 2: Tray-App — Per-Projekt-Breakdown.**

### Kontext

`Summary.by_project` enthält bereits Kosten und Token pro Projekt-Pfad. Die Tray-App
zeigt das noch nicht. Eine neue Sektion soll die Top-Projekte nach Kosten anzeigen.

### Schritte

1. **`tray/src/components/ProjectsPanel.tsx`** — neue Komponente:
   - Label: "PROJECTS (TODAY)"
   - Liste: Projektname (aus `projectName()`), Kosten rechts ausgerichtet
   - Maximal 5 Einträge, absteigend nach `total_cost_usd` sortiert
   - Kein Chart für MVP — pure Textliste

2. **`tray/src/App.tsx`** — `ProjectsPanel` zwischen `BlockPanel` und
   `ActiveSessionPanel` einbinden, mit Trennlinie.

3. Nur rendern wenn `today.by_project.length > 0`.

Danach: Notification-Schwellen (Limit-Warning bei 80% / 100% des Blocks).
