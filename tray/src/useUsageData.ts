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

async function loadApiToken(): Promise<string> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<string>("get_api_token");
  } catch {
    return ""; // web preview / dev mode — no auth required
  }
}

function authHeaders(token: string): HeadersInit {
  return token ? { Authorization: `Bearer ${token}` } : {};
}

async function fetchSummary(range: "today" | "month", token: string): Promise<SummaryResponse> {
  const resp = await fetch(`${API_BASE}/v1/summary?range=${range}`, {
    headers: authHeaders(token),
  });
  if (!resp.ok) {
    throw new Error(`summary?range=${range} returned ${resp.status}`);
  }
  return resp.json() as Promise<SummaryResponse>;
}

async function fetchActiveSessions(token: string): Promise<Session | null> {
  const resp = await fetch(`${API_BASE}/v1/sessions?active=true&limit=1`, {
    headers: authHeaders(token),
  });
  if (!resp.ok) {
    throw new Error(`sessions?active=true returned ${resp.status}`);
  }
  const body = (await resp.json()) as SessionsResponse;
  return body.sessions[0] ?? null;
}

async function fetchHeatmap(token: string): Promise<HeatmapDay[]> {
  const resp = await fetch(`${API_BASE}/v1/heatmap`, {
    headers: authHeaders(token),
  });
  if (!resp.ok) {
    throw new Error(`heatmap returned ${resp.status}`);
  }
  return resp.json() as Promise<HeatmapDay[]>;
}

export function useUsageData(): UsageData {
  const [token, setToken] = useState<string | null>(null);
  const [data, setData] = useState<UsageData>({
    today: null,
    month: null,
    activeSession: null,
    activeBlock: null,
    heatmap: [],
    loading: true,
    error: null,
  });

  // Load the API token from Tauri config once on mount.
  useEffect(() => {
    loadApiToken().then(setToken);
  }, []);

  const refresh = useCallback(async () => {
    const t = token ?? "";
    try {
      const [today, month, activeSession, heatmap] = await Promise.all([
        fetchSummary("today", t),
        fetchSummary("month", t),
        fetchActiveSessions(t),
        fetchHeatmap(t),
      ]);
      const activeBlock: ActiveBlock | null = today.active_block ?? null;
      setData({ today, month, activeSession, activeBlock, heatmap, loading: false, error: null });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setData((prev) => ({ ...prev, loading: false, error: message }));
    }
  }, [token]);

  // Start polling only after the token has been loaded.
  useEffect(() => {
    if (token === null) return;
    void refresh();
    const id = setInterval(() => void refresh(), POLL_INTERVAL_MS);
    return () => clearInterval(id);
  }, [refresh, token]);

  return data;
}
