import { ProviderRing } from './ProviderRing';
import { palette } from '../design/palette';
import { typography } from '../design/typography';

export function ProviderCell({ initial, percent }: { initial: string; percent: number }) {
  return (
    <div style={{ position: 'relative', width: 56, textAlign: 'center' }}>
      <div style={{ position: 'relative', width: 48, height: 48, margin: '0 auto' }}>
        <ProviderRing fraction={percent / 100} />
        <div
          style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: palette.textPrimary,
            fontSize: 16,
            fontWeight: 600,
          }}
        >
          {initial}
        </div>
      </div>
      <div style={{ color: palette.textPrimary, fontSize: typography.percentLabelPt, fontWeight: 600 }}>
        {percent}%
      </div>
    </div>
  );
}
