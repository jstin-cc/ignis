import { useState } from "react";

export function Footer({ onOpenDashboard }: { onOpenDashboard: () => void }) {
  const [cliCopied, setCliCopied] = useState(false);

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
        onClick={onOpenDashboard}
        title="Dashboard öffnen"
      >
        Open Dashboard
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
