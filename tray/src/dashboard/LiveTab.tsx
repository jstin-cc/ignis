import type { SummaryResponse, Session, ActiveBlock } from '../types';
import type { BurnRateBucket } from './useBurnRate';
import { fmt, formatCost, formatDuration, projectName } from '../components/format';
import { Sparkline } from './charts/Sparkline';
import { BlockRing } from './charts/BlockRing';
import { TokenTypeBar } from './charts/TokenTypeBar';

interface LiveTabProps {
  today: SummaryResponse | null;
  activeSession: Session | null;
  activeBlock: ActiveBlock | null;
  burnBuckets: BurnRateBucket[];
}

function shortModelName(model: string): string {
  const parts = model.replace(/\//g, '-').split('-');
  const name = parts.slice(-3).join('-');
  return name.length > 20 ? name.slice(0, 20) : name;
}

function avgTokPerMin(buckets: BurnRateBucket[]): number {
  const nonZero = buckets.filter((b) => b.tokens > 0);
  const sample = nonZero.length > 0 ? nonZero.slice(-5) : buckets.slice(-5);
  if (sample.length === 0) return 0;
  const sum = sample.reduce((acc, b) => acc + b.tokens, 0);
  return Math.round(sum / sample.length);
}

function blockResetLabel(block: ActiveBlock): string {
  const endMs = new Date(block.end).getTime();
  const nowMs = Date.now();
  const diffMs = endMs - nowMs;
  if (diffMs <= 0) return 'now';
  const totalMins = Math.floor(diffMs / 60_000);
  const h = Math.floor(totalMins / 60);
  const m = totalMins % 60;
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

export function LiveTab({ today, activeSession, activeBlock, burnBuckets }: LiveTabProps) {
  const sparklineValues = burnBuckets.map((b) => b.tokens);
  const avgTok = avgTokPerMin(burnBuckets);

  // Aggregated token types across all models in today
  const totalInput = today?.by_model.reduce((s, m) => s + m.input_tokens, 0) ?? 0;
  const totalOutput = today?.by_model.reduce((s, m) => s + m.output_tokens, 0) ?? 0;
  const totalCacheRead = today?.by_model.reduce((s, m) => s + m.cache_read_tokens, 0) ?? 0;
  const totalCacheWrite = today?.by_model.reduce((s, m) => s + m.cache_creation_tokens, 0) ?? 0;

  const sessionTotalTokens = activeSession
    ? activeSession.by_model.reduce((s, m) => s + m.tokens, 0)
    : 0;

  const models = today?.by_model ?? [];
  const maxModelCost = models.reduce(
    (max, m) => Math.max(max, parseFloat(m.cost_usd) || 0),
    0,
  );

  return (
    <div style={styles.container}>
      {/* BURN RATE */}
      <section style={styles.section}>
        <div className="section-label">BURN RATE</div>
        <Sparkline values={sparklineValues} />
        <span style={styles.meta} className="tabular">
          avg {fmt.tok(avgTok)} tok/min
        </span>
      </section>

      <hr className="section-divider" />

      {/* ACTIVE SESSION */}
      <section style={styles.section}>
        <div className="section-label">ACTIVE SESSION</div>
        {activeSession === null ? (
          <span style={styles.muted}>no active session</span>
        ) : (
          <>
            <div style={styles.row}>
              <div style={styles.sessionLeft}>
                <span style={styles.sessionName} className="tabular">
                  {projectName(activeSession.project_path)}
                </span>
                <span style={styles.meta} className="tabular">
                  {formatDuration(activeSession.first_seen, activeSession.last_seen)}
                </span>
              </div>
              <div style={styles.sessionRight}>
                <span style={styles.hero} className="tabular">
                  {formatCost(activeSession.total_cost_usd)}
                </span>
                <span style={styles.meta} className="tabular">
                  {fmt.tok(sessionTotalTokens)}
                </span>
              </div>
            </div>
            <div style={styles.tokenBar}>
              <TokenTypeBar
                input={totalInput}
                output={totalOutput}
                cacheRead={totalCacheRead}
                cacheWrite={totalCacheWrite}
              />
            </div>
          </>
        )}
      </section>

      <hr className="section-divider" />

      {/* SESSION BLOCK */}
      <section style={styles.section}>
        <div className="section-label">SESSION BLOCK</div>
        {activeBlock === null ? (
          <span style={styles.muted}>no active block</span>
        ) : (
          <div style={styles.blockContent}>
            <BlockRing pct={activeBlock.block_token_pct} />
            <div style={styles.blockMeta}>
              <span
                style={{
                  ...styles.blockPct,
                  color: activeBlock.block_token_pct >= 100 ? 'var(--danger)' : 'var(--text-primary)',
                }}
                className="tabular"
              >
                {activeBlock.block_token_pct}% used
              </span>
              <span style={styles.meta} className="tabular">
                resets in {blockResetLabel(activeBlock)}
              </span>
              {activeBlock.block_token_pct >= 100 && (
                <span style={styles.danger}>token limit reached</span>
              )}
            </div>
          </div>
        )}
      </section>

      <hr className="section-divider" />

      {/* BY MODEL */}
      <section style={styles.section}>
        <div className="section-label">BY MODEL</div>
        {models.length === 0 ? (
          <span style={styles.muted}>no data</span>
        ) : (
          <div style={styles.modelList}>
            {models.map((m) => {
              const cost = parseFloat(m.cost_usd) || 0;
              const barPct = maxModelCost > 0 ? (cost / maxModelCost) * 100 : 0;
              return (
                <div key={m.model} style={styles.modelRow}>
                  <span style={styles.modelName} className="tabular">
                    {shortModelName(m.model)}
                  </span>
                  <div style={styles.modelBarWrap}>
                    <div
                      style={{ ...styles.modelBar, width: `${barPct}%` }}
                    />
                  </div>
                  <span style={styles.modelCost} className="tabular">
                    {formatCost(m.cost_usd)}
                  </span>
                </div>
              );
            })}
          </div>
        )}
      </section>
    </div>
  );
}

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
  },
  section: {
    padding: '12px 16px',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '6px',
  },
  row: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    gap: '8px',
  },
  sessionLeft: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '2px',
    flex: 1,
    minWidth: 0,
  },
  sessionRight: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'flex-end',
    gap: '2px',
    flexShrink: 0,
  },
  sessionName: {
    fontSize: '13px',
    fontWeight: 500,
    color: 'var(--text-primary)',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap' as const,
  },
  hero: {
    fontSize: '18px',
    fontWeight: 600,
    color: 'var(--text-primary)',
  },
  meta: {
    fontSize: '11px',
    color: 'var(--text-secondary)',
  },
  muted: {
    fontSize: '12px',
    color: 'var(--text-muted)',
    marginTop: '4px',
  },
  tokenBar: {
    marginTop: '4px',
  },
  blockContent: {
    display: 'flex',
    alignItems: 'center',
    gap: '16px',
  },
  blockMeta: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '4px',
  },
  blockPct: {
    fontSize: '14px',
    fontWeight: 600,
  },
  danger: {
    fontSize: '11px',
    color: 'var(--danger)',
    fontWeight: 500,
  },
  modelList: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '6px',
  },
  modelRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  modelName: {
    fontSize: '11px',
    color: 'var(--accent)',
    fontFamily: 'var(--font-mono)',
    width: '120px',
    flexShrink: 0,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap' as const,
  },
  modelBarWrap: {
    flex: 1,
    height: '4px',
    background: 'var(--border-subtle)',
    borderRadius: '2px',
    overflow: 'hidden',
  },
  modelBar: {
    height: '100%',
    background: 'var(--accent)',
    borderRadius: '2px',
    transition: 'width 200ms ease-out',
  },
  modelCost: {
    fontSize: '11px',
    color: 'var(--text-secondary)',
    width: '48px',
    textAlign: 'right' as const,
    flexShrink: 0,
  },
} as const;
