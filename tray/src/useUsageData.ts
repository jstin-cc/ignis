import { useState, useEffect, useCallback } from "react";
import type {
  UsageData,
  SummaryResponse,
  SessionsResponse,
  Session,
  ActiveBlock,
  HeatmapDay,
} from "./types";

const API_BASE = "http://127.0.0.1:7337";
const POLL_INTERVAL_MS = 30_000;

async function fetchSummary(range: "today" | "month"): Promise<SummaryResponse> {
  const resp = await fetch(`${API_BASE}/v1/summary?range=${range}`);
  if (!resp.ok) {
    throw new Error(`summary?range=${range} returned ${resp.status}`);
  }
  return resp.json() as Promise<SummaryResponse>;
}

async function fetchActiveSessions(): Promise<Session | null> {
  const resp = await fetch(`${API_BASE}/v1/sessions?active=true&limit=1`);
  if (!resp.ok) {
    throw new Error(`sessions?active=true returned ${resp.status}`);
  }
  const body = (await resp.json()) as SessionsResponse;
  return body.sessions[0] ?? null;
}

async function fetchHeatmap(): Promise<HeatmapDay[]> {
  const resp = await fetch(`${API_BASE}/v1/heatmap`);
  if (!resp.ok) {
    throw new Error(`heatmap returned ${resp.status}`);
  }
  return resp.json() as Promise<HeatmapDay[]>;
}

export function useUsageData(): UsageData {
  const [data, setData] = useState<UsageData>({
    today: null,
    month: null,
    activeSession: null,
    activeBlock: null,
    heatmap: [],
    loading: true,
    error: null,
  });

  const refresh = useCallback(async () => {
    try {
      const [today, month, activeSession, heatmap] = await Promise.all([
        fetchSummary("today"),
        fetchSummary("month"),
        fetchActiveSessions(),
        fetchHeatmap(),
      ]);
      const activeBlock: ActiveBlock | null = today.active_block ?? null;
      setData({ today, month, activeSession, activeBlock, heatmap, loading: false, error: null });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setData((prev) => ({ ...prev, loading: false, error: message }));
    }
  }, []);

  useEffect(() => {
    void refresh();
    const id = setInterval(() => void refresh(), POLL_INTERVAL_MS);
    return () => clearInterval(id);
  }, [refresh]);

  return data;
}
