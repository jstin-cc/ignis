import { ringColorForPct } from '../utils/ringColors';

interface BlockRingProps {
  pct: number;
  size?: number;
}

export function BlockRing({ pct, size = 72 }: BlockRingProps) {
  const strokeWidth = 5;
  const r = size / 2 - strokeWidth / 2 - 1;
  const circumference = 2 * Math.PI * r;
  const clampedPct = Math.min(100, Math.max(0, pct));
  const offset = circumference * (1 - clampedPct / 100);
  const color = ringColorForPct(pct);

  return (
    <svg
      width={size}
      height={size}
      style={{ display: 'block', transform: 'rotate(-90deg)' }}
    >
      <circle
        cx={size / 2}
        cy={size / 2}
        r={r}
        fill="none"
        stroke="var(--border-subtle)"
        strokeWidth={strokeWidth}
      />
      <circle
        cx={size / 2}
        cy={size / 2}
        r={r}
        fill="none"
        stroke={color}
        strokeWidth={strokeWidth}
        strokeDasharray={circumference}
        strokeDashoffset={offset}
        strokeLinecap="round"
        style={{ transition: 'stroke-dashoffset 300ms ease-out, stroke 300ms ease-out' }}
      />
    </svg>
  );
}
