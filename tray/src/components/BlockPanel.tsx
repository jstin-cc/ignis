import type { ActiveBlock, AnthropicUsage } from "../types";
import { formatCost } from "./format";

interface BlockPanelProps {
  block: ActiveBlock | null;
  usage: AnthropicUsage | null;
  usageError?: string | null;
}

export function BlockPanel({ block, usage, usageError }: BlockPanelProps) {
  if (usage) {
    return <ThreeBarsPanel block={block} usage={usage} />;
  }
  return (
    <>
      {usageError && (
        <div style={styles.usageErrorBanner}>{usageError}</div>
      )}
      <FallbackBar block={block} />
    </>
  );
}

// ── Three-bar panel (Anthropic OAuth data) ────────────────────────────────────

function ThreeBarsPanel({
  block,
  usage,
}: {
  block: ActiveBlock | null;
  usage: AnthropicUsage;
}) {
  const fiveHourRemaining = block ? remainingTime(block.end) : null;
  const weekRemaining = usage.seven_day ? remainingLabel(usage.seven_day.resets_at) : null;

  return (
    <section style={styles.panel}>
      <span style={styles.label}>USAGE LIMITS</span>

      {usage.five_hour && (
        <UsageRow
          name="5h Block"
          pct={usage.five_hour.utilization}
          suffix={fiveHourRemaining ? `resets in ${fiveHourRemaining}` : undefined}
          cost={block ? formatCost(block.cost_usd) : undefined}
        />
      )}

      {usage.seven_day && (
        <UsageRow
          name="This Week"
          pct={usage.seven_day.utilization}
          suffix={weekRemaining ?? undefined}
        />
      )}

      {usage.extra_usage?.is_enabled && (
        <UsageRow
          name="Extra"
          pct={usage.extra_usage.pct}
          suffix={
            usage.extra_usage.is_unlimited
              ? `$${usage.extra_usage.used_usd} used`
              : `$${usage.extra_usage.used_usd} / $${usage.extra_usage.monthly_limit_usd}`
          }
        />
      )}
    </section>
  );
}

function UsageRow({
  name,
  pct,
  suffix,
  cost,
}: {
  name: string;
  pct: number;
  suffix?: string;
  cost?: string;
}) {
  const clamped = Math.min(100, Math.max(0, pct));
  const barColor =
    clamped >= 90
      ? "var(--warning)"
      : clamped >= 75
        ? "var(--accent)"
        : "var(--accent-muted)";

  return (
    <div style={styles.usageRow}>
      <span style={styles.rowName}>{name}</span>
      <div style={styles.rowRight}>
        <div style={styles.barTrack}>
          <div
            style={{
              ...styles.barFill,
              width: `${clamped}%`,
              backgroundColor: barColor,
              transition: "width 200ms ease-out, background-color 200ms ease-out",
            }}
          />
        </div>
        <div style={styles.rowMeta}>
          <span style={styles.pct} className="tabular">
            {clamped}%
          </span>
          {suffix && (
            <span style={styles.suffix} className="tabular">
              {suffix}
            </span>
          )}
          {cost && (
            <span style={styles.costSmall} className="tabular">
              {cost}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

// ── Fallback single bar (JSONL token data) ────────────────────────────────────

function FallbackBar({ block }: { block: ActiveBlock | null }) {
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
      <div style={styles.fallbackRow}>
        <span style={styles.costSmall} className="tabular">
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

// ── Helpers ───────────────────────────────────────────────────────────────────

function remainingTime(endIso: string): string {
  const diffMs = new Date(endIso).getTime() - Date.now();
  if (diffMs <= 0) return "0m";
  const totalMin = Math.floor(diffMs / 60_000);
  const h = Math.floor(totalMin / 60);
  const m = totalMin % 60;
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

function remainingLabel(resetsAtIso: string): string {
  const diffMs = new Date(resetsAtIso).getTime() - Date.now();
  if (diffMs <= 0) return "resets soon";
  const totalH = Math.floor(diffMs / 3_600_000);
  if (totalH < 24) return `resets in ${totalH}h`;
  const d = Math.floor(totalH / 24);
  return `resets in ${d}d`;
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

// ── Styles ────────────────────────────────────────────────────────────────────

const styles = {
  panel: {
    backgroundColor: "var(--bg-elevated)",
    padding: "16px",
    display: "flex",
    flexDirection: "column" as const,
    gap: "10px",
  },
  label: {
    fontSize: "12px",
    fontWeight: 500,
    color: "var(--text-secondary)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.04em",
  },
  usageRow: {
    display: "flex",
    alignItems: "flex-start",
    gap: "8px",
  },
  rowName: {
    fontSize: "12px",
    color: "var(--text-secondary)",
    minWidth: "68px",
    paddingTop: "1px",
    flexShrink: 0,
  },
  rowRight: {
    flex: 1,
    display: "flex",
    flexDirection: "column" as const,
    gap: "3px",
  },
  barTrack: {
    height: "5px",
    borderRadius: "3px",
    backgroundColor: "var(--border-subtle)",
    overflow: "hidden",
  },
  barFill: {
    height: "100%",
    borderRadius: "3px",
  },
  rowMeta: {
    display: "flex",
    alignItems: "baseline",
    gap: "6px",
    flexWrap: "wrap" as const,
  },
  pct: {
    fontSize: "12px",
    fontWeight: 600,
    color: "var(--text-primary)",
  },
  suffix: {
    fontSize: "11px",
    color: "var(--text-muted)",
  },
  costSmall: {
    fontSize: "12px",
    color: "var(--text-secondary)",
  },
  // fallback-only styles
  pctLabel: {
    fontSize: "13px",
    fontWeight: 500,
    color: "var(--text-primary)",
  },
  fallbackRow: {
    display: "flex",
    alignItems: "baseline",
    gap: "8px",
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
  usageErrorBanner: {
    padding: "6px 16px",
    fontSize: "11px",
    color: "var(--danger)",
    backgroundColor: "var(--bg-surface)",
    borderBottom: "1px solid var(--border-subtle)",
    wordBreak: "break-all" as const,
  },
} as const;
