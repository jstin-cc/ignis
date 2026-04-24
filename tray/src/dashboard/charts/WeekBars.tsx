interface WeekDay {
  label: string;
  thisWeek: number;
  lastWeek: number;
}

interface WeekBarsProps {
  days: WeekDay[];
  width?: number;
  height?: number;
}

export function WeekBars({ days, width = 328, height = 64 }: WeekBarsProps) {
  const labelHeight = 16;
  const chartHeight = height - labelHeight;
  const groupWidth = width / 7;
  const barWidth = groupWidth * 0.35;
  const barGap = 2;
  const groupCenter = groupWidth / 2;

  const allValues = days.flatMap((d) => [d.thisWeek, d.lastWeek]);
  const max = Math.max(...allValues, 0.0001);

  return (
    <svg width={width} height={height} style={{ display: 'block' }}>
      {days.map((day, i) => {
        const groupX = i * groupWidth;

        const thisH = Math.max(2, (day.thisWeek / max) * chartHeight);
        const lastH = Math.max(day.lastWeek > 0 ? 2 : 0, (day.lastWeek / max) * chartHeight);

        // Last week bar: left of center
        const lastX = groupX + groupCenter - barGap / 2 - barWidth;
        // This week bar: right of center
        const thisX = groupX + groupCenter + barGap / 2;

        return (
          <g key={i}>
            {day.lastWeek > 0 && (
              <rect
                x={lastX}
                y={chartHeight - lastH}
                width={barWidth}
                height={lastH}
                fill="rgba(130,120,110,0.45)"
                rx={1}
              />
            )}
            {day.thisWeek > 0 && (
              <rect
                x={thisX}
                y={chartHeight - thisH}
                width={barWidth}
                height={thisH}
                fill="var(--accent)"
                rx={1}
              />
            )}
            <text
              x={groupX + groupCenter}
              y={height - 2}
              textAnchor="middle"
              fill="var(--text-muted)"
              fontSize={10}
              fontFamily="var(--font-sans)"
            >
              {day.label}
            </text>
          </g>
        );
      })}
    </svg>
  );
}
