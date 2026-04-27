import type { ActiveBlock, AnthropicUsage } from "../types";
import { fmt, formatCost } from "./format";
import { progressClass } from "./MonthPanel";

interface BlockPanelProps {
  block: ActiveBlock | null;
  usage: AnthropicUsage | null;
  usageError?: string | null;
  alertThresholds?: number[];
}

function errorMessage(err: string): string {
  if (/do_refresh|401|token|auth/i.test(err))
    return 'Anthropic session expired. Re-run `claude` to reconnect.';
  if (/network|fetch|offline/i.test(err))
    return 'Offline — cached values shown.';
  return err;
}

export function BlockPanel({ block, usage, usageError, alertThresholds }: BlockPanelProps) {
  if (usage) {
    return <ThreeBarsPanel block={block} usage={usage} />;
  }
  return (
    <>
      {usageError && (
        <div style={styles.usageErrorBanner}>{errorMessage(usageError)}</div>
      )}
      <FallbackBar block={block} alertThresholds={alertThresholds} />
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
      <div className="section-label">USAGE LIMITS</div>

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
        <>
          <UsageRow
            name="Extra"
            pct={usage.extra_usage.pct}
            suffix={
              usage.extra_usage.is_unlimited
                ? `$${usage.extra_usage.used_usd} used`
                : `$${usage.extra_usage.used_usd} / $${usage.extra_usage.monthly_limit_usd}`
            }
          />
          {usage.extra_usage.pct > 0 && (
            <div className="extra-usage">
              <span>Extra Usage</span>
              <span>+{fmt.usd(parseFloat(usage.extra_usage.used_usd))}</span>
            </div>
          )}
        </>
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
  const cls = progressClass(clamped);

  return (
    <div style={styles.usageRow}>
      <span style={styles.rowName}>{name}</span>
      <div style={styles.rowRight}>
        <div className="progress-track">
          <div
            className={`progress-fill ${cls}`}
            style={{ width: `${clamped}%`, transition: 'width 200ms ease-out' }}
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

function nextAlertThreshold(thresholds: number[], currentPct: number): number | null {
  const sorted = [...thresholds].sort((a, b) => a - b);
  return sorted.find((t) => t > currentPct) ?? null;
}

function FallbackBar({
  block,
  alertThresholds = [50, 75, 90, 100],
}: {
  block: ActiveBlock | null;
  alertThresholds?: number[];
}) {
  if (!block) {
    return (
      <section style={styles.panel}>
        <div className="section-label">ACTIVE BLOCK</div>
        <span style={styles.empty}>no active billing block</span>
      </section>
    );
  }

  const tokenPct = Math.min(100, Math.max(0, block.block_token_pct));
  const remaining = remainingTime(block.end);
  const burnRate = computeBurnRate(block);
  const cls = progressClass(tokenPct);
  const nextAlert = nextAlertThreshold(alertThresholds, tokenPct);

  return (
    <section style={styles.panel}>
      <div className="section-label">ACTIVE BLOCK</div>
      <div className="progress-track">
        <div
          className={`progress-fill ${cls}`}
          style={{ width: `${tokenPct}%`, transition: 'width 200ms ease-out' }}
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
      {nextAlert !== null && (
        <span style={styles.nextAlert}>Next alert: {nextAlert}%</span>
      )}
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
    padding: "16px",
    display: "flex",
    flexDirection: "column" as const,
    gap: "10px",
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
    backgroundColor: "var(--bg-overlay)",
    borderBottom: "1px solid var(--border-subtle)",
    wordBreak: "break-all" as const,
  },
  nextAlert: {
    fontSize: "11px",
    color: "var(--text-muted)",
  },
} as const;
