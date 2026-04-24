import type { SummaryResponse, HeatmapDay } from '../types';
import { formatCost, projectName } from '../components/format';
import { WeekBars } from './charts/WeekBars';
import { LineChart } from './charts/LineChart';
import { useHistoryData } from './useHistoryData';

interface HistoryTabProps {
  month: SummaryResponse | null;
  heatmap: HeatmapDay[];
}

export function HistoryTab({ month, heatmap }: HistoryTabProps) {
  const { weekComparison, costTrend30d } = useHistoryData(heatmap);

  const topProjects = [...(month?.by_project ?? [])]
    .sort((a, b) => parseFloat(b.total_cost_usd) - parseFloat(a.total_cost_usd))
    .slice(0, 8);

  const maxProjectCost = topProjects.reduce(
    (max, p) => Math.max(max, parseFloat(p.total_cost_usd) || 0),
    0,
  );

  return (
    <div style={styles.container}>
      {/* THIS WEEK VS LAST WEEK */}
      <section style={styles.section}>
        <div className="section-label">THIS WEEK VS LAST WEEK</div>
        <WeekBars days={weekComparison} />
      </section>

      <hr className="section-divider" />

      {/* COST TREND 30 DAYS */}
      <section style={styles.section}>
        <div className="section-label">COST TREND 30 DAYS</div>
        <LineChart values={costTrend30d} />
      </section>

      <hr className="section-divider" />

      {/* TOP PROJECTS THIS MONTH */}
      <section style={styles.section}>
        <div className="section-label">TOP PROJECTS THIS MONTH</div>
        {topProjects.length === 0 ? (
          <span style={styles.muted}>no data</span>
        ) : (
          <div style={styles.projectList}>
            {topProjects.map((p) => {
              const cost = parseFloat(p.total_cost_usd) || 0;
              const barPct = maxProjectCost > 0 ? (cost / maxProjectCost) * 100 : 0;
              return (
                <div key={p.project_path} style={styles.projectRow}>
                  <span style={styles.projectName} className="tabular">
                    {projectName(p.project_path)}
                  </span>
                  <div style={styles.barWrap}>
                    <div style={{ ...styles.bar, width: `${barPct}%` }} />
                  </div>
                  <span style={styles.projectCost} className="tabular">
                    {formatCost(p.total_cost_usd)}
                  </span>
                </div>
              );
            })}
          </div>
        )}
      </section>
    </div>
  );
}

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
  },
  section: {
    padding: '12px 16px',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
  },
  muted: {
    fontSize: '12px',
    color: 'var(--text-muted)',
    marginTop: '4px',
  },
  projectList: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '6px',
  },
  projectRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  projectName: {
    fontSize: '11px',
    color: 'var(--text-primary)',
    width: '100px',
    flexShrink: 0,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap' as const,
  },
  barWrap: {
    flex: 1,
    height: '4px',
    background: 'var(--border-subtle)',
    borderRadius: '2px',
    overflow: 'hidden',
  },
  bar: {
    height: '100%',
    background: 'var(--accent)',
    borderRadius: '2px',
    transition: 'width 200ms ease-out',
  },
  projectCost: {
    fontSize: '11px',
    color: 'var(--text-secondary)',
    width: '48px',
    textAlign: 'right' as const,
    flexShrink: 0,
  },
} as const;
