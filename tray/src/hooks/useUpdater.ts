import { useCallback, useState } from "react";

interface UpdateCheckResult {
  available: boolean;
  version: string;
}

async function invokeCheckForUpdate(): Promise<UpdateCheckResult> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<UpdateCheckResult>("check_for_update");
  } catch (e) {
    throw new Error(e instanceof Error ? e.message : String(e));
  }
}

export function useUpdater() {
  const [checking, setChecking] = useState(false);
  const [result, setResult] = useState<UpdateCheckResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const checkForUpdate = useCallback(async () => {
    setChecking(true);
    setResult(null);
    setError(null);
    try {
      const info = await invokeCheckForUpdate();
      setResult(info);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setChecking(false);
    }
  }, []);

  return { checking, result, error, checkForUpdate };
}
