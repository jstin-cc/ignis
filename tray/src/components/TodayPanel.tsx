import type { HeatmapHourBucket, SummaryResponse } from "../types";
import { fmt } from "./format";
import { Sparkline } from "../dashboard/charts/Sparkline";

interface TodayPanelProps {
  data: SummaryResponse | null;
  hourlyWeek?: HeatmapHourBucket[];
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

export function TodaySection({ data, hourlyWeek = [] }: TodayPanelProps) {
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
    </section>
  );
}

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
