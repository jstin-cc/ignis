import type { SummaryResponse } from "../types";
import { fmt } from "./format";

interface TodayPanelProps {
  data: SummaryResponse | null;
}

export function TodaySection({ data }: TodayPanelProps) {
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
    </section>
  );
}

/** @deprecated Verwende TodaySection */
export function TodayPanel({ data }: TodayPanelProps) {
  return <TodaySection data={data} />;
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
} as const;
