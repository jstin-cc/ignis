import { useRef, useState, useEffect, useCallback } from "react";
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

async function fetchSummary(
  range: "today" | "month",
  token: string,
  signal: AbortSignal,
): Promise<SummaryResponse> {
  const resp = await fetch(`${API_BASE}/v1/summary?range=${range}`, {
    headers: authHeaders(token),
    signal,
  });
  if (!resp.ok) {
    throw new Error(`summary?range=${range} returned ${resp.status}`);
  }
  return resp.json() as Promise<SummaryResponse>;
}

async function fetchActiveSessions(
  token: string,
  signal: AbortSignal,
): Promise<Session | null> {
  const resp = await fetch(`${API_BASE}/v1/sessions?active=true&limit=1`, {
    headers: authHeaders(token),
    signal,
  });
  if (!resp.ok) {
    throw new Error(`sessions?active=true returned ${resp.status}`);
  }
  const body = (await resp.json()) as SessionsResponse;
  return body.sessions[0] ?? null;
}

async function fetchHeatmap(token: string, signal: AbortSignal): Promise<HeatmapDay[]> {
  const resp = await fetch(`${API_BASE}/v1/heatmap`, {
    headers: authHeaders(token),
    signal,
  });
  if (!resp.ok) {
    throw new Error(`heatmap returned ${resp.status}`);
  }
  return resp.json() as Promise<HeatmapDay[]>;
}

export function useUsageData(): UsageData {
  const [token, setToken] = useState<string | null>(null);
  const abortRef = useRef<AbortController | null>(null);
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
    // Abort any in-flight request from a previous run.
    abortRef.current?.abort();
    const controller = new AbortController();
    abortRef.current = controller;
    const { signal } = controller;

    const t = token ?? "";
    try {
      const [today, month, activeSession, heatmap] = await Promise.all([
        fetchSummary("today", t, signal),
        fetchSummary("month", t, signal),
        fetchActiveSessions(t, signal),
        fetchHeatmap(t, signal),
      ]);
      if (signal.aborted) return;
      const activeBlock: ActiveBlock | null = today.active_block ?? null;
      setData({ today, month, activeSession, activeBlock, heatmap, loading: false, error: null });
    } catch (err) {
      if (err instanceof Error && err.name === "AbortError") return;
      const message = err instanceof Error ? err.message : String(err);
      setData((prev) => ({ ...prev, loading: false, error: message }));
    }
  }, [token]);

  // Start polling only after the token has been loaded.
  useEffect(() => {
    if (token === null) return;
    void refresh();
    const id = setInterval(() => void refresh(), POLL_INTERVAL_MS);
    return () => {
      clearInterval(id);
      abortRef.current?.abort();
    };
  }, [refresh, token]);

  return data;
}
