# NEXT

Der **eine** konkrete nächste Schritt. Bei Kontextverlust: erste Datei, die gelesen wird
(nach `CLAUDE.md` und `PROGRESS.md`).

---

## Jetzt

**Phase 2: Tray-App — Per-Projekt-Breakdown + Burn-Rate-Block (Recharts).**

### Kontext

Die Tray-App (`tray/`) zeigt bislang Today / This Month / Active Session (aus Phase 1).
Phase 2 ergänzt:
- Den aktiven Billing-Block als Fortschrittsbalken (analog zum TUI-Panel)
- Einen Per-Projekt-Breakdown (Kosten sortiert nach Projekt)

### Schritte

1. **HTTP-API erweitern** (`src/api.rs`): `/v1/summary`-Response um `active_block`
   erweitern (start, end, cost_usd, token_count, percent_elapsed).

2. **`tray/src/types.ts`** — `ActiveBlock`-Interface hinzufügen.

3. **`tray/src/components/BlockPanel.tsx`** — neues Panel:
   - Fortschrittsbalken (CSS-Gradient, kein Recharts) mit `--accent-muted` bis 75%,
     dann `--accent`, dann `--warning` ab 90%.
   - Kosten + $/h + verbleibende Zeit.

4. **`tray/src/App.tsx`** — BlockPanel oberhalb des Footer-Bereichs einbinden.

Danach: Per-Projekt-Breakdown + Recharts-Chart.
