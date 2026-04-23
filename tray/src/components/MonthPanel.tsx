import type { SummaryResponse } from "../types";
import { fmt } from "./format";

interface MonthPanelProps {
  data: SummaryResponse | null;
  variant?: 'week' | 'full';
}

export function progressClass(pct: number): string {
  if (pct >= 100) return 'progress-fill--danger';
  if (pct >= 90)  return 'progress-fill--warning';
  if (pct >= 75)  return 'progress-fill--high';
  return '';
}

function daysProgressPct(): number {
  const now = new Date();
  const daysInMonth = new Date(now.getFullYear(), now.getMonth() + 1, 0).getDate();
  return Math.round((now.getDate() / daysInMonth) * 100);
}

export function WeekSection({ data }: { data: SummaryResponse | null }) {
  const pct = daysProgressPct();
  const cls = progressClass(pct);

  return (
    <section style={styles.section}>
      <div className="section-label">THIS MONTH</div>
      <span style={styles.hero} className="tabular">
        {data ? fmt.usd(parseFloat(data.total_cost_usd)) : "—"}
      </span>
      <div className="progress-track" style={styles.track}>
        <div className={`progress-fill ${cls}`} style={{ width: `${pct}%` }} />
      </div>
      <span style={styles.meta} className="tabular">
        {data
          ? `${fmt.tok(data.total_tokens)} tok · ${data.event_count} events`
          : "—"}
      </span>
    </section>
  );
}

export function MonthPanel({ data, variant = 'full' }: MonthPanelProps) {
  if (variant === 'week') return <WeekSection data={data} />;

  return (
    <section style={styles.section}>
      <div className="section-label">THIS MONTH</div>
      <span style={styles.hero} className="tabular">
        {data ? fmt.usd(parseFloat(data.total_cost_usd)) : "—"}
      </span>
      <span style={styles.meta} className="tabular">
        {data
          ? `${fmt.tok(data.total_tokens)} tok · ${data.event_count} events`
          : "—"}
      </span>
    </section>
  );
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
  track: {
    marginTop: "6px",
  },
  meta: {
    fontSize: "12px",
    color: "var(--text-secondary)",
    marginTop: "2px",
  },
} as const;
