import { useEffect, useRef, useState } from 'react';

const API_BASE = 'http://127.0.0.1:7337';
const POLL_INTERVAL_MS = 30_000;
const FETCH_TIMEOUT_MS = 10_000;

export interface BurnRateBucket {
  minute_start: string;
  tokens: number;
  cost_usd: string;
}

async function loadApiToken(): Promise<string> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke<string>('get_api_token');
  } catch {
    return '';
  }
}

function authHeaders(token: string): HeadersInit {
  return token ? { Authorization: `Bearer ${token}` } : {};
}

export function useBurnRate(): { buckets: BurnRateBucket[]; error: string | null } {
  const [token, setToken] = useState<string | null>(null);
  const [buckets, setBuckets] = useState<BurnRateBucket[]>([]);
  const [error, setError] = useState<string | null>(null);
  const abortRef = useRef<AbortController | null>(null);

  useEffect(() => {
    loadApiToken().then(setToken);
  }, []);

  useEffect(() => {
    if (token === null) return;

    async function fetchBurnRate() {
      abortRef.current?.abort();
      const controller = new AbortController();
      abortRef.current = controller;
      const timeoutId = setTimeout(() => controller.abort(), FETCH_TIMEOUT_MS);

      try {
        const resp = await fetch(`${API_BASE}/v1/burn-rate`, {
          headers: authHeaders(token ?? ''),
          signal: controller.signal,
        });
        clearTimeout(timeoutId);
        if (controller.signal.aborted) return;
        if (!resp.ok) throw new Error(`burn-rate returned ${resp.status}`);
        const data = (await resp.json()) as BurnRateBucket[];
        setBuckets(data);
        setError(null);
      } catch (err) {
        clearTimeout(timeoutId);
        if (err instanceof Error && err.name === 'AbortError') {
          setError('API nicht erreichbar (Timeout)');
          return;
        }
        setError(err instanceof Error ? err.message : String(err));
      }
    }

    void fetchBurnRate();
    const id = setInterval(() => void fetchBurnRate(), POLL_INTERVAL_MS);
    return () => {
      clearInterval(id);
      abortRef.current?.abort();
    };
  }, [token]);

  return { buckets, error };
}
