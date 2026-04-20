import { useUsageData } from "./useUsageData";
import { useBlockNotifications } from "./hooks/useBlockNotifications";
import { TodayPanel } from "./components/TodayPanel";
import { MonthPanel } from "./components/MonthPanel";
import { BlockPanel } from "./components/BlockPanel";
import { ProjectsPanel } from "./components/ProjectsPanel";
import { ActiveSessionPanel } from "./components/ActiveSessionPanel";
import { Footer } from "./components/Footer";

export function App() {
  const { today, month, activeSession, activeBlock, error } = useUsageData();
  useBlockNotifications(activeBlock);

  function handleOpenDashboard() {
    // Phase 2: open a full dashboard window via Tauri IPC.
    // In MVP this is a no-op placeholder.
  }

  return (
    <div style={styles.shell}>
      <header style={styles.header}>
        <span style={styles.appName}>WinUsage</span>
        <div style={styles.headerActions}>
          {error && <span style={styles.errorDot} title={error}>!</span>}
          <button
            style={styles.iconBtn}
            aria-label="Settings"
            onClick={() => {
              /* Phase 2: open settings */
            }}
          >
            ⚙
          </button>
          <button
            style={styles.iconBtn}
            aria-label="Close"
            onClick={() => {
              /* Tauri: hide window — wired in main.rs via tray toggle */
            }}
          >
            ×
          </button>
        </div>
      </header>

      <hr className="section-divider" />
      <TodayPanel data={today} />
      <hr className="section-divider" />
      <MonthPanel data={month} />
      <hr className="section-divider" />
      <BlockPanel block={activeBlock} />
      {today?.by_project.length ? (
        <>
          <hr className="section-divider" />
          <ProjectsPanel data={today} />
        </>
      ) : null}
      <hr className="section-divider" />
      <ActiveSessionPanel session={activeSession} />
      <Footer onOpenDashboard={handleOpenDashboard} />
    </div>
  );
}

const styles = {
  shell: {
    width: "360px",
    display: "flex",
    flexDirection: "column" as const,
    backgroundColor: "var(--bg-base)",
    // Prevent the panel from growing taller than the Tauri window height.
    maxHeight: "520px",
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
    fontFamily: "var(--font-ui)",
    lineHeight: 1,
    transition: "color 120ms ease-out, background-color 120ms ease-out",
  },
  errorDot: {
    fontSize: "12px",
    fontWeight: 700,
    color: "var(--danger)",
    marginRight: "4px",
  },
} as const;
