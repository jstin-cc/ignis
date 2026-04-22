import { useState } from "react";

export function Footer({ onOpenDashboard: _ }: { onOpenDashboard: () => void }) {
  const [cliCopied, setCliCopied] = useState(false);
  const [dashError, setDashError] = useState<string | null>(null);

  async function handleOpenDashboard() {
    setDashError(null);
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("open_cli_dashboard");
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setDashError(msg);
      setTimeout(() => setDashError(null), 2500);
    }
  }

  async function handleCopyCli() {
    try {
      await navigator.clipboard.writeText("ignis");
      setCliCopied(true);
      setTimeout(() => setCliCopied(false), 1500);
    } catch {
      /* clipboard unavailable — ignore */
    }
  }

  return (
    <footer style={styles.footer}>
      <button
        style={styles.primaryBtn}
        onClick={() => void handleOpenDashboard()}
        title={dashError ?? "ignis-watch (TUI) starten"}
      >
        {dashError ? "Fehler" : "Open Dashboard"}
      </button>
      <button
        style={styles.secondaryBtn}
        onClick={() => void handleCopyCli()}
        title="Kopiert 'ignis' in die Zwischenablage"
      >
        {cliCopied ? "Kopiert ✓" : "CLI: ignis"}
      </button>
    </footer>
  );
}

const styles = {
  footer: {
    display: "flex",
    gap: "8px",
    padding: "12px 16px",
    backgroundColor: "var(--bg-base)",
    borderTop: "1px solid var(--border-subtle)",
    flexShrink: 0,
  },
  primaryBtn: {
    flex: 1,
    height: "32px",
    backgroundColor: "var(--accent)",
    color: "var(--bg-base)",
    border: "none",
    borderRadius: "4px",
    fontSize: "13px",
    fontWeight: 500,
    cursor: "pointer",
    fontFamily: "var(--font-ui)",
    transition: "background-color 120ms ease-out",
  },
  secondaryBtn: {
    flex: 1,
    height: "32px",
    backgroundColor: "var(--bg-elevated)",
    color: "var(--text-primary)",
    border: "1px solid var(--border-default)",
    borderRadius: "4px",
    fontSize: "13px",
    fontWeight: 500,
    cursor: "pointer",
    fontFamily: "var(--font-ui)",
    transition: "border-color 120ms ease-out",
  },
} as const;
