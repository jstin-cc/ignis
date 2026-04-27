import { useState, useEffect, useCallback } from "react";
import type { PlanConfig, PlanKind } from "../types";

const DEFAULT_PLAN: PlanConfig = {
  kind: "max5",
  custom_token_limit: null,
  usage_poll_interval_secs: 60,
  block_alert_thresholds: [50, 75, 90, 100],
  weekly_budget_usd: null,
  monthly_budget_usd: null,
};

export function usePlanConfig() {
  const [plan, setPlanState] = useState<PlanConfig>(DEFAULT_PLAN);

  useEffect(() => {
    import("@tauri-apps/api/core")
      .then(({ invoke }) => invoke<PlanConfig>("get_plan_config"))
      .then((cfg) => setPlanState(cfg))
      .catch(() => {/* dev/web mode – keep default */});
  }, []);

  const setPlan = useCallback(
    async (kind: PlanKind, customTokenLimit?: number, pollIntervalSecs?: number) => {
      const next: PlanConfig = {
        ...plan,
        kind,
        custom_token_limit: customTokenLimit ?? null,
        usage_poll_interval_secs: pollIntervalSecs ?? plan.usage_poll_interval_secs,
      };
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("set_plan_config", {
          kind,
          customTokenLimit: customTokenLimit ?? null,
          usagePollIntervalSecs: next.usage_poll_interval_secs,
        });
        setPlanState(next);
      } catch {
        setPlanState(next);
      }
    },
    [plan],
  );

  const setThresholds = useCallback(
    async (thresholds: number[]) => {
      const sorted = [...new Set(thresholds)].sort((a, b) => a - b);
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("set_alert_thresholds", { thresholds: sorted });
      } catch {
        // dev mode — ignore
      }
      setPlanState((prev) => ({ ...prev, block_alert_thresholds: sorted }));
    },
    [],
  );

  const setBudgets = useCallback(
    async (weeklyUsd: number | null, monthlyUsd: number | null) => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("set_budget_caps", {
          weeklyUsd,
          monthlyUsd,
        });
      } catch {
        // dev mode — ignore
      }
      setPlanState((prev) => ({
        ...prev,
        weekly_budget_usd: weeklyUsd,
        monthly_budget_usd: monthlyUsd,
      }));
    },
    [],
  );

  return { plan, setPlan, setThresholds, setBudgets };
}
