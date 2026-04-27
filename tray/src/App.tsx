import { useState } from "react";
import { useUsageData } from "./useUsageData";
import { useBlockNotifications } from "./hooks/useBlockNotifications";
import { useAutoStart } from "./hooks/useAutoStart";
import { useUpdater } from "./hooks/useUpdater";
import { usePlanConfig } from "./hooks/usePlanConfig";
import { useAnthropicUsage } from "./hooks/useAnthropicUsage";
import { TabBar } from "./components/TabBar";
import type { TabId } from "./components/TabBar";
import { TodaySection } from "./components/TodayPanel";
import { MonthPanel, WeekSection } from "./components/MonthPanel";
import { BlockPanel } from "./components/BlockPanel";
import { ProjectsPanel } from "./components/ProjectsPanel";
import { HeatmapPanel } from "./components/HeatmapPanel";
import { SessionSection } from "./components/ActiveSessionPanel";
import { SettingsTab } from "./components/SettingsTab";
import { Footer } from "./components/Footer";
import { Dashboard } from './dashboard/Dashboard';

export function App() {
  const { today, week, month, last30Days, activeSession, activeBlock, heatmap, hourlyHeatmapWeek, error } = useUsageData();
  const { isEnabled, toggle } = useAutoStart();
  const { checking, result, error: updateError, checkForUpdate } = useUpdater();
  const { plan, setPlan, setThresholds, setBudgets } = usePlanConfig();
  const { usage: anthropicUsage, error: usageError } = useAnthropicUsage(plan.usage_poll_interval_secs);
  useBlockNotifications(
    activeBlock,
    plan.block_alert_thresholds,
    week?.total_cost_usd,
    plan.weekly_budget_usd,
    month?.total_cost_usd,
    plan.monthly_budget_usd,
  );
  const [activeTab, setActiveTab] = useState<TabId>('today');
  const [dashboardOpen, setDashboardOpen] = useState(false);

  return (
    <div style={styles.shell}>
      {/* Header — 48px */}
      <header style={styles.header} data-tauri-drag-region>
        <span style={styles.appName} data-tauri-drag-region>
          Ignis
        </span>
        <div style={styles.headerActions}>
          <button
            style={styles.iconBtn}
            aria-label="Close"
            onClick={() => {
              void import("@tauri-apps/api/window").then((m) =>
                m.getCurrentWindow().hide()
              );
            }}
          >
            ×
          </button>
        </div>
      </header>

      {/* TabBar — 36px */}
      <TabBar active={activeTab} onChange={setActiveTab} />

      {/* Content */}
      <div style={styles.content}>
        {dashboardOpen && (
          <Dashboard
            onClose={() => setDashboardOpen(false)}
            today={today}
            last30Days={last30Days}
            activeSession={activeSession}
            activeBlock={activeBlock}
            heatmap={heatmap}
          />
        )}

        {error && (
          <div style={styles.errorBanner}>
            API nicht erreichbar — starte ignis-api
          </div>
        )}

        {activeTab === 'today' && (
          <>
            <TodaySection data={today} hourlyWeek={hourlyHeatmapWeek} />
            <hr className="section-divider" />
            <WeekSection data={week} />
            <hr className="section-divider" />
            <BlockPanel block={activeBlock} usage={anthropicUsage} usageError={usageError} alertThresholds={plan.block_alert_thresholds} />
            <hr className="section-divider" />
            <SessionSection session={activeSession} />
          </>
        )}

        {activeTab === 'month' && (
          <MonthPanel data={month} variant="full" />
        )}

        {activeTab === 'projects' && (
          <ProjectsPanel data={today} />
        )}

        {activeTab === 'heatmap' && (
          <HeatmapPanel days={heatmap} hourlyWeek={hourlyHeatmapWeek} />
        )}

        {activeTab === 'settings' && (
          <SettingsTab
            autoStart={{ isEnabled, toggle }}
            plan={plan}
            setPlan={setPlan}
            setThresholds={setThresholds}
            setBudgets={setBudgets}
            updater={{ checking, result, error: updateError, checkForUpdate }}
          />
        )}
      </div>

      {/* Footer — 56px */}
      <Footer onOpenDashboard={() => setDashboardOpen(true)} />
    </div>
  );
}

const styles = {
  shell: {
    width: "360px",
    height: "100vh",
    display: "flex",
    flexDirection: "column" as const,
    backgroundColor: "var(--bg-base)",
    overflow: "hidden",
  },
  header: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    height: "48px",
    padding: "0 16px",
    backgroundColor: "var(--bg-base)",
    flexShrink: 0,
    cursor: "grab",
  },
  content: {
    flex: 1,
    minHeight: 0,
    overflowY: "auto" as const,
    display: "flex",
    flexDirection: "column" as const,
    position: "relative" as const,
  },
  appName: {
    fontSize: "14px",
    fontWeight: 600,
    color: "var(--text-primary)",
    letterSpacing: "0.02em",
  },
  headerActions: {
    display: "flex",
    alignItems: "center",
    gap: "4px",
  },
  iconBtn: {
    width: "28px",
    height: "28px",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    backgroundColor: "transparent",
    border: "none",
    borderRadius: "4px",
    color: "var(--text-secondary)",
    fontSize: "16px",
    cursor: "pointer",
    fontFamily: "var(--font-sans)",
    lineHeight: 1,
    transition: "color 120ms ease-out, background-color 120ms ease-out",
  },
  errorBanner: {
    padding: "8px 16px",
    fontSize: "12px",
    color: "var(--danger)",
    backgroundColor: "var(--bg-overlay)",
    borderBottom: "1px solid var(--border-subtle)",
    flexShrink: 0,
  },
} as const;
