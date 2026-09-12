import { bandFor, colorFor } from '../model/usageBand';
import { palette } from '../design/palette';
import { ProviderStatus } from '../model/providerTypes';

const R = 20;
const C = 2 * Math.PI * R;

export function ProviderRing({ fraction, status }: { fraction: number; status: ProviderStatus }) {
  const clamped = Math.min(1, Math.max(0, fraction));
  const band = bandFor(clamped);
  
  // Stale and backoff states dim the ring opacity
  const isStale = status.kind === 'stale' || status.kind === 'backoff';
  const opacity = isStale ? 0.4 : 1.0;
  
  // Error or needsAuth renders a distinct track with no active arc
  const isError = status.kind === 'error' || status.kind === 'needsAuth';
  const trackColor = isError ? '#D97757' : palette.ringTrack; // Amber-ish red for error track
  const showArc = !isError;

  return (
    <svg width={48} height={48} viewBox="0 0 48 48" style={{ display: 'block', opacity }}>
      <circle cx={24} cy={24} r={R} fill="none" stroke={trackColor} strokeWidth={5} />
      {showArc && (
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
      )}
    </svg>
  );
}
