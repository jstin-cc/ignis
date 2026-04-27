import { useState } from "react";
import type { HeatmapDay, HeatmapHourBucket } from "../types";

interface HeatmapPanelProps {
  days: HeatmapDay[];
  hourlyWeek: HeatmapHourBucket[];
}

const DAY_LABELS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const HOUR_LABELS = ["0", "6", "12", "18", "24"];
const CELL = 12;
const GAP = 2;
const HOUR_CELL = 11;
const HOUR_GAP = 1;

function isoWeekday(dateStr: string): number {
  const d = new Date(dateStr + "T00:00:00");
  const day = d.getDay();
  return day === 0 ? 7 : day; // Mon=1 … Sun=7
}

function cellColor(value: number, max: number): string {
  if (!isFinite(value) || value <= 0 || max <= 0) return "var(--bg-elevated)";
  const ratio = Math.min(1, value / max);
  const alpha = 0.15 + ratio * 0.85;
  return `rgba(193, 95, 60, ${alpha.toFixed(2)})`;
}

function WeeksView({ days }: { days: HeatmapDay[] }) {
  if (days.length === 0) return null;
  const costs = days.map((d) => parseFloat(d.cost_usd)).filter((v) => isFinite(v));
  const maxCost = costs.length > 0 ? Math.max(...costs) : 0;
  const firstWeekday = isoWeekday(days[0].date);
  const paddingCells = firstWeekday - 1;

  const cells: Array<{ date: string | null; cost: string }> = [
    ...Array.from({ length: paddingCells }, () => ({ date: null, cost: "0" })),
    ...days.map((d) => ({ date: d.date, cost: d.cost_usd })),
  ];

  const numCols = Math.ceil(cells.length / 7);
  const gridWidth = numCols * CELL + (numCols - 1) * GAP;

  return (
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
          if (!c.date) return <div key={`pad-${i}`} style={styles.cellPad} />;
          const costNum = parseFloat(c.cost);
          const title = costNum > 0 ? `${c.date}: $${costNum.toFixed(4)}` : c.date;
          return (
            <div
              key={c.date}
              title={title}
              style={{
                ...styles.cell,
                backgroundColor: cellColor(costNum, maxCost),
              }}
            />
          );
        })}
      </div>
    </div>
  );
}

function WeekView({ hourlyWeek }: { hourlyWeek: HeatmapHourBucket[] }) {
  if (hourlyWeek.length === 0) return <div style={styles.empty}>—</div>;

  const tokens = hourlyWeek.map((b) => b.tokens);
  const maxTokens = Math.max(...tokens);

  // 7 rows (Mon=0…Sun=6) × 24 columns (h=0…23)
  // hourlyWeek[i].hour_start is UTC; day index derived from local time.
  const cells: Array<{ bucket: HeatmapHourBucket; day: number; hour: number }> =
    hourlyWeek.map((b) => {
      const local = new Date(b.hour_start);
      const jsDay = local.getDay(); // 0=Sun…6=Sat
      const day = jsDay === 0 ? 6 : jsDay - 1; // 0=Mon…6=Sun
      return { bucket: b, day, hour: local.getHours() };
    });

  // Build 7×24 matrix indexed [day][hour]
  const matrix: Array<Array<(typeof cells)[number] | null>> = Array.from({ length: 7 }, () =>
    Array(24).fill(null),
  );
  cells.forEach((c) => {
    if (c.day >= 0 && c.day < 7 && c.hour >= 0 && c.hour < 24) {
      matrix[c.day][c.hour] = c;
    }
  });

  const gridCellW = HOUR_CELL;
  const gridCellH = HOUR_CELL;
  const gridGap = HOUR_GAP;
  const gridWidth = 24 * gridCellW + 23 * gridGap;

  const fmtTok = (n: number) =>
    n >= 1_000_000 ? `${(n / 1_000_000).toFixed(1)}M` : n >= 1_000 ? `${(n / 1_000).toFixed(0)}k` : String(n);

  return (
    <div style={styles.gridWrap}>
      <div style={styles.dayLabels}>
        {DAY_LABELS.map((l) => (
          <span key={l} style={{ ...styles.dayLabel, height: `${gridCellH}px`, lineHeight: `${gridCellH}px` }}>
            {l}
          </span>
        ))}
      </div>
      <div>
        {/* Hour tick labels: 0, 6, 12, 18, 24 */}
        <div style={{ ...styles.hourLabels, width: `${gridWidth}px` }}>
          {HOUR_LABELS.map((l) => (
            <span key={l} style={styles.hourLabel}>
              {l}
            </span>
          ))}
        </div>
        <div
          style={{
            display: "grid",
            gridTemplateColumns: `repeat(24, ${gridCellW}px)`,
            gridTemplateRows: `repeat(7, ${gridCellH}px)`,
            gap: `${gridGap}px`,
            width: `${gridWidth}px`,
            gridAutoFlow: "row" as const,
          }}
        >
          {matrix.map((row, dayIdx) =>
            row.map((c, hourIdx) => {
              const tok = c?.bucket.tokens ?? 0;
              const cost = c?.bucket.cost_usd ?? "0";
              const costNum = parseFloat(cost);
              const title = tok > 0
                ? `${DAY_LABELS[dayIdx]} ${hourIdx}:00–${hourIdx + 1}:00\n${fmtTok(tok)} tok · $${costNum.toFixed(4)}`
                : `${DAY_LABELS[dayIdx]} ${hourIdx}:00`;
              return (
                <div
                  key={`${dayIdx}-${hourIdx}`}
                  title={title}
                  style={{
                    width: `${gridCellW}px`,
                    height: `${gridCellH}px`,
                    borderRadius: "2px",
                    backgroundColor: cellColor(tok, maxTokens),
                  }}
                />
              );
            }),
          )}
        </div>
      </div>
    </div>
  );
}

export function HeatmapPanel({ days, hourlyWeek }: HeatmapPanelProps) {
  const [view, setView] = useState<"weeks" | "week">("weeks");

  return (
    <section style={styles.panel}>
      <div style={styles.header}>
        <div className="section-label">
          {view === "weeks" ? "ACTIVITY (12 WEEKS)" : "ACTIVITY (THIS WEEK)"}
        </div>
        <button
          style={styles.toggle}
          onClick={() => setView((v) => (v === "weeks" ? "week" : "weeks"))}
        >
          {view === "weeks" ? "This Week" : "12 Weeks"}
        </button>
      </div>
      {view === "weeks" ? <WeeksView days={days} /> : <WeekView hourlyWeek={hourlyWeek} />}
    </section>
  );
}

const styles = {
  panel: {
    padding: "10px 16px 12px",
  },
  header: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: "6px",
  },
  toggle: {
    background: "none",
    border: "none",
    cursor: "pointer",
    fontSize: "9px",
    color: "var(--text-tertiary)",
    padding: "2px 4px",
    borderRadius: "3px",
    letterSpacing: "0.04em",
    textTransform: "uppercase" as const,
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
  hourLabels: {
    display: "flex",
    justifyContent: "space-between",
    marginBottom: "2px",
  },
  hourLabel: {
    fontSize: "9px",
    color: "var(--text-tertiary)",
    width: "16px",
    textAlign: "center" as const,
  },
  empty: {
    fontSize: "12px",
    color: "var(--text-tertiary)",
    padding: "8px 0",
  },
} as const;
