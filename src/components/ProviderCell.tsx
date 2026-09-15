import { ProviderRing } from './ProviderRing';
import { palette } from '../design/palette';
import { typography } from '../design/typography';
import { ProviderStatus } from '../model/providerTypes';

export function ProviderCell({ index, initial, percent, status }: { index: number; initial: string; percent: number; status: ProviderStatus }) {
  const isStale = status.kind === 'stale' || status.kind === 'backoff';
  const textOpacity = isStale ? 0.5 : 1.0;
  const staggerDelay = `${index * 0.08}s`;

  return (
    <div
      className="provider-cell"
      style={{
        position: 'relative',
        width: 56,
        textAlign: 'center',
        animationDelay: staggerDelay,
      }}
    >
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
            fontFamily: typography.fontFamily,
            fontSize: typography.providerInitialPt,
            fontWeight: 600,
            opacity: textOpacity,
            letterSpacing: '-0.02em',
          }}
        >
          {initial}
        </div>
      </div>
      <div
        style={{
          color: palette.textSecondary,
          fontFamily: typography.fontFamily,
          fontSize: typography.percentLabelPt,
          fontWeight: 500,
          opacity: textOpacity,
          letterSpacing: '0.04em',
          marginTop: 1,
        }}
      >
        {percent}%
      </div>
    </div>
  );
}
