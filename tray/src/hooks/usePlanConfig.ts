import { useState, useEffect, useCallback } from "react";
import type { PlanConfig, PlanKind } from "../types";

const DEFAULT_PLAN: PlanConfig = { kind: "max5", custom_token_limit: null, usage_poll_interval_secs: 60 };

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
        kind,
        custom_token_limit: customTokenLimit ?? null,
        usage_poll_interval_secs: pollIntervalSecs ?? plan.usage_poll_interval_secs,
      };
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("set_plan_config", {
          kind,
          custom_token_limit: customTokenLimit ?? null,
          usage_poll_interval_secs: next.usage_poll_interval_secs,
        });
        setPlanState(next);
      } catch {
        setPlanState(next);
      }
    },
    [plan.usage_poll_interval_secs],
  );

  return { plan, setPlan };
}
