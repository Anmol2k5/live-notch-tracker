import { bandFor, colorFor } from '../model/usageBand';
import { palette } from '../design/palette';

const R = 20;
const C = 2 * Math.PI * R;

export function ProviderRing({ fraction }: { fraction: number }) {
  const clamped = Math.min(1, Math.max(0, fraction));
  const band = bandFor(clamped);
  return (
    <svg width={48} height={48} viewBox="0 0 48 48" style={{ display: 'block' }}>
      <circle cx={24} cy={24} r={R} fill="none" stroke={palette.ringTrack} strokeWidth={5} />
      <circle
        cx={24}
        cy={24}
        r={R}
        fill="none"
        stroke={colorFor(band)}
        strokeWidth={5}
        strokeLinecap="round"
        strokeDasharray={`${(clamped * C).toFixed(2)} ${C.toFixed(2)}`}
        transform="rotate(-90 24 24)"
      />
    </svg>
  );
}
