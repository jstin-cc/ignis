interface FooterProps {
  onOpenDashboard: () => void;
}

export function Footer({ onOpenDashboard }: FooterProps) {
  return (
    <footer style={styles.footer}>
      <button style={styles.primaryBtn} onClick={onOpenDashboard}>
        Open Dashboard
      </button>
      <button
        style={styles.secondaryBtn}
        onClick={() => {
          /* CLI copy-hint: no shell spawn in MVP */
        }}
      >
        CLI: winusage
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
