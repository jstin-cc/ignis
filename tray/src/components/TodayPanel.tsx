import type { SummaryResponse } from "../types";
import { formatTokens, formatCost } from "./format";

interface TodayPanelProps {
  data: SummaryResponse | null;
}

export function TodayPanel({ data }: TodayPanelProps) {
  return (
    <section style={styles.panel}>
      <span style={styles.label}>TODAY</span>
      <span style={styles.hero} className="tabular">
        {data ? formatCost(data.total_cost_usd) : "—"}
      </span>
      <span style={styles.meta} className="tabular">
        {data
          ? `${formatTokens(data.total_tokens)} · ${data.event_count} events`
          : "—"}
      </span>
    </section>
  );
}

const styles = {
  panel: {
    backgroundColor: "var(--bg-elevated)",
    padding: "16px",
    display: "flex",
    flexDirection: "column" as const,
    gap: "4px",
  },
  label: {
    fontSize: "12px",
    fontWeight: 500,
    color: "var(--text-secondary)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.04em",
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
