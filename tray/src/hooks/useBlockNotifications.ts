import { useEffect, useRef } from "react";
import type { ActiveBlock } from "../types";
import { formatCost } from "../components/format";

async function sendTauriNotification(title: string, body: string) {
  try {
    const { isPermissionGranted, requestPermission, sendNotification } =
      await import("@tauri-apps/plugin-notification");
    let ok = await isPermissionGranted();
    if (!ok) {
      ok = (await requestPermission()) === "granted";
    }
    if (ok) {
      sendNotification({ title, body });
    }
  } catch {
    // Tauri runtime not available — silently skip.
  }
}

/**
 * Fires Windows notifications when the active billing block crosses 80% or 100%.
 * Thresholds reset automatically when a new block starts.
 * Fires only when crossing from below — not when opening the tray mid-block.
 */
export function useBlockNotifications(block: ActiveBlock | null): void {
  const lastStart = useRef<string | null>(null);
  const fired = useRef(new Set<number>());
  const prevPct = useRef<number>(0);

  useEffect(() => {
    if (!block) return;

    if (block.start !== lastStart.current) {
      // New block: establish baseline, do not fire immediately.
      lastStart.current = block.start;
      fired.current = new Set();
      prevPct.current = block.percent_elapsed;
      return;
    }

    const prev = prevPct.current;
    const pct = block.percent_elapsed;
    prevPct.current = pct;

    if (prev < 80 && pct >= 80 && !fired.current.has(80)) {
      fired.current.add(80);
      void sendTauriNotification(
        "WinUsage — Block at 80%",
        `${formatCost(block.cost_usd)} used in this 5-hour billing block.`,
      );
    }

    if (prev < 100 && pct >= 100 && !fired.current.has(100)) {
      fired.current.add(100);
      void sendTauriNotification(
        "WinUsage — Block complete",
        `Billing window closed. Total: ${formatCost(block.cost_usd)}.`,
      );
    }
  }, [block]);
}
