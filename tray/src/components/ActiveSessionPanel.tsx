import type { Session } from "../types";
import { formatCost, formatDuration, projectName } from "./format";

interface ActiveSessionPanelProps {
  session: Session | null;
}

export function ActiveSessionPanel({ session }: ActiveSessionPanelProps) {
  return (
    <section style={styles.panel}>
      <span style={styles.label}>ACTIVE SESSION</span>
      {session ? (
        <>
          <div style={styles.row}>
            <span style={styles.projectName} className="tabular">
              {projectName(session.project_path)}
            </span>
            <span style={styles.duration} className="tabular">
              {formatDuration(session.first_seen, session.last_seen)}
            </span>
          </div>
          <span style={styles.meta} className="tabular">
            {/* total_tokens is not in SessionDto — derive from by_model */}
            {formatSessionTokens(session)} &middot;{" "}
            {formatCost(session.total_cost_usd)}
          </span>
        </>
      ) : (
        <span style={styles.noSession}>no active session</span>
      )}
    </section>
  );
}

function formatSessionTokens(session: Session): string {
  const total = session.by_model.reduce((sum, m) => sum + m.tokens, 0);
  // Reuse the same formatTokens helper used in the other panels.
  return formatTokensRaw(total);
}

function formatTokensRaw(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M tokens`;
  if (n >= 1_000) return `${Math.round(n / 1_000)}k tokens`;
  return `${n} tokens`;
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
  row: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "baseline",
    marginTop: "8px",
  },
  projectName: {
    fontSize: "14px",
    fontWeight: 500,
    color: "var(--text-primary)",
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap" as const,
    maxWidth: "200px",
  },
  duration: {
    fontSize: "14px",
    color: "var(--accent)",
    fontWeight: 500,
  },
  meta: {
    fontSize: "12px",
    color: "var(--text-secondary)",
  },
  noSession: {
    fontSize: "13px",
    color: "var(--text-muted)",
    marginTop: "8px",
  },
} as const;
