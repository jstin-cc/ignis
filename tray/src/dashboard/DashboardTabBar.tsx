export type DashTab = 'live' | 'history';

interface DashboardTabBarProps {
  active: DashTab;
  onChange: (tab: DashTab) => void;
}

const TABS: { id: DashTab; label: string }[] = [
  { id: 'live',    label: 'Live' },
  { id: 'history', label: 'History' },
];

export function DashboardTabBar({ active, onChange }: DashboardTabBarProps) {
  return (
    <div style={styles.bar}>
      {TABS.map((tab) => (
        <button
          key={tab.id}
          style={{
            ...styles.tab,
            color: active === tab.id ? 'var(--text-primary)' : 'var(--text-muted)',
            borderBottom: active === tab.id
              ? '2px solid var(--accent)'
              : '2px solid transparent',
          }}
          onClick={() => onChange(tab.id)}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}

const styles = {
  bar: {
    display: 'flex',
    height: '36px',
    width: '100%',
    borderBottom: '1px solid var(--border-subtle)',
    backgroundColor: 'var(--bg-elevated)',
    flexShrink: 0,
  },
  tab: {
    flex: 1,
    height: '100%',
    background: 'transparent',
    border: 'none',
    borderRadius: 0,
    fontFamily: 'var(--font-sans)',
    fontSize: '11px',
    fontWeight: 500,
    textTransform: 'uppercase' as const,
    letterSpacing: '0.06em',
    cursor: 'pointer',
    transition: 'color 120ms',
  },
} as const;
