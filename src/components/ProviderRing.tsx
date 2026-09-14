import { bandFor, colorFor } from '../model/usageBand';
import { palette, glowColor } from '../design/palette';
import { ProviderStatus } from '../model/providerTypes';

const R = 20;
const C = 2 * Math.PI * R;

export function ProviderRing({ fraction, status }: { fraction: number; status: ProviderStatus }) {
  const clamped = Math.min(1, Math.max(0, fraction));
  const band = bandFor(clamped);
  const arcColor = colorFor(band);
  const glow = glowColor[band] ?? 'rgba(255,255,255,0.2)';

  // Stale and backoff states dim the ring opacity
  const isStale = status.kind === 'stale' || status.kind === 'backoff';
  const opacity = isStale ? 0.4 : 1.0;

  // Error or needsAuth renders a distinct track with no active arc
  const isError = status.kind === 'error' || status.kind === 'needsAuth';
  const trackColor = isError ? '#b45335' : palette.ringTrack;
  const showArc = !isError;

  const filterId = `ring-glow-${band}`;
  const glowClass = band === 'critical' && !isStale ? 'ring-glow-critical' : '';

  return (
    <svg width={48} height={48} viewBox="0 0 48 48" style={{ display: 'block', opacity }}>
      <defs>
        <filter id={filterId} x="-50%" y="-50%" width="200%" height="200%">
          <feGaussianBlur in="SourceGraphic" stdDeviation="2.2" result="blur" />
          <feFlood floodColor={glow} result="color" />
          <feComposite in="color" in2="blur" operator="in" result="glow" />
          <feMerge>
            <feMergeNode in="glow" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>
      {/* Track ring */}
      <circle
        cx={24} cy={24} r={R}
        fill="none"
        stroke={trackColor}
        strokeWidth={4}
        strokeLinecap="round"
        opacity={0.6}
      />
      {/* Active arc with glow */}
      {showArc && (
        <circle
          className={`ring-arc ${glowClass}`}
          cx={24} cy={24} r={R}
          fill="none"
          stroke={arcColor}
          strokeWidth={4.5}
          strokeLinecap="round"
          strokeDasharray={`${(clamped * C).toFixed(2)} ${C.toFixed(2)}`}
          transform="rotate(-90 24 24)"
          filter={`url(#${filterId})`}
        />
      )}
    </svg>
  );
}
