import { useState, useEffect, useCallback } from "react";
import type { PlanConfig, PlanKind } from "../types";

const DEFAULT_PLAN: PlanConfig = { kind: "max5", custom_token_limit: null };

export function usePlanConfig() {
  const [plan, setPlanState] = useState<PlanConfig>(DEFAULT_PLAN);

  useEffect(() => {
    import("@tauri-apps/api/core")
      .then(({ invoke }) => invoke<PlanConfig>("get_plan_config"))
      .then((cfg) => setPlanState(cfg))
      .catch(() => {/* dev/web mode – keep default */});
  }, []);

  const setPlan = useCallback(
    async (kind: PlanKind, customTokenLimit?: number) => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("set_plan_config", {
          kind,
          custom_token_limit: customTokenLimit ?? null,
        });
        setPlanState({ kind, custom_token_limit: customTokenLimit ?? null });
      } catch {
        // dev/web mode – update local state only
        setPlanState({ kind, custom_token_limit: customTokenLimit ?? null });
      }
    },
    [],
  );

  return { plan, setPlan };
}
