import { useState } from "react";
import { useUsageData } from "./useUsageData";
import { useBlockNotifications } from "./hooks/useBlockNotifications";
import { useAutoStart } from "./hooks/useAutoStart";
import { useUpdater } from "./hooks/useUpdater";
import { usePlanConfig } from "./hooks/usePlanConfig";
import { useAnthropicUsage } from "./hooks/useAnthropicUsage";
import type { PlanKind } from "./types";
import { TabBar } from "./components/TabBar";
import type { TabId } from "./components/TabBar";
import { TodaySection } from "./components/TodayPanel";
import { MonthPanel, WeekSection } from "./components/MonthPanel";
import { BlockPanel } from "./components/BlockPanel";
import { ProjectsPanel } from "./components/ProjectsPanel";
import { HeatmapPanel } from "./components/HeatmapPanel";
import { SessionSection } from "./components/ActiveSessionPanel";
import { Footer } from "./components/Footer";

export function App() {
  const { today, month, activeSession, activeBlock, heatmap, error } = useUsageData();
  useBlockNotifications(activeBlock);
  const { isEnabled, toggle } = useAutoStart();
  const { checking, result, error: updateError, checkForUpdate } = useUpdater();
  const { plan, setPlan } = usePlanConfig();
  const { usage: anthropicUsage, error: usageError } = useAnthropicUsage(plan.usage_poll_interval_secs);
  const [activeTab, setActiveTab] = useState<TabId>('today');
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [customLimitInput, setCustomLimitInput] = useState<string>("");

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

      {/* TabBar — 36px */}
      <TabBar active={activeTab} onChange={setActiveTab} />

      {/* Content — 380px, kein Scroll */}
      <div style={styles.content}>
        {/* Settings-Overlay */}
        {settingsOpen && (
          <div style={styles.settingsOverlay}>
            <div style={styles.settingsPanel}>
              <div style={styles.settingsHeader}>
                <span style={styles.settingsTitle}>Settings</span>
                <button
                  style={styles.iconBtn}
                  aria-label="Close settings"
                  onClick={() => setSettingsOpen(false)}
                >
                  ×
                </button>
              </div>

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

              <div style={styles.planRow}>
                <span style={styles.settingsLabel}>Aktualisierung</span>
                <select
                  style={styles.planSelect}
                  value={plan.usage_poll_interval_secs}
                  onChange={(e) => {
                    const secs = parseInt(e.target.value, 10);
                    void setPlan(plan.kind, plan.custom_token_limit ?? undefined, secs);
                  }}
                >
                  <option value={30}>30 Sekunden</option>
                  <option value={60}>1 Minute</option>
                  <option value={120}>2 Minuten</option>
                  <option value={300}>5 Minuten</option>
                  <option value={600}>10 Minuten</option>
                </select>
              </div>

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
                  <span style={{ ...styles.updateStatus, color: "var(--text-muted)" }}>
                    kein Server
                  </span>
                )}
              </div>
            </div>
          </div>
        )}

        {error && (
          <div style={styles.errorBanner}>
            API nicht erreichbar — starte ignis-api
          </div>
        )}

        {activeTab === 'today' && (
          <>
            <TodaySection data={today} />
            <hr className="section-divider" />
            <WeekSection data={month} />
            <hr className="section-divider" />
            <BlockPanel block={activeBlock} usage={anthropicUsage} usageError={usageError} />
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
          <HeatmapPanel days={heatmap} />
        )}
      </div>

      {/* Footer — 56px */}
      <Footer onOpenDashboard={() => {}} />
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
  settingsOverlay: {
    position: "absolute" as const,
    top: 0,
    left: 0,
    width: "100%",
    height: "100%",
    background: "var(--bg-overlay)",
    zIndex: 10,
    overflowY: "auto" as const,
  },
  settingsPanel: {
    padding: "12px 16px",
    display: "flex",
    flexDirection: "column" as const,
    gap: "4px",
  },
  settingsHeader: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    marginBottom: "8px",
  },
  settingsTitle: {
    fontSize: "13px",
    fontWeight: 600,
    color: "var(--text-primary)",
    textTransform: "uppercase" as const,
    letterSpacing: "0.06em",
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
    fontFamily: "var(--font-sans)",
  },
  customInput: {
    flex: 1,
    fontSize: "12px",
    padding: "3px 6px",
    backgroundColor: "var(--bg-elevated)",
    border: "1px solid var(--border-subtle)",
    borderRadius: "4px",
    color: "var(--text-secondary)",
    fontFamily: "var(--font-sans)",
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
    fontFamily: "var(--font-sans)",
  },
  updateStatus: {
    fontSize: "12px",
    color: "var(--accent)",
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
