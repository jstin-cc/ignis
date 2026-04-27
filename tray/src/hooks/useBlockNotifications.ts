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
 * Fires Windows notifications when the active billing block crosses configurable
 * token-% thresholds. Also notifies when weekly or monthly spending exceeds budgets.
 *
 * Thresholds reset automatically when a new block starts.
 * Fires only when crossing from below — not when opening the tray mid-block.
 */
export function useBlockNotifications(
  block: ActiveBlock | null,
  thresholds: number[],
  weekCostUsd?: string | null,
  weekBudgetUsd?: number | null,
  monthCostUsd?: string | null,
  monthBudgetUsd?: number | null,
): void {
  const lastStart = useRef<string | null>(null);
  const fired = useRef(new Set<number>());
  const prevPct = useRef<number>(0);

  const prevWeekCost = useRef<number | null>(null);
  const prevMonthCost = useRef<number | null>(null);

  // Block threshold notifications.
  useEffect(() => {
    if (!block) return;

    if (block.start !== lastStart.current) {
      // New block: establish baseline, do not fire immediately.
      lastStart.current = block.start;
      fired.current = new Set();
      prevPct.current = block.block_token_pct;
      return;
    }

    const prev = prevPct.current;
    const pct = block.block_token_pct;
    prevPct.current = pct;

    const sorted = [...thresholds].sort((a, b) => a - b);
    for (const t of sorted) {
      if (prev < t && pct >= t && !fired.current.has(t)) {
        fired.current.add(t);
        if (t >= 100) {
          void sendTauriNotification(
            "Ignis — Block complete",
            `Billing window closed. Total: ${formatCost(block.cost_usd)}.`,
          );
        } else {
          void sendTauriNotification(
            `Ignis — Block at ${t}%`,
            `${formatCost(block.cost_usd)} used in this 5-hour billing block.`,
          );
        }
      }
    }
  }, [block, thresholds]);

  // Weekly budget notification.
  useEffect(() => {
    if (weekCostUsd == null || weekBudgetUsd == null) return;
    const cost = parseFloat(weekCostUsd);
    if (!isFinite(cost)) return;

    if (prevWeekCost.current === null) {
      prevWeekCost.current = cost;
      return;
    }

    const prev = prevWeekCost.current;
    prevWeekCost.current = cost;
    if (prev <= weekBudgetUsd && cost > weekBudgetUsd) {
      void sendTauriNotification(
        "Ignis — Wochenbudget überschritten",
        `${formatCost(weekCostUsd)} Wochenkosten übersteigen dein Budget von $${weekBudgetUsd.toFixed(2)}.`,
      );
    }
  }, [weekCostUsd, weekBudgetUsd]);

  // Monthly budget notification.
  useEffect(() => {
    if (monthCostUsd == null || monthBudgetUsd == null) return;
    const cost = parseFloat(monthCostUsd);
    if (!isFinite(cost)) return;

    if (prevMonthCost.current === null) {
      prevMonthCost.current = cost;
      return;
    }

    const prev = prevMonthCost.current;
    prevMonthCost.current = cost;
    if (prev <= monthBudgetUsd && cost > monthBudgetUsd) {
      void sendTauriNotification(
        "Ignis — Monatsbudget überschritten",
        `${formatCost(monthCostUsd)} Monatskosten übersteigen dein Budget von $${monthBudgetUsd.toFixed(2)}.`,
      );
    }
  }, [monthCostUsd, monthBudgetUsd]);
}
