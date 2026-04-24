export function ringColorForPct(pct: number): string {
  if (pct >= 100) return 'var(--danger)';
  if (pct >= 90)  return 'var(--warning)';
  if (pct >= 75)  return 'var(--accent)';
  return 'var(--accent-muted)';
}
