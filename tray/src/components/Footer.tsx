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
        className="btn btn--primary"
        style={styles.grow}
        onClick={() => void handleOpenDashboard()}
        title={dashError ?? "ignis-watch (TUI) starten"}
      >
        {dashError ? "Fehler" : "Open Dashboard"}
      </button>
      <button
        className="btn btn--ghost"
        style={styles.grow}
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
  grow: {
    flex: 1,
  },
} as const;
