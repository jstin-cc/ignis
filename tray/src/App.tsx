import { useState } from "react";
import { useUsageData } from "./useUsageData";
import { useBlockNotifications } from "./hooks/useBlockNotifications";
import { useAutoStart } from "./hooks/useAutoStart";
import { useUpdater } from "./hooks/useUpdater";
import { usePlanConfig } from "./hooks/usePlanConfig";
import { useAnthropicUsage } from "./hooks/useAnthropicUsage";
import type { PlanKind } from "./types";
import { TodayPanel } from "./components/TodayPanel";
import { MonthPanel } from "./components/MonthPanel";
import { BlockPanel } from "./components/BlockPanel";
import { ProjectsPanel } from "./components/ProjectsPanel";
import { HeatmapPanel } from "./components/HeatmapPanel";
import { ActiveSessionPanel } from "./components/ActiveSessionPanel";
import { Footer } from "./components/Footer";

export function App() {
  const { today, month, activeSession, activeBlock, heatmap, error } = useUsageData();
  useBlockNotifications(activeBlock);
  const { isEnabled, toggle } = useAutoStart();
  const { checking, result, error: updateError, checkForUpdate } = useUpdater();
  const { plan, setPlan } = usePlanConfig();
  const { usage: anthropicUsage, error: usageError } = useAnthropicUsage();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [customLimitInput, setCustomLimitInput] = useState<string>("");

  function handleOpenDashboard() {
    // Phase 2: open a full dashboard window via Tauri IPC.
    // In MVP this is a no-op placeholder.
  }

  return (
    <div style={styles.shell}>
      <header style={styles.header} data-tauri-drag-region>
        <span style={styles.appName} data-tauri-drag-region>
          WinUsage
        </span>
        <div style={styles.headerActions}>
          <button
            style={styles.iconBtn}
            aria-label="Settings"
            onClick={() => setSettingsOpen((v) => !v)}
          >
            ⚙
          </button>
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

      <div style={styles.content}>
        {settingsOpen && (
          <div style={styles.settingsPanel}>
            <label style={styles.settingsRow}>
              <input
                type="checkbox"
                checked={isEnabled}
                onChange={() => void toggle()}
                style={styles.checkbox}
              />
              <span style={styles.settingsLabel}>Auto-Start bei Windows-Login</span>
            </label>

            <div style={styles.planRow}>
              <span style={styles.settingsLabel}>Plan</span>
              <select
                style={styles.planSelect}
                value={plan.kind}
                onChange={(e) => {
                  const kind = e.target.value as PlanKind;
                  if (kind !== "custom") {
                    void setPlan(kind);
                  } else {
                    setCustomLimitInput(String(plan.custom_token_limit ?? 88000));
                    void setPlan(kind, plan.custom_token_limit ?? 88000);
                  }
                }}
              >
                <option value="pro">Pro (44k tokens)</option>
                <option value="max5">Max 5× (88k tokens)</option>
                <option value="max20">Max 20× (220k tokens)</option>
                <option value="custom">Custom</option>
              </select>
            </div>

            {plan.kind === "custom" && (
              <div style={styles.planRow}>
                <span style={styles.settingsLabel}>Token-Limit</span>
                <input
                  type="number"
                  style={styles.customInput}
                  value={customLimitInput}
                  min={1000}
                  step={1000}
                  onChange={(e) => setCustomLimitInput(e.target.value)}
                  onBlur={() => {
                    const limit = parseInt(customLimitInput, 10);
                    if (!isNaN(limit) && limit > 0) {
                      void setPlan("custom", limit);
                    }
                  }}
                />
              </div>
            )}

            <div style={styles.updateRow}>
              <button
                style={styles.updateBtn}
                disabled={checking}
                onClick={() => void checkForUpdate()}
              >
                {checking ? "Prüfe…" : "Updates prüfen"}
              </button>
              {result && (
                <span style={styles.updateStatus}>
                  {result.available ? `v${result.version} verfügbar` : "Aktuell"}
                </span>
              )}
              {updateError && (
                <span style={{ ...styles.updateStatus, color: "var(--text-tertiary)" }}>
                  kein Server
                </span>
              )}
            </div>
          </div>
        )}

        {error && (
          <div style={styles.errorBanner}>
            API nicht erreichbar — starte winusage-api
          </div>
        )}
        <hr className="section-divider" />
        <TodayPanel data={today} />
        <hr className="section-divider" />
        <MonthPanel data={month} />
        <hr className="section-divider" />
        <BlockPanel block={activeBlock} usage={anthropicUsage} usageError={usageError} />
        {today?.by_project.length ? (
          <>
            <hr className="section-divider" />
            <ProjectsPanel data={today} />
          </>
        ) : null}
        {heatmap.length > 0 && (
          <>
            <hr className="section-divider" />
            <HeatmapPanel days={heatmap} />
          </>
        )}
        <hr className="section-divider" />
        <ActiveSessionPanel session={activeSession} />
      </div>

      <Footer onOpenDashboard={handleOpenDashboard} />
    </div>
  );
}

const styles = {
  shell: {
    width: "360px",
    height: "520px",
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
    overflowY: "auto" as const,
    display: "flex",
    flexDirection: "column" as const,
    minHeight: 0,
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
  settingsPanel: {
    padding: "10px 16px",
    backgroundColor: "var(--bg-surface)",
    borderBottom: "1px solid var(--border-subtle)",
  },
  settingsRow: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
    cursor: "pointer",
  },
  checkbox: {
    accentColor: "var(--accent)",
    width: "14px",
    height: "14px",
    cursor: "pointer",
  },
  settingsLabel: {
    fontSize: "13px",
    color: "var(--text-secondary)",
  },
  planRow: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
    marginTop: "8px",
  },
  planSelect: {
    flex: 1,
    fontSize: "12px",
    padding: "3px 6px",
    backgroundColor: "var(--bg-elevated)",
    border: "1px solid var(--border-subtle)",
    borderRadius: "4px",
    color: "var(--text-secondary)",
    cursor: "pointer",
    fontFamily: "var(--font-ui)",
  },
  customInput: {
    flex: 1,
    fontSize: "12px",
    padding: "3px 6px",
    backgroundColor: "var(--bg-elevated)",
    border: "1px solid var(--border-subtle)",
    borderRadius: "4px",
    color: "var(--text-secondary)",
    fontFamily: "var(--font-ui)",
    width: "80px",
  },
  updateRow: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
    marginTop: "8px",
  },
  updateBtn: {
    fontSize: "12px",
    padding: "3px 8px",
    backgroundColor: "var(--bg-elevated)",
    border: "1px solid var(--border-subtle)",
    borderRadius: "4px",
    color: "var(--text-secondary)",
    cursor: "pointer",
    fontFamily: "var(--font-ui)",
  },
  updateStatus: {
    fontSize: "12px",
    color: "var(--accent)",
  },
  errorBanner: {
    padding: "8px 16px",
    fontSize: "12px",
    color: "var(--danger)",
    backgroundColor: "var(--bg-surface)",
    borderBottom: "1px solid var(--border-subtle)",
  },
} as const;
