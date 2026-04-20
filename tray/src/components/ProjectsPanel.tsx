import type { SummaryResponse } from "../types";
import { formatCost, projectName } from "./format";

interface ProjectsPanelProps {
  data: SummaryResponse | null;
}

export function ProjectsPanel({ data }: ProjectsPanelProps) {
  if (!data || data.by_project.length === 0) return null;

  const top5 = [...data.by_project]
    .sort((a, b) => parseFloat(b.total_cost_usd) - parseFloat(a.total_cost_usd))
    .slice(0, 5);

  const maxCost = parseFloat(top5[0].total_cost_usd) || 1;

  return (
    <section style={styles.panel}>
      <span style={styles.label}>PROJECTS (TODAY)</span>
      <div style={styles.list}>
        {top5.map((p) => {
          const cost = parseFloat(p.total_cost_usd);
          const barPct = Math.round((cost / maxCost) * 100);
          return (
            <div key={p.project_path} style={styles.row}>
              <span style={styles.name} title={p.project_path}>
                {projectName(p.project_path)}
              </span>
              <div style={styles.barTrack}>
                <div
                  style={{
                    ...styles.barFill,
                    width: `${barPct}%`,
                  }}
                />
              </div>
              <span style={styles.cost} className="tabular">
                {formatCost(p.total_cost_usd)}
              </span>
            </div>
          );
        })}
      </div>
    </section>
  );
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
  list: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "6px",
  },
  row: {
    display: "grid",
    gridTemplateColumns: "1fr 80px 52px",
    alignItems: "center",
    gap: "8px",
  },
  name: {
    fontSize: "13px",
    color: "var(--text-primary)",
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap" as const,
  },
  barTrack: {
    height: "4px",
    borderRadius: "2px",
    backgroundColor: "var(--border-subtle)",
    overflow: "hidden",
  },
  barFill: {
    height: "100%",
    borderRadius: "2px",
    backgroundColor: "var(--accent-muted)",
    transition: "width 200ms ease-out",
  },
  cost: {
    fontSize: "13px",
    color: "var(--text-secondary)",
    textAlign: "right" as const,
  },
} as const;
