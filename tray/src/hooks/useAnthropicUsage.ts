import { useState, useEffect, useRef } from "react";
import type { AnthropicUsage } from "../types";

const POLL_INTERVAL_MS = 5 * 60 * 1_000; // 5 minutes, same as OpenUsage

export function useAnthropicUsage(): AnthropicUsage | null {
  const [usage, setUsage] = useState<AnthropicUsage | null>(null);
  const cacheRef = useRef<AnthropicUsage | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function fetch() {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const result = await invoke<AnthropicUsage>("get_anthropic_usage");
        if (!cancelled) {
          cacheRef.current = result;
          setUsage(result);
        }
      } catch {
        // Offline or credentials missing — keep last cached value, don't surface error
        if (!cancelled && cacheRef.current) {
          setUsage(cacheRef.current);
        }
      }
    }

    void fetch();
    const id = setInterval(() => void fetch(), POLL_INTERVAL_MS);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, []);

  return usage;
}
