# Ignis — Design Handoff

> Diese Datei gehört in `CLAUDE.md` oder als `@file`-Referenz in Cursor-Sessions,
> die an `apps/tray-ui/` arbeiten.

---

## Design-System-Dateien

| Datei | Zweck | Ziel im Repo |
|---|---|---|
| `handoff/tokens.css` | Alle CSS Custom Properties + Utility-Klassen | `apps/tray-ui/src/styles/tokens.css` |
| `assets/ignis-icon.svg` | Quell-SVG für App-Icon | `apps/tray-ui/src/assets/` |
| `icons/32x32.png` … `icons/512x512.png` | Tauri-Bundle-Icons | `tray/src-tauri/icons/` |

**Import einmalig im App-Root:**
```ts
// apps/tray-ui/src/main.tsx
import './styles/tokens.css';
```

---

## Farbpalette

Alle Farben sind CSS Custom Properties. **Niemals Hex-Werte hardcoden** — immer `var(--token)`.

```css
/* Backgrounds */
--bg-base:        #1F1E1B   /* Haupt-Background */
--bg-elevated:    #292724   /* Karten, Panels */
--bg-overlay:     #34312C   /* Modals, Tooltips */

/* Text */
--text-primary:   #F4F3EE
--text-secondary: #B1ADA1
--text-muted:     #7A766D

/* Akzent */
--accent:         #C15F3C   /* DIE eine Signalfarbe — sparsam! */
--accent-hover:   #D47551
--accent-muted:   #8B4428

/* Status */
--success:        #7A9B76
--warning:        #D4A574
--danger:         #C06862
```

---

## Typografie

```tsx
// Fonts im HTML-Head laden:
// <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&family=IBM+Plex+Sans:wght@300;400;500;600&display=swap" rel="stylesheet">

// In CSS:
font-family: var(--font-sans);   // IBM Plex Sans — UI-Text
font-family: var(--font-mono);   // IBM Plex Mono — Zahlen, Code

// Tabular nums aktivieren für alle Live-Werte:
font-variant-numeric: tabular-nums;
// oder Utility-Klasse: className="tabular"
```

**Größen-Hierarchie:** Hero 24px · Body 14px · Label 12px · Section 11px — keine anderen Größen.

---

## Spacing

Nur diese Werte verwenden: `4 / 8 / 12 / 16 / 24 / 32 px`  
Als Tokens: `var(--space-1)` bis `var(--space-8)`.

---

## Tray-Panel — Struktur

```tsx
// Breite: 360px fix (var(--tray-width))
// Header: 48px (var(--tray-header-height))
// Sections: 14px/16px padding, border-bottom: 1px solid var(--border-subtle)

<div style={{ width: 'var(--tray-width)' }}>
  <Header />           {/* 48px, drag-region, cursor: move */}
  <TabBar />           {/* Today / Month / Projects / Heatmap */}
  <TodaySection />     {/* Hero-Zahl + meta */}
  <WeekSection />      {/* Hero + ProgressBar + meta */}
  <SessionSection />   {/* Name + duration, wenn aktive Session */}
  <BlockSection />     {/* ProgressBar + optional ExtraUsage */}
  <Footer />           {/* Open Dashboard + CLI-Button */}
</div>
```

---

## Progress Bar — Farblogik

```ts
// In JS/TS berechnen, welche CSS-Klasse:
function progressClass(pct: number): string {
  if (pct >= 100) return 'progress-fill--danger';
  if (pct >= 90)  return 'progress-fill--warning';
  if (pct >= 75)  return 'progress-fill--high';
  return '';  // default: --accent-muted
}
```

---

## Buttons

```tsx
// Primär (Open Dashboard)
<button className="btn btn--primary">Open Dashboard</button>

// Sekundär (Settings, CLI)
<button className="btn btn--secondary">Settings</button>

// Ghost (CLI-Link)
<button className="btn btn--ghost">CLI: ignis</button>
```

---

## Zahlen formatieren

```ts
// Exakt so in der UI verwenden:
const fmt = {
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

---

## Extra Usage

Wenn `extraUsage > 0` im Block-Objekt, diesen Row einblenden:

```tsx
{block.extraUsage > 0 && (
  <div className="extra-usage">
    <span>Extra Usage</span>
    <span>+{fmt.usd(block.extraUsage)}</span>
  </div>
)}
```

---

---

## Dashboard

Das Dashboard öffnet sich als Overlay über dem Tray-Panel (gleiche 360px Breite) wenn der User "Open Dashboard" klickt. Header mit ← Tray-Back-Button, zwei Tabs: **Live** und **History**.

### Struktur

```tsx
<Dashboard onClose={() => setShowDashboard(false)}>
  <DashboardHeader />         {/* 48px, --bg-elevated, ← Tray Button */}
  <DashboardTabBar />         {/* Live / History */}
  <ScrollBody maxHeight={540}>{/* overflowY: auto */}
    {tab === 'live'    && <LiveTab />}
    {tab === 'history' && <HistoryTab />}
  </ScrollBody>
</Dashboard>
```

### Live-Tab — Sektionen

1. **Burn Rate** — Sparkline-Balkendiagramm (30 Datenpunkte = letzte 30 Min), avg tok/min rechts
2. **Active Session** — Name + Laufzeit links, Kosten-Hero + Tokens rechts. Darunter 4px Token-Typ-Bar (Input/Output/Cache-r/Cache-w) + Legende
3. **Session Block** — Ring-Progress (SVG, 72px) + Reset-Zeit + Extra-Usage-Warning. Darunter 5-Segment-Timeline (0h–5h)
4. **By Model** — Pro Modell: Name (--accent, mono) + Kosten + Tokens + relativer Balken

### History-Tab — Sektionen

1. **This Week vs Last Week** — Doppelbalken pro Wochentag (this=--accent, last=--border-default opacity 0.6)
2. **Cost Trend 30 Days** — SVG Polyline + Flächenfüllung (Gradient --accent → transparent), letzter Punkt als Dot
3. **Top Projects 30 Days** — Rangfolge, relativer Balken, Kosten + Token-Anzahl

### Chart-Komponenten (pure SVG, keine externe Library)

```tsx
// Sparkline — Balken
const Sparkline = ({ values, width = 328, height = 48, color = 'var(--accent)' }) => { ... }
// Letzter Balken: volle Farbe. Übrige: rgba(193,95,60, 0.25 + (v/max)*0.55)

// Linechart — Polyline + Area
const LineChart = ({ values, width = 328, height = 80 }) => { ... }
// Area: linearGradient --accent 22% → 0%. Linie: stroke #C15F3C, strokeWidth 1.5

// BlockRing — SVG Kreis-Progress
const BlockRing = ({ pct, size = 72 }) => { ... }
// r = size/2 - 6, strokeWidth 5
// Farb-Logik: >= 100 → --danger, >= 90 → --warning, >= 75 → --accent, sonst --accent-muted
```

### Token-Typ-Farben (Chart-Palette)

```ts
input:       var(--chart-input)       // #C15F3C
output:      var(--chart-output)      // #D4A574
cache-read:  var(--chart-cache-read)  // #7A9B76
cache-write: var(--chart-cache-write) // #8B6B9B
```

### Zahlen im Dashboard

Gleiche `fmt`-Funktionen wie im Tray-Panel (usd / tok / dur). Alle Zahlen: `font-variant-numeric: tabular-nums`.

---

## Nicht-Ziele (MVP v1.0)

- Kein Light-Mode
- Kein Custom-Theming
- Keine Icon-Library (nur Unicode: ⚙ ×)
- Keine Animationen außer `transition: width 200ms ease-out` auf Progress-Bars
- Kein Scroll im Tray-Panel — "Open Dashboard" für mehr Inhalt
