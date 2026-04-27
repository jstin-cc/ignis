export type TabId = 'today' | 'month' | 'projects' | 'heatmap' | 'settings';

interface TabBarProps {
  active: TabId;
  onChange: (tab: TabId) => void;
}

const TABS: { id: TabId; label: string }[] = [
  { id: 'today',    label: 'Today' },
  { id: 'month',    label: 'Month' },
  { id: 'projects', label: 'Projects' },
  { id: 'heatmap',  label: 'Heatmap' },
  { id: 'settings', label: 'Settings' },
];

export function TabBar({ active, onChange }: TabBarProps) {
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
    width: '360px',
    borderBottom: '1px solid var(--border-subtle)',
    flexShrink: 0,
  },
  tab: {
    flex: 1,
    height: '100%',
    padding: '0 2px',
    background: 'transparent',
    border: 'none',
    borderRadius: 0,
    fontFamily: 'var(--font-sans)',
    fontSize: '11px',
    fontWeight: 500,
    textTransform: 'uppercase' as const,
    letterSpacing: '0.03em',
    cursor: 'pointer',
    transition: 'color 120ms',
  },
} as const;
