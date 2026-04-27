import { useEffect, useState } from "react";
import type { PlanConfig, PlanKind } from "../types";

interface SettingsTabProps {
  autoStart: { isEnabled: boolean; toggle: () => Promise<void> };
  plan: PlanConfig;
  setPlan: (kind: PlanKind, customTokenLimit?: number, pollIntervalSecs?: number) => Promise<void>;
  setThresholds: (thresholds: number[]) => Promise<void>;
  setBudgets: (weekly: number | null, monthly: number | null) => Promise<void>;
  updater: {
    checking: boolean;
    installing: boolean;
    result: { available: boolean; version: string; body: string | null } | null;
    error: string | null;
    checkForUpdate: () => Promise<void>;
    installUpdate: () => Promise<void>;
  };
}

const FIXED_THRESHOLDS = [50, 75, 90, 100];

export function SettingsTab({ autoStart, plan, setPlan, setThresholds, setBudgets, updater }: SettingsTabProps) {
  const [customLimitInput, setCustomLimitInput] = useState<string>(
    String(plan.custom_token_limit ?? 88000),
  );
  const [weeklyBudgetInput, setWeeklyBudgetInput] = useState<string>(
    plan.weekly_budget_usd != null ? String(plan.weekly_budget_usd) : "",
  );
  const [monthlyBudgetInput, setMonthlyBudgetInput] = useState<string>(
    plan.monthly_budget_usd != null ? String(plan.monthly_budget_usd) : "",
  );
  const [apiToken, setApiToken] = useState<string>("");
  const [tokenCopied, setTokenCopied] = useState(false);

  useEffect(() => {
    void import("@tauri-apps/api/core")
      .then(({ invoke }) => invoke<string>("get_api_token"))
      .then((t) => setApiToken(t))
      .catch(() => setApiToken(""));
  }, []);

  useEffect(() => {
    setCustomLimitInput(String(plan.custom_token_limit ?? 88000));
  }, [plan.custom_token_limit]);

  useEffect(() => {
    setWeeklyBudgetInput(plan.weekly_budget_usd != null ? String(plan.weekly_budget_usd) : "");
    setMonthlyBudgetInput(plan.monthly_budget_usd != null ? String(plan.monthly_budget_usd) : "");
  }, [plan.weekly_budget_usd, plan.monthly_budget_usd]);

  const toggleThreshold = (t: number) => {
    const active = plan.block_alert_thresholds;
    const next = active.includes(t) ? active.filter((x) => x !== t) : [...active, t];
    void setThresholds(next);
  };

  const commitBudgets = () => {
    const weekly = parseFloat(weeklyBudgetInput);
    const monthly = parseFloat(monthlyBudgetInput);
    void setBudgets(
      isFinite(weekly) && weekly > 0 ? weekly : null,
      isFinite(monthly) && monthly > 0 ? monthly : null,
    );
  };

  const copyToken = async () => {
    if (!apiToken) return;
    try {
      await navigator.clipboard.writeText(apiToken);
      setTokenCopied(true);
      setTimeout(() => setTokenCopied(false), 1500);
    } catch {
      // ignore — clipboard not available in some contexts
    }
  };

  return (
    <div style={styles.panel}>
      {/* Auto-Start */}
      <section style={styles.section}>
        <div className="section-label" style={styles.sectionLabel}>Allgemein</div>
        <label
          style={styles.row}
          title="Ignis und ignis-api starten automatisch wenn du dich anmeldest — kein manueller Start nötig."
        >
          <input
            type="checkbox"
            checked={autoStart.isEnabled}
            onChange={() => void autoStart.toggle()}
            style={styles.checkbox}
          />
          <span style={styles.label}>Auto-Start bei Windows-Login</span>
        </label>
      </section>

      {/* Plan */}
      <section style={styles.section}>
        <div className="section-label" style={styles.sectionLabel}>Plan</div>
        <div
          style={styles.row}
          title="Pro: 44k tok · Max 5×: 88k tok · Max 20×: 220k tok pro 5-Stunden-Block. Custom: eigenes Limit."
        >
          <span style={styles.label}>Modell</span>
          <select
            style={styles.select}
            value={plan.kind}
            onChange={(e) => {
              const kind = e.target.value as PlanKind;
              if (kind !== "custom") {
                void setPlan(kind);
              } else {
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
          <div style={styles.row}>
            <span style={styles.label}>Token-Limit</span>
            <input
              type="number"
              style={styles.input}
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

        <div
          style={styles.row}
          title="Wie oft die Nutzungsdaten der Anthropic-API abgerufen werden. Kürzere Intervalle verbrauchen mehr Netzwerk."
        >
          <span style={styles.label}>Aktualisierung</span>
          <select
            style={styles.select}
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
      </section>

      {/* Benachrichtigungen */}
      <section style={styles.section}>
        <div className="section-label" style={styles.sectionLabel}>Benachrichtigungen</div>
        {FIXED_THRESHOLDS.map((t) => (
          <label
            key={t}
            style={styles.row}
            title={
              t < 100
                ? `Windows-Notification wenn ${t}% des Plan-Token-Limits im aktiven 5h-Block verbraucht sind.`
                : "Windows-Notification wenn der 5h-Abrechnungsblock vollständig abgeschlossen ist."
            }
          >
            <input
              type="checkbox"
              checked={plan.block_alert_thresholds.includes(t)}
              onChange={() => toggleThreshold(t)}
              style={styles.checkbox}
            />
            <span style={styles.label}>
              {t < 100 ? `Block bei ${t}%` : "Block abgeschlossen (100%)"}
            </span>
          </label>
        ))}
      </section>

      {/* Budget */}
      <section style={styles.section}>
        <div className="section-label" style={styles.sectionLabel}>Budget</div>
        <div
          style={styles.row}
          title="Notification wenn die Wochenkosten diesen USD-Betrag überschreiten. Leer = deaktiviert."
        >
          <span style={styles.label}>Woche (USD)</span>
          <input
            type="number"
            style={styles.input}
            value={weeklyBudgetInput}
            placeholder="—"
            min={0}
            step={1}
            onChange={(e) => setWeeklyBudgetInput(e.target.value)}
            onBlur={commitBudgets}
          />
        </div>
        <div
          style={styles.row}
          title="Notification wenn die Monatskosten diesen USD-Betrag überschreiten. Leer = deaktiviert."
        >
          <span style={styles.label}>Monat (USD)</span>
          <input
            type="number"
            style={styles.input}
            value={monthlyBudgetInput}
            placeholder="—"
            min={0}
            step={1}
            onChange={(e) => setMonthlyBudgetInput(e.target.value)}
            onBlur={commitBudgets}
          />
        </div>
        <p style={styles.hint}>Leer lassen um kein Budget zu setzen.</p>
      </section>

      {/* Updates */}
      <section style={styles.section}>
        <div className="section-label" style={styles.sectionLabel}>Updates</div>
        <div style={styles.row}>
          <button
            className="btn btn--secondary"
            style={styles.btn}
            disabled={updater.checking || updater.installing}
            onClick={() => void updater.checkForUpdate()}
          >
            {updater.checking ? "Prüfe…" : "Updates prüfen"}
          </button>
          {updater.result && (
            <span style={updater.result.available ? styles.statusAccent : styles.statusMuted}>
              {updater.result.available ? `v${updater.result.version} verfügbar` : "Aktuell"}
            </span>
          )}
          {updater.error && (
            <span style={styles.statusMuted}>kein Server</span>
          )}
        </div>
        {updater.result?.available && updater.result.body && (
          <pre style={styles.releaseNotes}>{updater.result.body}</pre>
        )}
        {updater.result?.available && (
          <button
            className="btn btn--primary"
            style={styles.btnInstall}
            disabled={updater.installing}
            onClick={() => void updater.installUpdate()}
          >
            {updater.installing ? "Installiere…" : "Installieren & Neu starten"}
          </button>
        )}
      </section>

      {/* API-Token */}
      <section style={styles.section}>
        <div className="section-label" style={styles.sectionLabel}>API-Token</div>
        <div style={styles.row}>
          <code style={styles.tokenCode}>
            {apiToken ? maskToken(apiToken) : "—"}
          </code>
          <button
            className="btn btn--secondary"
            style={styles.btn}
            disabled={!apiToken}
            onClick={() => void copyToken()}
          >
            {tokenCopied ? "Kopiert" : "Kopieren"}
          </button>
        </div>
        <p style={styles.hint}>
          Read-only. Für CLI/curl-Zugriff auf 127.0.0.1:7337.
        </p>
      </section>
    </div>
  );
}

function maskToken(token: string): string {
  if (token.length <= 8) return token;
  return `${token.slice(0, 4)}…${token.slice(-4)}`;
}

const styles = {
  panel: {
    padding: "8px 16px 16px",
    display: "flex",
    flexDirection: "column" as const,
    gap: "16px",
  },
  section: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "6px",
  },
  sectionLabel: {
    marginBottom: "2px",
  },
  row: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
    minHeight: "24px",
  },
  label: {
    fontSize: "13px",
    color: "var(--text-secondary)",
    flex: 1,
  },
  checkbox: {
    accentColor: "var(--accent)",
    width: "14px",
    height: "14px",
    cursor: "pointer",
  },
  select: {
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
  input: {
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
  btn: {
    fontSize: "12px",
    padding: "3px 8px",
    fontFamily: "var(--font-sans)",
  },
  statusAccent: {
    fontSize: "12px",
    color: "var(--accent)",
  },
  statusMuted: {
    fontSize: "12px",
    color: "var(--text-muted)",
  },
  tokenCode: {
    flex: 1,
    fontFamily: "var(--font-mono)",
    fontSize: "12px",
    color: "var(--text-secondary)",
    backgroundColor: "var(--bg-elevated)",
    padding: "4px 8px",
    borderRadius: "4px",
    border: "1px solid var(--border-subtle)",
    letterSpacing: "0.02em",
  },
  hint: {
    fontSize: "11px",
    color: "var(--text-muted)",
    margin: "4px 0 0",
    lineHeight: 1.4,
  },
  releaseNotes: {
    fontSize: "11px",
    color: "var(--text-secondary)",
    backgroundColor: "var(--bg-elevated)",
    border: "1px solid var(--border-subtle)",
    borderRadius: "4px",
    padding: "6px 8px",
    margin: "2px 0 0",
    whiteSpace: "pre-wrap" as const,
    wordBreak: "break-word" as const,
    fontFamily: "var(--font-sans)",
    lineHeight: 1.5,
    maxHeight: "120px",
    overflowY: "auto" as const,
  },
  btnInstall: {
    marginTop: "2px",
    fontSize: "12px",
    padding: "4px 12px",
    fontFamily: "var(--font-sans)",
    backgroundColor: "var(--accent)",
    color: "#fff",
    border: "none",
    borderRadius: "5px",
    cursor: "pointer",
    fontWeight: 600,
    alignSelf: "flex-start" as const,
  },
} as const;
