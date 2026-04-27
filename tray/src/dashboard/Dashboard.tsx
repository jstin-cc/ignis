import { useEffect, useState } from 'react';
import type { SummaryResponse, Session, ActiveBlock, HeatmapDay } from '../types';
import { DashboardTabBar } from './DashboardTabBar';
import type { DashTab } from './DashboardTabBar';
import { LiveTab } from './LiveTab';
import { HistoryTab } from './HistoryTab';
import { useBurnRate } from './useBurnRate';

interface DashboardProps {
  onClose: () => void;
  today: SummaryResponse | null;
  last30Days: SummaryResponse | null;
  activeSession: Session | null;
  activeBlock: ActiveBlock | null;
  heatmap: HeatmapDay[];
}

export function Dashboard({
  onClose,
  today,
  last30Days,
  activeSession,
  activeBlock,
  heatmap,
}: DashboardProps) {
  const [tab, setTab] = useState<DashTab>('live');
  const { buckets } = useBurnRate();

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [onClose]);

  return (
    <div style={styles.overlay}>
      {/* Header */}
      <header style={styles.header}>
        <button style={styles.backBtn} onClick={onClose} aria-label="Back">
          ←
        </button>
        <span style={styles.title}>Dashboard</span>
        {/* Symmetry placeholder matching back button width */}
        <div style={styles.placeholder} />
      </header>

      <DashboardTabBar active={tab} onChange={setTab} />

      <div style={styles.scrollBody}>
        {tab === 'live' ? (
          <LiveTab
            today={today}
            activeSession={activeSession}
            activeBlock={activeBlock}
            burnBuckets={buckets}
          />
        ) : (
          <HistoryTab last30Days={last30Days} heatmap={heatmap} />
        )}
      </div>
    </div>
  );
}

const styles = {
  overlay: {
    position: 'absolute' as const,
    top: 0,
    left: 0,
    width: '100%',
    height: '100%',
    background: 'var(--bg-base)',
    zIndex: 11,
    display: 'flex',
    flexDirection: 'column' as const,
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    height: '48px',
    padding: '0 8px 0 4px',
    backgroundColor: 'var(--bg-elevated)',
    flexShrink: 0,
    borderBottom: '1px solid var(--border-subtle)',
  },
  backBtn: {
    width: '28px',
    height: '28px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: 'transparent',
    border: 'none',
    borderRadius: '4px',
    color: 'var(--text-secondary)',
    fontSize: '16px',
    cursor: 'pointer',
    fontFamily: 'var(--font-sans)',
    lineHeight: 1,
    flexShrink: 0,
  },
  title: {
    fontSize: '14px',
    fontWeight: 600,
    color: 'var(--text-primary)',
  },
  placeholder: {
    width: '28px',
    flexShrink: 0,
  },
  scrollBody: {
    flex: 1,
    overflowY: 'auto' as const,
    maxHeight: '540px',
  },
} as const;
