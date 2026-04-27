import type { HeatmapHourBucket, SummaryResponse } from "../types";
import { fmt } from "./format";
import { Sparkline } from "../dashboard/charts/Sparkline";

interface TodayPanelProps {
  data: SummaryResponse | null;
  hourlyWeek?: HeatmapHourBucket[];
  isEmpty?: boolean;
}

/** Derive 24 hourly token values for today from the weekly 168-bucket data. */
function todayHourlyTokens(hourlyWeek: HeatmapHourBucket[]): number[] {
  const todayStr = new Date().toLocaleDateString("en-CA"); // YYYY-MM-DD in local time
  const buckets = Array(24).fill(0) as number[];
  for (const b of hourlyWeek) {
    const local = new Date(b.hour_start);
    if (local.toLocaleDateString("en-CA") === todayStr) {
      const h = local.getHours();
      if (h >= 0 && h < 24) buckets[h] = b.tokens;
    }
  }
  return buckets;
}

export function TodaySection({ data, hourlyWeek = [], isEmpty = false }: TodayPanelProps) {
  const hourlyTokens = todayHourlyTokens(hourlyWeek);
  const hasActivity = hourlyTokens.some((v) => v > 0);

  return (
    <section style={styles.section}>
      <div className="section-label">TODAY</div>
      <span style={styles.hero} className="tabular">
        {data ? fmt.usd(parseFloat(data.total_cost_usd)) : "—"}
      </span>
      <span style={styles.meta} className="tabular">
        {data
          ? `${fmt.tok(data.total_tokens)} tok · ${data.event_count} events`
          : "—"}
      </span>
      {hasActivity && (
        <div style={styles.sparklineWrap}>
          <Sparkline values={hourlyTokens} width={328} height={28} />
        </div>
      )}
      {isEmpty && <EmptyHint />}
    </section>
  );
}

function EmptyHint() {
  return (
    <div style={emptyStyles.box}>
      <span style={emptyStyles.title}>Noch keine Logs gefunden</span>
      <span style={emptyStyles.body}>
        Ignis liest JSONL-Logs aus:
      </span>
      <code style={emptyStyles.path}>%USERPROFILE%\.claude\projects\</code>
      <span style={emptyStyles.body}>
        Starte Claude Code und führe einen Task aus — Daten erscheinen automatisch.
      </span>
    </div>
  );
}

const emptyStyles = {
  box: {
    marginTop: "12px",
    padding: "10px 12px",
    backgroundColor: "var(--bg-elevated)",
    borderRadius: "6px",
    border: "1px solid var(--border-subtle)",
    display: "flex",
    flexDirection: "column" as const,
    gap: "6px",
  },
  title: {
    fontSize: "12px",
    fontWeight: 600,
    color: "var(--text-secondary)",
  },
  body: {
    fontSize: "11px",
    color: "var(--text-muted)",
    lineHeight: 1.4,
  },
  path: {
    fontFamily: "var(--font-mono)",
    fontSize: "10px",
    color: "var(--accent)",
    letterSpacing: "0.01em",
  },
} as const;

/** @deprecated Verwende TodaySection */
export function TodayPanel({ data, hourlyWeek }: TodayPanelProps) {
  return <TodaySection data={data} hourlyWeek={hourlyWeek} />;
}

const styles = {
  section: {
    padding: "16px",
    display: "flex",
    flexDirection: "column" as const,
    gap: "4px",
  },
  hero: {
    fontSize: "24px",
    fontWeight: 600,
    color: "var(--text-primary)",
    marginTop: "4px",
  },
  meta: {
    fontSize: "12px",
    color: "var(--text-secondary)",
  },
  sparklineWrap: {
    marginTop: "8px",
  },
} as const;
