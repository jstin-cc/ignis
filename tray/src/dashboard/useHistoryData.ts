import type { HeatmapDay } from '../types';

export interface WeekDay {
  label: string;
  thisWeek: number;
  lastWeek: number;
}

export interface HistoryData {
  weekComparison: WeekDay[];
  costTrend30d: number[];
}

const DAY_LABELS = ['Mo', 'Di', 'Mi', 'Do', 'Fr', 'Sa', 'So'];

// Returns the Monday of the ISO week containing `date`.
function mondayOf(date: Date): Date {
  const d = new Date(date);
  // getDay(): 0=Sun, 1=Mon, ..., 6=Sat. Convert to Mon-based (0=Mon, 6=Sun).
  const dow = (d.getDay() + 6) % 7;
  d.setDate(d.getDate() - dow);
  d.setHours(0, 0, 0, 0);
  return d;
}

function dateOnly(iso: string): string {
  return iso.slice(0, 10);
}

export function useHistoryData(heatmap: HeatmapDay[]): HistoryData {
  // costTrend30d: last 30 entries (already sorted by date ascending)
  const last30 = heatmap.slice(-30);
  const costTrend30d = last30.map((d) => {
    const v = parseFloat(d.cost_usd);
    return isNaN(v) ? 0 : v;
  });

  // weekComparison
  const today = heatmap.length > 0 ? new Date(heatmap[heatmap.length - 1].date) : new Date();
  const thisWeekStart = mondayOf(today);
  const lastWeekStart = new Date(thisWeekStart);
  lastWeekStart.setDate(lastWeekStart.getDate() - 7);
  const lastWeekEnd = new Date(thisWeekStart);
  lastWeekEnd.setDate(lastWeekEnd.getDate() - 1);

  // Build lookup: date string -> cost
  const costByDate = new Map<string, number>();
  for (const d of heatmap) {
    const v = parseFloat(d.cost_usd);
    costByDate.set(d.date, isNaN(v) ? 0 : v);
  }

  const weekComparison: WeekDay[] = DAY_LABELS.map((label, dayIndex) => {
    const thisDate = new Date(thisWeekStart);
    thisDate.setDate(thisDate.getDate() + dayIndex);
    const lastDate = new Date(lastWeekStart);
    lastDate.setDate(lastDate.getDate() + dayIndex);

    return {
      label,
      thisWeek: costByDate.get(dateOnly(thisDate.toISOString())) ?? 0,
      lastWeek: costByDate.get(dateOnly(lastDate.toISOString())) ?? 0,
    };
  });

  return { weekComparison, costTrend30d };
}
