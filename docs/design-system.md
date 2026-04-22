# Design System

**Philosophie.** Claude.ai arbeitet mit einer warmen, menschlichen Ästhetik —
Terrakotta-Orange auf cremigen oder tiefen, warmen Dunkel-Tönen. Ignis übernimmt
dasselbe Gefühl: ein Tool, das sich wie ein vertrauter Arbeitsbegleiter anfühlt, nicht
wie ein Dashboard.

Leitprinzipien:

- **Warm statt kalt.** Keine Pure-Black-Backgrounds, keine klinischen Blautöne.
- **Dicht, aber nicht erdrückend.** Zahlen und Details sichtbar, Luft dazwischen.
- **Ein Akzent, nicht fünf.** Terrakotta (`--accent`) ist DIE eine Signalfarbe.
- **Typografie trägt.** Kräftige Zahlen, zurückhaltende Labels.
- **Keine Emoji-Ikonografie** in der UI — Ignis ist kein Konsumenten-Tool.

Gilt für Tray (React) **und** CLI/TUI (ratatui) gleichermaßen. Der TUI verwendet die
Hex-Werte über ANSI-TrueColor.

---

## 1. Farbpalette (Dark Mode, primär)

```css
/* Backgrounds */
--bg-base:        #1F1E1B;   /* Haupt-Background, warmes Anthrazit */
--bg-elevated:    #292724;   /* Karten, Panels */
--bg-overlay:     #34312C;   /* Modals, Tooltips */

/* Borders & Lines */
--border-subtle:  #3D3A34;
--border-default: #524E46;

/* Text */
--text-primary:   #F4F3EE;   /* Haupt-Text, "Pampas" */
--text-secondary: #B1ADA1;   /* Labels, Metadaten, "Cloudy" */
--text-muted:     #7A766D;   /* Disabled, sekundäre Info */

/* Akzent (Terrakotta) */
--accent:         #C15F3C;   /* "Crail" — DIE Signatur-Farbe */
--accent-hover:   #D47551;
--accent-muted:   #8B4428;   /* Fortschritts-Balken, sekundäre Akzent-Flächen */

/* Status */
--success:        #7A9B76;   /* Gedämpftes Grün, warm */
--warning:        #D4A574;   /* Bernstein */
--danger:         #C06862;   /* Gedämpftes Rot, nicht schreiend */

/* Charts (Token-Typ-Unterscheidung) */
--chart-input:       #C15F3C;  /* Akzent */
--chart-output:      #D4A574;  /* Warm Amber */
--chart-cache-read:  #7A9B76;  /* Sage */
--chart-cache-write: #8B6B9B;  /* Gedämpftes Lila */
```

**Light Mode** kommt in v0.3+. Nicht im MVP.

---

## 2. Typografie

| Rolle            | Font                                      | Größe   | Gewicht |
|------------------|-------------------------------------------|---------|---------|
| UI-Body          | `"Segoe UI Variable", system-ui, sans-serif` | 14 px   | 400     |
| UI-Label         | `"Segoe UI Variable", system-ui, sans-serif` | 12 px   | 500     |
| Hero-Zahl        | `"Segoe UI Variable", system-ui, sans-serif` | 24 px   | 600     |
| Mono (Zahlen-Spalten, CLI) | `"Cascadia Code", "JetBrains Mono", Consolas, monospace` | 13 px | 400 |

**Tabular-Numerals aktivieren** für jede Stelle, an der Zahlen live tickern:

```css
font-variant-numeric: tabular-nums;
```

Sonst springt `1.234` → `1.245` beim Update hin und her.

Keine 5 unterschiedlichen Größen. Hierarchie: Hero (24), Body (14), Label (12) — fertig.

---

## 3. Spacing und Geometrie

- **Spacing-Scale:** 4 / 8 / 12 / 16 / 24 / 32 px. Andere Werte nicht verwenden.
- **Corner-Radius:** 6 px Karten · 4 px Buttons · 2 px Chips/Badges.
- **Border:** 1 px, Default `--border-subtle`.
- **Schatten:** sparsam, nur bei `--bg-overlay` (Modals/Tooltips). `box-shadow: 0 4px 12px rgba(0,0,0,0.32)`.

---

## 4. Komponenten-Prinzipien

### 4.1 Kacheln / Panels

- Hintergrund `--bg-elevated`.
- 16 px Innenabstand, 12 px zwischen Label-Zeile und Hero-Zahl.
- Section-Label oben in `--text-secondary`, uppercase, letter-spacing 0.04em.

### 4.2 Buttons

- Primär: `--accent` Hintergrund, `--bg-base` Text.
- Sekundär: `--bg-elevated` Hintergrund, 1 px `--border-default` Outline, `--text-primary` Text.
- Hover-Primär: `--accent-hover`. Hover-Sekundär: Border → `--accent`.
- Fokus: 2 px `--accent`-Ring außen, `outline-offset: 2px`.

### 4.3 Progress-Bars

- Hintergrund `--border-subtle`, Füllung `--accent-muted` bis 75 %, `--accent` ab 75 %,
  `--warning` ab 90 %, `--danger` ab 100 %.
- Höhe 6 px, Radius 3 px. Keine Animationen im MVP außer `transition: width 200ms ease-out`.

### 4.4 Charts (Tray, Phase 2)

- Recharts, Paletten aus `--chart-*`.
- Keine 3D, keine Schatten, keine Gradient-Füllungen außerhalb `opacity: 0.4` Flächen.
- Legenden immer unten, nie rechts (Space-Effizienz im Tray-Panel).

---

## 5. Tray-Panel (MVP-Wireframe)

```
┌─────────────────────────────────────────┐
│  Ignis              ⚙  ×             │  Header 48 px · --bg-base
├─────────────────────────────────────────┤
│  TODAY                                  │  Section-Label
│  $2.43                                  │  Hero-Zahl (24 px, tabular)
│  1.2M tokens · 14 sessions              │  Metadata (12 px, --text-secondary)
├─────────────────────────────────────────┤
│  THIS MONTH                             │
│  $48.17                                 │
│  29.8M tokens · 182 sessions            │
├─────────────────────────────────────────┤
│  ACTIVE SESSION                         │
│  my-project          2h 14m             │  Name links, Duration rechts
│  312k tokens · $0.71                    │
├─────────────────────────────────────────┤
│  [ Open Dashboard ]  [ CLI: ignis ]  │  Footer-Actions
└─────────────────────────────────────────┘
```

- Breite **360 px** (fix).
- Höhe **content-fit**, max 520 px. Kein Scroll im Panel — wenn mehr Inhalt kommt,
  öffnet der "Open Dashboard"-Button ein eigenes Fenster.
- **Kein Session-Block-Progress-Balken im MVP** — er wandert nach v0.2 (ADR-010).
  Der Platz über "THIS MONTH" bleibt leer bzw. zeigt nur die aktuelle Session-Dauer.

## 6. TUI-Layout (`ignis watch`, v0.2)

```
┌ Ignis watch ─────────────────────── 14:23:05 ─┐
│                                                  │
│  ╭─ Today ─────────────╮ ╭─ Session ─────────╮   │
│  │  $2.43              │ │  2h 14m           │   │
│  │  1.2M tokens        │ │  312k tokens      │   │
│  ╰─────────────────────╯ ╰───────────────────╯   │
│                                                  │
│  ╭─ By Model ──────────────────────────────────╮ │
│  │  claude-opus-4-7    $1.82   890k tokens     │ │
│  │  claude-sonnet-4-6  $0.61   310k tokens     │ │
│  ╰─────────────────────────────────────────────╯ │
│                                                  │
│  ╭─ Burn Rate ─────────────────────────────────╮ │
│  │  ▁▂▃▅▇█▇▅▃▂▁  avg: 14k tok/min              │ │
│  ╰─────────────────────────────────────────────╯ │
│                                                  │
│ [q] quit  [r] refresh  [d] daily  [m] monthly    │
└──────────────────────────────────────────────────┘
```

- ANSI-TrueColor, dieselben Hex-Werte wie §1.
- Bei `NO_COLOR`-Env-Var oder TTY ohne TrueColor: reduzierter 8-Farben-Fallback
  (Bernstein / Dunkelgrau / Default), Terrakotta approximiert als `yellow`/`red`
  gemischt — in der Praxis: `--warning`-Bernstein als Akzent-Fallback.

## 7. Nicht-Ziele

- Kein Light-Mode im MVP.
- Keine Theming-API für Nutzer im MVP.
- Keine Icon-Library-Integration im Tray-MVP. Das Header-⚙ und × sind Unicode, fertig.
- Keine Animationen über `transition` auf Widths/Colors hinaus.
