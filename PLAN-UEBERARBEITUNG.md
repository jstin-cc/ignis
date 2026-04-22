# Plan: Ignis Tray-UI Überarbeitung (v1.1.0)

Erstellt: 2026-04-22 | Ausführung: geplant 2026-04-23

---

## Kontext & Ziel

Die Tray-UI entspricht noch nicht vollständig den Vorgaben aus `DESIGN.md`. Konkrete Lücken:

- **Kein TabBar** — alle Panels scrollen vertikal, statt tabweise zu wechseln
- **Progress-Bar-Farben** via Inline-Styles statt `progressClass()`-CSS-Klassen; `danger`-Zustand (≥100%) fehlt komplett
- **Buttons** mit Inline-Styles statt `.btn--primary` / `.btn--ghost`
- **Token-Ablauf** zeigt keinen spezifischen Hinweis, wenn Anthropic-Auth fehlschlägt
- **Port-Konflikt** beim API-Child-Spawn nicht geprüft (kann doppelten Spawn verursachen)
- **v1.1.0** noch nicht getaggt; CHANGELOG/README veraltet

**Strategie:** In-Place-Überarbeitung in `tray/src/` (kein Umzug nach `apps/tray-ui/`).
Die Datei `apps/tray-ui/src/styles/tokens.css` dient als vollständige Token-Referenz.

---

## Kritische Dateien

| Datei | Was sich ändert |
|---|---|
| `tray/src/index.css` | Token-Set mit `apps/tray-ui/src/styles/tokens.css` abgleichen |
| `tray/index.html` | Google-Fonts-Link (IBM Plex Sans + Mono) einfügen |
| `tray/src/App.tsx` | TabBar-Layout, Settings-Overlay, kein Scroll |
| `tray/src/components/format.ts` | `fmt`-Objekt (usd/tok/dur) nach DESIGN.md-Spec |
| `tray/src/components/BlockPanel.tsx` | `progressClass`, Token-Ablauf-UX, CSS-Klassen |
| `tray/src/components/Footer.tsx` | `.btn--primary` + `.btn--ghost` |
| `tray/src/components/TodayPanel.tsx` | → `TodaySection` (section-label, fmt) |
| `tray/src/components/MonthPanel.tsx` | WeekSection-Variante mit ProgressBar |
| `tray/src/components/ActiveSessionPanel.tsx` | → `SessionSection` |
| `tray/src/components/ProjectsPanel.tsx` | Tab-Layout (380px, kein Panel-Background) |
| `tray/src/components/HeatmapPanel.tsx` | Tab-Layout, englische Labels |
| `tray/src-tauri/src/main.rs` | Port-Konflikt-Check vor `spawn_api()` |
| `CHANGELOG.md` | `[Unreleased]` → `[1.1.0]` |
| `README.md` | Auto-Spawn, Usage-Balken, Plan-Settings |
| `NEXT.md` | Items abhaken, v1.2.0-Kandidaten |
| `PROGRESS.md` | Nach jedem Schritt aktualisieren |

---

## Schritte (sequenziell, je mit PROGRESS.md-Update + Commit)

### Schritt 0 — Design-Token-Basis

**Dateien:** `tray/src/index.css`, `tray/index.html`

Fehlende Tokens aus `apps/tray-ui/src/styles/tokens.css` ergänzen:
- `--tray-width: 360px`, `--tray-header-height: 48px`
- `--space-1` bis `--space-8` (4 / 8 / 12 / 16 / 20 / 24 / 28 / 32 px)
- `--radius-chip: 2px`, `--radius-button: 4px`, `--radius-card: 6px`, `--radius-panel: 8px`
- `--shadow-overlay`, `--shadow-panel`
- `.progress-track`, `.progress-fill`, `.progress-fill--high`, `.progress-fill--warning`, `.progress-fill--danger`
- `.btn`, `.btn--primary`, `.btn--secondary`, `.btn--ghost`
- `.section-label`, `.extra-usage`, `.badge`

In `tray/index.html` einfügen (im `<head>`):
```html
<link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&family=IBM+Plex+Sans:wght@300;400;500;600&display=swap" rel="stylesheet">
```

**PROGRESS.md:** "Schritt 0 — Design-Tokens vollständig, IBM Plex Fonts geladen"

---

### Schritt 1 — `format.ts` nach DESIGN.md

**Datei:** `tray/src/components/format.ts`

`fmt`-Objekt exportieren (exakt diese Implementierung laut DESIGN.md):
```ts
export const fmt = {
  usd: (n: number) => '$' + n.toFixed(2),
  tok: (n: number) => {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
    if (n >= 1_000)     return (n / 1_000).toFixed(0) + 'k';
    return n.toString();
  },
  dur: (s: number) => {
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    return h > 0 ? `${h}h ${m}m` : `${m}m ${s % 60}s`;
  },
};
```

`formatTokens` / `formatDuration` als bestehende Wrapper **erhalten** (Rückwärtskompatibilität mit anderen Komponenten).

**PROGRESS.md:** "Schritt 1 — format.ts: fmt-Objekt nach DESIGN.md-Spec exportiert"

---

### Schritt 2 — `TabBar.tsx` (neue Datei)

**Datei:** `tray/src/components/TabBar.tsx` (neu erstellen)

```ts
export type TabId = 'today' | 'month' | 'projects' | 'heatmap';
interface TabBarProps {
  active: TabId;
  onChange: (tab: TabId) => void;
}
```

Styling:
- Höhe: 36px, Breite: 360px
- 4 gleichbreite Buttons (je 90px) als `.btn--ghost`-Basis
- Aktiver Tab: `color: var(--text-primary)`, `border-bottom: 2px solid var(--accent)`
- Inaktiver Tab: `color: var(--text-muted)`, hover: `color: var(--text-secondary)`
- Font: `var(--font-sans)`, 12px, weight 500, uppercase
- `border-bottom: 1px solid var(--border-subtle)` auf der gesamten TabBar

**PROGRESS.md:** "Schritt 2 — TabBar.tsx erstellt (today/month/projects/heatmap)"

---

### Schritt 3 — `App.tsx` Shell-Umbau

**Datei:** `tray/src/App.tsx`

Neues Layout (exakt 520px Gesamt-Höhe):
```
Header         48px  (drag-region)
TabBar         36px
content area  380px  (overflow: hidden, kein Scroll)
Footer         56px
```

Tab-Switching-Logik:
```
active === 'today'    → <TodaySection> + <WeekSection> + <SessionSection> + <BlockSection>
active === 'month'    → <MonthPanel variant="full">
active === 'projects' → <ProjectsPanel>
active === 'heatmap'  → <HeatmapPanel>
```

Settings-Panel: als **Overlay** (absolute, top: 0, left: 0, width: 100%, height: 380px, `background: var(--bg-overlay)`, z-index: 10) über dem content-Bereich. ⚙-Button im Header togglet das Overlay; × schließt es.

State in App.tsx:
```ts
const [activeTab, setActiveTab] = useState<TabId>('today');
const [settingsOpen, setSettingsOpen] = useState(false);
```

**PROGRESS.md:** "Schritt 3 — App.tsx: TabBar-Layout, Settings-Overlay, kein Scroll"

---

### Schritt 4 — TodaySection (aus TodayPanel)

**Datei:** `tray/src/components/TodayPanel.tsx` (überarbeiten, Komponente als `TodaySection` exportieren)

Änderungen:
- Abschnittsbezeichnung via `<div className="section-label">TODAY</div>`
- Hero: `fmt.usd(parseFloat(data.total_cost_usd))`
- Meta: `` `${fmt.tok(data.total_tokens)} tok · ${data.event_count} events` ``
- Kein `backgroundColor: "var(--bg-elevated)"` auf dem Wrapper

**PROGRESS.md:** "Schritt 4 — TodaySection überarbeitet (section-label, fmt)"

---

### Schritt 5 — WeekSection-Variante (aus MonthPanel)

**Datei:** `tray/src/components/MonthPanel.tsx` (überarbeiten)

`variant`-Prop hinzufügen: `'week' | 'full'` (default: `'full'`)

`week`-Variante (auf Today-Tab):
- Label: "THIS MONTH" (kein echter Wochen-Endpoint vorhanden — Monatsdaten als Proxy)
- Hero: `fmt.usd(parseFloat(data.total_cost_usd))`
- ProgressBar: Monats-Fortschritt in Tagen = `(heute.getDate() / tageImMonat) * 100`
- Farbe via `progressClass(pct)` → CSS-Klasse auf `.progress-fill`
- Meta: Token-Summe + Event-Count

`full`-Variante: bisheriges Verhalten, wird auf Month-Tab verwendet.

```ts
function progressClass(pct: number): string {
  if (pct >= 100) return 'progress-fill--danger';
  if (pct >= 90)  return 'progress-fill--warning';
  if (pct >= 75)  return 'progress-fill--high';
  return '';
}
```

**PROGRESS.md:** "Schritt 5 — MonthPanel: WeekSection-Variante + progressClass implementiert"

---

### Schritt 6 — BlockSection + Token-Ablauf-UX (aus BlockPanel)

**Datei:** `tray/src/components/BlockPanel.tsx` (überarbeiten)

Vier Änderungen:

1. **progressClass statt barColor-Inline-Logik:**
   ```tsx
   // Vorher: style={{ backgroundColor: barColor }}
   // Nachher:
   <div className={`progress-fill ${progressClass(pct)}`} style={{ width: `${pct}%`, transition: 'width 200ms ease-out' }} />
   ```

2. **Transition:** nur `width 200ms ease-out` — `background-color`-Übergang entfernen.

3. **Extra-Usage-Row:** `.extra-usage`-CSS-Klasse statt Inline-Styles:
   ```tsx
   {block.extraUsage > 0 && (
     <div className="extra-usage">
       <span>Extra Usage</span>
       <span>+{fmt.usd(block.extraUsage)}</span>
     </div>
   )}
   ```

4. **Token-Ablauf-UX** im Error-Banner:
   ```ts
   function errorMessage(err: string): string {
     if (/do_refresh|401|token|auth/i.test(err))
       return 'Anthropic session expired. Re-run `claude` to reconnect.';
     if (/network|fetch|offline/i.test(err))
       return 'Offline — cached values shown.';
     return err;
   }
   ```

**PROGRESS.md:** "Schritt 6 — BlockPanel: progressClass, Token-Ablauf-UX, CSS-Klassen"

---

### Schritt 7 — SessionSection (aus ActiveSessionPanel)

**Datei:** `tray/src/components/ActiveSessionPanel.tsx` (überarbeiten)

Änderungen:
- Label: `<div className="section-label">ACTIVE SESSION</div>`
- Token-Anzeige: `fmt.tok(total_tokens)` (ohne Einheit im Wert, Einheit aus Label-Kontext)
- Duration bleibt: `formatDuration(first_seen, last_seen)` (passt bereits zu DESIGN.md-Format)
- Kein Panel-Background

**PROGRESS.md:** "Schritt 7 — SessionSection überarbeitet"

---

### Schritt 8 — ProjectsPanel + HeatmapPanel für Tab-Layout

**Dateien:** `tray/src/components/ProjectsPanel.tsx`, `tray/src/components/HeatmapPanel.tsx`

ProjectsPanel:
- Panel-eigenen `backgroundColor: "var(--bg-elevated)"` entfernen
- Volle Tab-Höhe nutzen (380px), CSS-Klassen für Labels

HeatmapPanel:
- `backgroundColor: "var(--bg-elevated)"` entfernen
- Labels auf Englisch: `DAY_LABELS: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']`
- Grid nutzt gesamte 380px Höhe

**PROGRESS.md:** "Schritt 8 — Projects- und HeatmapPanel auf Tab-Layout angepasst"

---

### Schritt 9 — Footer CSS-Klassen

**Datei:** `tray/src/components/Footer.tsx`

```tsx
// Vorher: button mit Inline-Styles
// Nachher:
<button className="btn btn--primary" onClick={handleDashboard}>Open Dashboard</button>
<button className="btn btn--ghost" onClick={handleCli}>CLI: ignis</button>
```

**PROGRESS.md:** "Schritt 9 — Footer: .btn--primary + .btn--ghost CSS-Klassen"

---

### Schritt 10 — Port-Konflikt-Handling (Rust)

**Datei:** `tray/src-tauri/src/main.rs`

Vor dem `spawn_api()`-Call prüfen ob Port 7337 frei ist:
```rust
fn port_is_free(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}
// Im spawn_api()-Aufruf:
if !port_is_free(7337) {
    // Port bereits belegt — vorhandene Instanz nutzen, kein neuer Spawn
    return;
}
```

**PROGRESS.md:** "Schritt 10 — Port 7337 Konflikt-Check vor spawn_api()"

---

### Schritt 11 — CHANGELOG + README + NEXT.md

**Dateien:** `CHANGELOG.md`, `README.md`, `NEXT.md`

CHANGELOG.md:
- `[Unreleased]` → `[1.1.0] — 2026-04-23`
- Added: TabBar-Navigation, progressClass-Farblogik, fmt-Objekt, Token-Ablauf-UX, Port-Konflikt-Handling, CSS-Button-Klassen
- Neuer leerer `[Unreleased]`-Block oben

README.md (Abschnitte aktualisieren):
- Auto-Spawn: Tray spawnt `ignis-api` automatisch beim Start
- Usage-Balken: 3 Balken (5h-Block, Woche, Extra) via Anthropic OAuth
- Plan-Settings: Pro / Max5 / Max20 / Custom-Limit im ⚙-Menü

NEXT.md:
- Alle aktuellen Items als `[x]` markieren
- Neue mögliche v1.2.0-Kandidaten: echter `/v1/summary?range=week`-Endpoint, Settings als eigener Tab, Wochen-Heatmap-Ansicht

**PROGRESS.md:** "Schritt 11 — CHANGELOG v1.1.0, README + NEXT.md aktualisiert"

---

### Schritt 12 — v1.1.0 taggen

```bash
git tag v1.1.0
git push origin v1.1.0
```

**PROGRESS.md:** "v1.1.0 getaggt — Phase v1.1.0 abgeschlossen"

---

## Abhängigkeitsgraph

```
Schritt 0 (tokens.css)
  ↓
Schritt 1 (format.ts)
  ↓
Schritt 2 (TabBar.tsx)
  ↓
Schritt 4, 5, 6, 7 ──────┐
Schritt 8, 9             ↓
Schritt 3 (App.tsx) ←────┘ (importiert alle Komponenten)
  │
  ├── Schritt 10 (Rust, parallel möglich)
  ↓
Schritt 11 (Docs)
  ↓
Schritt 12 (Tag)
```

---

## Verifikationsliste (Acceptance Criteria)

### Layout
- [ ] Tray-Panel 360×520px, kein Scroll auf Root-Ebene
- [ ] Header 48px mit `data-tauri-drag-region`
- [ ] TabBar 36px mit 4 Tabs (Today / Month / Projects / Heatmap)
- [ ] Tab-Wechsel zeigt korrekte Inhalte
- [ ] Footer immer sichtbar (nicht scroll-weg)
- [ ] Settings-Overlay öffnet/schließt via ⚙ / ×

### Design-Token-Konformität
- [ ] Alle Farben via `var(--token)` — kein Hex in Komponenten hardcoded
- [ ] IBM Plex Sans für UI-Text, IBM Plex Mono für Zahlen
- [ ] Spacing nur: 4 / 8 / 12 / 16 / 24 / 32 px

### Progress-Bar-Farblogik
- [ ] pct ≥ 100 → `.progress-fill--danger` (rot)
- [ ] pct ≥ 90  → `.progress-fill--warning` (gelb)
- [ ] pct ≥ 75  → `.progress-fill--high` (heller Akzent)
- [ ] pct < 75  → kein Modifier (accent-muted default)
- [ ] Nur `transition: width 200ms ease-out`, kein bg-color-Übergang

### Zahlenformatierung
- [ ] `fmt.usd(1.234)` → `'$1.23'`
- [ ] `fmt.tok(1_500_000)` → `'1.5M'`
- [ ] `fmt.tok(25_000)` → `'25k'`
- [ ] `fmt.tok(999)` → `'999'`
- [ ] `fmt.dur(5401)` → `'1h 30m'`
- [ ] `fmt.dur(95)` → `'1m 35s'`

### Button-Klassen
- [ ] "Open Dashboard" hat Klasse `.btn.btn--primary`
- [ ] "CLI: ignis" hat Klasse `.btn.btn--ghost`

### Token-Ablauf-UX
- [ ] Auth-Fehler → "Anthropic session expired. Re-run `claude` to reconnect."

### Port-Konflikt
- [ ] Port 7337 bereits belegt → kein Crash, kein doppelter Spawn

### Nicht-Ziele (dürfen nicht eingeführt werden)
- [ ] Kein Light-Mode-Code
- [ ] Keine Icon-Library (nur Unicode ⚙ ×)
- [ ] Keine weiteren Animationen außer progress-bar `width 200ms ease-out`
- [ ] Kein Scroll im Tray-Panel

### Build
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] `npm run build` in `tray/` ohne Fehler
- [ ] `cargo test` — alle Tests grün
