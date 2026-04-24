interface LineChartProps {
  values: number[];
  width?: number;
  height?: number;
}

export function LineChart({ values, width = 328, height = 80 }: LineChartProps) {
  const gradientId = 'linechart-gradient';

  if (values.length === 0 || values.every((v) => v === 0)) {
    return (
      <svg width={width} height={height} style={{ display: 'block' }}>
        <line
          x1={0}
          y1={height - 1}
          x2={width}
          y2={height - 1}
          stroke="#C15F3C"
          strokeWidth={1.5}
          opacity={0.3}
        />
      </svg>
    );
  }

  const max = Math.max(...values);
  const min = 0;
  const range = max - min || 1;

  const xStep = values.length > 1 ? width / (values.length - 1) : 0;

  const points = values.map((v, i) => {
    const x = i * xStep;
    const y = height - ((v - min) / range) * (height - 4);
    return { x, y };
  });

  const polylinePoints = points.map((p) => `${p.x},${p.y}`).join(' ');

  // Area path: go along the line points, then close along the bottom.
  const areaPath =
    `M ${points[0].x},${height} ` +
    points.map((p) => `L ${p.x},${p.y}`).join(' ') +
    ` L ${points[points.length - 1].x},${height} Z`;

  const lastPoint = points[points.length - 1];

  return (
    <svg width={width} height={height} style={{ display: 'block' }}>
      <defs>
        <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="rgba(193,95,60,0.22)" />
          <stop offset="100%" stopColor="rgba(193,95,60,0)" />
        </linearGradient>
      </defs>
      <path d={areaPath} fill={`url(#${gradientId})`} />
      <polyline
        points={polylinePoints}
        fill="none"
        stroke="#C15F3C"
        strokeWidth={1.5}
        strokeLinejoin="round"
        strokeLinecap="round"
      />
      <circle
        cx={lastPoint.x}
        cy={lastPoint.y}
        r={3}
        fill="#C15F3C"
      />
    </svg>
  );
}
