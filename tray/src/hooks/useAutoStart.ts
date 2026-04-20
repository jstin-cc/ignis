import { useCallback, useEffect, useState } from "react";

async function invokeAutostart(
  cmd: string,
  args?: Record<string, unknown>
): Promise<unknown> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke(cmd, args);
  } catch {
    return undefined;
  }
}

export function useAutoStart() {
  const [isEnabled, setIsEnabled] = useState(false);

  useEffect(() => {
    void invokeAutostart("get_autostart_enabled").then((val) => {
      if (typeof val === "boolean") setIsEnabled(val);
    });
  }, []);

  const toggle = useCallback(async () => {
    const next = !isEnabled;
    await invokeAutostart("set_autostart_enabled", { enabled: next });
    setIsEnabled(next);
  }, [isEnabled]);

  return { isEnabled, toggle };
}
