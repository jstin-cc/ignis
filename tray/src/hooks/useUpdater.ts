import { useCallback, useState } from "react";

interface UpdateCheckResult {
  available: boolean;
  version: string;
  body: string | null;
}

async function invokeCheckForUpdate(): Promise<UpdateCheckResult> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<UpdateCheckResult>("check_for_update");
  } catch (e) {
    throw new Error(e instanceof Error ? e.message : String(e));
  }
}

async function invokeInstallUpdate(): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("install_update");
}

export function useUpdater() {
  const [checking, setChecking] = useState(false);
  const [installing, setInstalling] = useState(false);
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

  const installUpdate = useCallback(async () => {
    setInstalling(true);
    setError(null);
    try {
      await invokeInstallUpdate();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setInstalling(false);
    }
    // On success Tauri restarts the app — no setInstalling(false) needed.
  }, []);

  return { checking, installing, result, error, checkForUpdate, installUpdate };
}
