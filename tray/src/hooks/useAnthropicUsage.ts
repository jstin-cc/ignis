import { useState, useEffect, useRef } from "react";
import type { AnthropicUsage } from "../types";

export function useAnthropicUsage(
  pollIntervalSecs: number,
): { usage: AnthropicUsage | null; error: string | null } {
  const [usage, setUsage] = useState<AnthropicUsage | null>(null);
  const [error, setError] = useState<string | null>(null);
  const cacheRef = useRef<AnthropicUsage | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const result = await invoke<AnthropicUsage>("get_anthropic_usage");
        if (!cancelled) {
          cacheRef.current = result;
          setUsage(result);
          setError(null);
        }
      } catch (e) {
        if (!cancelled) {
          const msg = e instanceof Error ? e.message : String(e);
          setError(msg);
          if (cacheRef.current) setUsage(cacheRef.current);
        }
      }
    }

    void load();
    const id = setInterval(() => void load(), pollIntervalSecs * 1_000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [pollIntervalSecs]);

  return { usage, error };
}
