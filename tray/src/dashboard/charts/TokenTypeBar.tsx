interface TokenTypeBarProps {
  input: number;
  output: number;
  cacheRead: number;
  cacheWrite: number;
  height?: number;
}

export function TokenTypeBar({
  input,
  output,
  cacheRead,
  cacheWrite,
  height = 4,
}: TokenTypeBarProps) {
  const total = input + output + cacheRead + cacheWrite;

  if (total === 0) {
    return (
      <div
        style={{
          height: `${height}px`,
          background: 'var(--border-subtle)',
          borderRadius: `${height / 2}px`,
          width: '100%',
        }}
      />
    );
  }

  const segments: { value: number; color: string }[] = [
    { value: input,      color: 'var(--chart-input)' },
    { value: output,     color: 'var(--chart-output)' },
    { value: cacheRead,  color: 'var(--chart-cache-read)' },
    { value: cacheWrite, color: 'var(--chart-cache-write)' },
  ];

  return (
    <div
      style={{
        display: 'flex',
        height: `${height}px`,
        borderRadius: `${height / 2}px`,
        overflow: 'hidden',
        width: '100%',
      }}
    >
      {segments
        .filter((s) => s.value > 0)
        .map((s, i) => (
          <div
            key={i}
            style={{
              width: `${(s.value / total) * 100}%`,
              background: s.color,
            }}
          />
        ))}
    </div>
  );
}
