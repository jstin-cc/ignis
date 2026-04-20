import { useEffect, useRef } from "react";
import type { ActiveBlock } from "../types";
import { formatCost } from "../components/format";

async function sendTauriNotification(title: string, body: string) {
  try {
    // Dynamic import: app stays functional outside Tauri (dev browser, tests).
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
 */
export function useBlockNotifications(block: ActiveBlock | null): void {
  const lastStart = useRef<string | null>(null);
  const fired = useRef(new Set<number>());

  useEffect(() => {
    if (!block) return;

    if (block.start !== lastStart.current) {
      lastStart.current = block.start;
      fired.current = new Set();
    }

    const pct = block.percent_elapsed;

    if (pct >= 80 && !fired.current.has(80)) {
      fired.current.add(80);
      void sendTauriNotification(
        "WinUsage — Block at 80%",
        `${formatCost(block.cost_usd)} used in this 5-hour billing block.`,
      );
    }

    if (pct >= 100 && !fired.current.has(100)) {
      fired.current.add(100);
      void sendTauriNotification(
        "WinUsage — Block complete",
        `Billing window closed. Total: ${formatCost(block.cost_usd)}.`,
      );
    }
  }, [block]);
}
