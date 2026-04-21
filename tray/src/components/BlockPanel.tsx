import type { ActiveBlock } from "../types";
import { formatCost } from "./format";

interface BlockPanelProps {
  block: ActiveBlock | null;
}

export function BlockPanel({ block }: BlockPanelProps) {
  if (!block) {
    return (
      <section style={styles.panel}>
        <span style={styles.label}>ACTIVE BLOCK</span>
        <span style={styles.empty}>no active billing block</span>
      </section>
    );
  }

  const tokenPct = Math.min(100, Math.max(0, block.block_token_pct));
  const remaining = remainingTime(block.end);
  const burnRate = computeBurnRate(block);

  const barColor =
    tokenPct >= 90
      ? "var(--warning)"
      : tokenPct >= 75
        ? "var(--accent)"
        : "var(--accent-muted)";

  return (
    <section style={styles.panel}>
      <span style={styles.label}>ACTIVE BLOCK</span>

      <div style={styles.barTrack}>
        <div
          style={{
            ...styles.barFill,
            width: `${tokenPct}%`,
            backgroundColor: barColor,
            transition: "width 200ms ease-out, background-color 200ms ease-out",
          }}
        />
      </div>

      <span style={styles.pctLabel} className="tabular">
        {tokenPct}% used · resets in {remaining}
      </span>

      <div style={styles.row}>
        <span style={styles.cost} className="tabular">
          {formatCost(block.cost_usd)}
        </span>
        {burnRate && (
          <span style={styles.rate} className="tabular">
            {burnRate}/h
          </span>
        )}
      </div>
    </section>
  );
}

function remainingTime(endIso: string): string {
  const diffMs = new Date(endIso).getTime() - Date.now();
  if (diffMs <= 0) return "0m";
  const totalMin = Math.floor(diffMs / 60_000);
  const h = Math.floor(totalMin / 60);
  const m = totalMin % 60;
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

function computeBurnRate(block: ActiveBlock): string | null {
  if (block.percent_elapsed <= 0) return null;
  const totalH =
    (new Date(block.end).getTime() - new Date(block.start).getTime()) / 3_600_000;
  const elapsedH = (block.percent_elapsed / 100) * totalH;
  if (elapsedH < 0.01) return null;
  const cost = parseFloat(block.cost_usd);
  if (isNaN(cost)) return null;
  return formatCost((cost / elapsedH).toFixed(4));
}

const styles = {
  panel: {
    backgroundColor: "var(--bg-elevated)",
    padding: "16px",
    display: "flex",
    flexDirection: "column" as const,
    gap: "8px",
  },
  label: {
    fontSize: "12px",
    fontWeight: 500,
    color: "var(--text-secondary)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.04em",
  },
  barTrack: {
    height: "6px",
    borderRadius: "3px",
    backgroundColor: "var(--border-subtle)",
    overflow: "hidden",
  },
  barFill: {
    height: "100%",
    borderRadius: "3px",
  },
  pctLabel: {
    fontSize: "13px",
    fontWeight: 500,
    color: "var(--text-primary)",
  },
  row: {
    display: "flex",
    alignItems: "baseline",
    gap: "8px",
    flexWrap: "wrap" as const,
  },
  cost: {
    fontSize: "13px",
    color: "var(--text-secondary)",
  },
  rate: {
    fontSize: "12px",
    color: "var(--accent)",
    fontWeight: 500,
  },
  empty: {
    fontSize: "13px",
    color: "var(--text-muted)",
  },
} as const;
