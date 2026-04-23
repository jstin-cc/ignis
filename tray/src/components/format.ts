// Pure formatting helpers — no React, no side effects.

export const fmt = {
  usd: (n: number) => '$' + n.toFixed(2),
  tok: (n: number) => {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
    if (n >= 1_000)     return (n / 1_000).toFixed(0) + 'k';
    return n.toString();
  },
  dur: (s: number) => {
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    return h > 0 ? `${h}h ${m}m` : `${m}m ${s % 60}s`;
  },
};

export function formatCost(usd: string): string {
  const n = parseFloat(usd);
  if (isNaN(n)) return "$—";
  return `$${n.toFixed(2)}`;
}

export function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M tokens`;
  if (n >= 1_000) return `${Math.round(n / 1_000)}k tokens`;
  return `${n} tokens`;
}

/**
 * Derives elapsed duration between two ISO-8601 timestamp strings.
 * Returns a human-readable string like "2h 14m" or "45m".
 */
export function formatDuration(firstSeen: string, lastSeen: string): string {
  const diffMs =
    new Date(lastSeen).getTime() - new Date(firstSeen).getTime();
  const totalMinutes = Math.max(0, Math.floor(diffMs / 60_000));
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}

/**
 * Extracts a short display name from a full project path.
 * "D:\\.claude\\projects\\winusage" → "winusage"
 */
export function projectName(projectPath: string): string {
  // Handle both Windows backslash and POSIX forward-slash separators.
  const parts = projectPath.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] ?? projectPath;
}
