import type { FixtureCell } from '../state/fixtures';
import { ProviderCell } from './ProviderCell';
import { SideNotchShape } from './SideNotchShape';

export function NotchRootView({ cells, width, height }: { cells: FixtureCell[]; width: number; height: number }) {
  return (
    <div style={{ position: 'relative', width, height }}>
      <div style={{ position: 'absolute', inset: 0 }}>
        <SideNotchShape width={width} height={height} />
      </div>
      <div
        style={{
          position: 'absolute',
          inset: 0,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'space-evenly',
        }}
      >
        {cells.map((cell) => (
          <ProviderCell key={cell.id} initial={cell.initial} percent={cell.percent} />
        ))}
      </div>
    </div>
  );
}
