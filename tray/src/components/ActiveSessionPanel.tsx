import type { Session } from "../types";
import { fmt, formatCost, formatDuration, projectName } from "./format";

interface ActiveSessionPanelProps {
  session: Session | null;
}

export function SessionSection({ session }: ActiveSessionPanelProps) {
  return (
    <section style={styles.section}>
      <div className="section-label">ACTIVE SESSION</div>
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
            {fmt.tok(sessionTotalTokens(session))} tok &middot;{" "}
            {formatCost(session.total_cost_usd)}
          </span>
        </>
      ) : (
        <span style={styles.noSession}>no active session</span>
      )}
    </section>
  );
}

/** @deprecated Verwende SessionSection */
export function ActiveSessionPanel({ session }: ActiveSessionPanelProps) {
  return <SessionSection session={session} />;
}

function sessionTotalTokens(session: Session): number {
  return session.by_model.reduce((sum, m) => sum + m.tokens, 0);
}

const styles = {
  section: {
    padding: "16px",
    display: "flex",
    flexDirection: "column" as const,
    gap: "4px",
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
