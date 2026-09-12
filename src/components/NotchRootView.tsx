import { ProviderCell } from './ProviderCell';
import { SideNotchShape } from './SideNotchShape';
import { ProviderStatus } from '../model/providerTypes';

export interface UIProviderCell {
  id: string;
  initial: string;
  percent: number;
  status: ProviderStatus;
}

export function NotchRootView({ cells, width, height }: { cells: UIProviderCell[]; width: number; height: number }) {
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
          <ProviderCell key={cell.id} initial={cell.initial} percent={cell.percent} status={cell.status} />
        ))}
      </div>
    </div>
  );
}
