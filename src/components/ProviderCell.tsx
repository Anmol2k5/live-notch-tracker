import { ProviderRing } from './ProviderRing';
import { palette } from '../design/palette';
import { typography } from '../design/typography';
import { ProviderStatus } from '../model/providerTypes';

export function ProviderCell({ initial, percent, status }: { initial: string; percent: number; status: ProviderStatus }) {
  const isStale = status.kind === 'stale' || status.kind === 'backoff';
  const textOpacity = isStale ? 0.6 : 1.0;
  
  return (
    <div style={{ position: 'relative', width: 56, textAlign: 'center' }}>
      <div style={{ position: 'relative', width: 48, height: 48, margin: '0 auto' }}>
        <ProviderRing fraction={percent / 100} status={status} />
        <div
          style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: palette.textPrimary,
            fontSize: typography.providerInitialPt,
            fontWeight: 600,
            opacity: textOpacity,
          }}
        >
          {initial}
        </div>
      </div>
      <div style={{ color: palette.textPrimary, fontSize: typography.percentLabelPt, fontWeight: 600, opacity: textOpacity }}>
        {percent}%
      </div>
    </div>
  );
}
