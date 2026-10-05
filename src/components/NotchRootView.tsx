import { ProviderCell } from './ProviderCell';
import { SideNotchShape } from './SideNotchShape';
import { ProviderId, ProviderStatus, UsageMetric, UsageSnapshot } from '../model/providerTypes';
import { palette } from '../design/palette';

export interface UIProviderCell {
  id: ProviderId;
  metric: UsageMetric;
  status: ProviderStatus;
  snapshot: UsageSnapshot;
}

export function NotchRootView({
  cells,
  width,
  height,
  onRefreshAll,
}: {
  cells: UIProviderCell[];
  width: number;
  height: number;
  onRefreshAll?: () => void;
}) {
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
        {cells.length === 0 ? (
          // Sensible empty / listening state when no AI tool is running
          <div
            className="idle-action"
            title="Live Notch Tracker&#10;No active AI provider sessions detected&#10;Open Claude, Cursor, Codex, or Antigravity&#10;Click to refresh"
            onClick={onRefreshAll}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                onRefreshAll?.();
              }
            }}
            tabIndex={0}
            role="button"
            aria-label="No active AI provider sessions detected. Press Enter to check for active providers."
            style={{
              cursor: 'pointer',
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              padding: 8,
              textAlign: 'center',
            }}
          >
            <svg
              width={28}
              height={28}
              viewBox="0 0 24 24"
              fill="none"
              stroke={palette.textSecondary}
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="ring-glow-critical"
              style={{ opacity: 0.6 }}
            >
              <circle cx="12" cy="12" r="10" />
              <path d="M12 6v6l4 2" />
            </svg>
            <div
              style={{
                fontSize: 10,
                color: palette.textSecondary,
                marginTop: 6,
                fontWeight: 600,
                letterSpacing: '0.05em',
                textTransform: 'uppercase',
              }}
            >
              Idle
            </div>
          </div>
        ) : (
          cells.map((cell, index) => (
            <ProviderCell
              key={cell.id}
              id={cell.id}
              index={index}
              metric={cell.metric}
              status={cell.status}
              snapshot={cell.snapshot}
              onRefresh={onRefreshAll}
            />
          ))
        )}
      </div>
    </div>
  );
}
