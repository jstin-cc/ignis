interface SparklineProps {
  values: number[];
  width?: number;
  height?: number;
}

export function Sparkline({ values, width = 328, height = 48 }: SparklineProps) {
  const count = values.length;
  if (count === 0) return <svg width={width} height={height} />;

  const barWidth = width / count;
  const gap = 1;
  const effectiveBarWidth = Math.max(1, barWidth - gap);
  const max = Math.max(...values);

  return (
    <svg
      width={width}
      height={height}
      style={{ display: 'block', overflow: 'visible' }}
    >
      {values.map((v, i) => {
        const barHeight = max === 0 ? 2 : Math.max(2, (v / max) * height);
        const x = i * barWidth;
        const y = height - barHeight;
        const isLast = i === count - 1;
        const opacity = max === 0 ? 0.25 : 0.25 + (v / max) * 0.55;
        const fill = isLast
          ? 'var(--accent)'
          : `rgba(193,95,60,${opacity.toFixed(3)})`;

        return (
          <rect
            key={i}
            x={x}
            y={y}
            width={effectiveBarWidth}
            height={barHeight}
            fill={fill}
            rx={1}
          />
        );
      })}
    </svg>
  );
}
