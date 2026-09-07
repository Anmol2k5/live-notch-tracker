export interface NotchProfile {
  edgeX: number;
  top: [number, number];
  bottom: [number, number];
  samples: Array<[number, number]>;
  d: string;
}

export function notchPath(width: number, height: number): NotchProfile {
  const edgeX = width;
  const curl = Math.min(38.5 * (44 / 117) * (117 / 44), width, height / 2);
  const flare = Math.min(width * 0.55, curl);
  const top: [number, number] = [edgeX, 0];
  const bottom: [number, number] = [edgeX, height];
  const innerTop: [number, number] = [edgeX - width + flare * 0.2, flare];
  const innerBottom: [number, number] = [edgeX - width + flare * 0.2, height - flare];
  const d = [
    `M ${edgeX} 0`,
    `L ${edgeX} ${height}`,
    `L ${innerBottom[0]} ${innerBottom[1]}`,
    `Q ${edgeX - width} ${height / 2} ${innerTop[0]} ${innerTop[1]}`,
    'Z',
  ].join(' ');
  const samples: Array<[number, number]> = [top, innerTop, [edgeX - width, height / 2], innerBottom, bottom];
  return { edgeX, top, bottom, samples, d };
}

export function SideNotchShape({ width, height }: { width: number; height: number }) {
  const { d } = notchPath(width, height);
  return (
    <svg width={width} height={height} style={{ display: 'block' }}>
      <path d={d} fill="#000000" />
    </svg>
  );
}
