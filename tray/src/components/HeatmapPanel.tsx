import type { HeatmapDay } from "../types";

interface HeatmapPanelProps {
  days: HeatmapDay[];
}

const DAY_LABELS = ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"];
const CELL = 12;
const GAP = 2;

function isoWeekday(dateStr: string): number {
  const d = new Date(dateStr + "T00:00:00");
  const day = d.getDay();
  return day === 0 ? 7 : day; // Mon=1 … Sun=7
}

function cellColor(costUsd: string, maxCost: number): string {
  const v = parseFloat(costUsd);
  if (!isFinite(v) || v <= 0 || maxCost <= 0) return "var(--bg-elevated)";
  const ratio = Math.min(1, v / maxCost);
  const alpha = 0.15 + ratio * 0.85;
  return `rgba(193, 95, 60, ${alpha.toFixed(2)})`;
}

export function HeatmapPanel({ days }: HeatmapPanelProps) {
  if (days.length === 0) return null;

  const costs = days.map((d) => parseFloat(d.cost_usd)).filter((v) => isFinite(v));
  const maxCost = costs.length > 0 ? Math.max(...costs) : 0;

  // Align grid so that column 0 starts on a Monday.
  const firstWeekday = isoWeekday(days[0].date);
  const paddingCells = firstWeekday - 1;

  const cells: Array<{ date: string | null; cost: string }> = [
    ...Array.from({ length: paddingCells }, () => ({ date: null, cost: "0" })),
    ...days.map((d) => ({ date: d.date, cost: d.cost_usd })),
  ];

  const numCols = Math.ceil(cells.length / 7);
  const gridWidth = numCols * CELL + (numCols - 1) * GAP;

  return (
    <section style={styles.panel}>
      <span style={styles.label}>AKTIVITÄT (12 WOCHEN)</span>
      <div style={styles.gridWrap}>
        <div style={styles.dayLabels}>
          {DAY_LABELS.map((l) => (
            <span key={l} style={styles.dayLabel}>
              {l}
            </span>
          ))}
        </div>
        <div
          style={{
            ...styles.grid,
            gridTemplateColumns: `repeat(${numCols}, ${CELL}px)`,
            gridTemplateRows: `repeat(7, ${CELL}px)`,
            width: `${gridWidth}px`,
          }}
        >
          {cells.map((c, i) => {
            if (!c.date) {
              return <div key={`pad-${i}`} style={styles.cellPad} />;
            }
            const costNum = parseFloat(c.cost);
            const title =
              costNum > 0
                ? `${c.date}: $${costNum.toFixed(4)}`
                : c.date;
            return (
              <div
                key={c.date}
                title={title}
                style={{
                  ...styles.cell,
                  backgroundColor: cellColor(c.cost, maxCost),
                }}
              />
            );
          })}
        </div>
      </div>
    </section>
  );
}

const styles = {
  panel: {
    padding: "10px 16px 12px",
  },
  label: {
    display: "block",
    fontSize: "10px",
    fontWeight: 600,
    color: "var(--text-tertiary)",
    letterSpacing: "0.08em",
    marginBottom: "8px",
  },
  gridWrap: {
    display: "flex",
    alignItems: "flex-start",
    gap: "6px",
  },
  dayLabels: {
    display: "flex",
    flexDirection: "column" as const,
    gap: `${GAP}px`,
  },
  dayLabel: {
    fontSize: "9px",
    color: "var(--text-tertiary)",
    lineHeight: `${CELL}px`,
    height: `${CELL}px`,
    width: "14px",
    textAlign: "right" as const,
  },
  grid: {
    display: "grid",
    gap: `${GAP}px`,
    gridAutoFlow: "column" as const,
  },
  cell: {
    width: `${CELL}px`,
    height: `${CELL}px`,
    borderRadius: "2px",
  },
  cellPad: {
    width: `${CELL}px`,
    height: `${CELL}px`,
  },
} as const;
