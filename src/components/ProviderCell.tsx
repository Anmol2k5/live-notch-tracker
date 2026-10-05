import { useState } from 'react';
import { ProviderRing } from './ProviderRing';
import { ProviderMark } from './ProviderMark';
import { palette } from '../design/palette';
import { typography } from '../design/typography';
import {
  ProviderId,
  ProviderStatus,
  UsageMetric,
  UsageSnapshot,
  PROVIDER_METADATA,
  formatMetricLabel,
  metricRingFraction,
} from '../model/providerTypes';

function formatResetCountdown(resetsAt?: number | null): string {
  if (!resetsAt) return '';
  const diffMs = resetsAt - Date.now();
  if (diffMs <= 0) return 'resets soon';
  const mins = Math.floor(diffMs / 60000);
  const hours = Math.floor(mins / 60);
  const remMins = mins % 60;
  if (hours > 0) {
    return `resets in ${hours}h ${remMins}m`;
  }
  return `resets in ${mins}m`;
}

export interface ProviderCellProps {
  id: ProviderId;
  index: number;
  metric: UsageMetric;
  status: ProviderStatus;
  snapshot?: UsageSnapshot;
  onRefresh?: () => void;
}

export function ProviderCell({
  id,
  index,
  metric,
  status,
  snapshot,
  onRefresh,
}: ProviderCellProps) {
  const [isHovered, setIsHovered] = useState(false);
  const meta = PROVIDER_METADATA[id] ?? {
    id,
    displayName: id,
    shortCode: id.slice(0, 2).toUpperCase(),
    accentColor: '#FFFFFF',
  };

  const isStale = status.kind === 'stale' || status.kind === 'backoff';
  const textOpacity = isStale ? 0.5 : 1.0;
  const staggerDelay = `${index * 0.08}s`;

  const resetText = formatResetCountdown(metric.resetsAt);
  const metricText = formatMetricLabel(metric);

  // Rich tooltip formatted for Windows WebView native tooltip rendering
  const tooltipParts = [
    `${meta.displayName} (${meta.shortCode})`,
    metric.kind === 'percentage'
      ? `Usage: ${metric.usedPercent}%${resetText ? ` · ${resetText}` : ''}`
      : metric.kind === 'count'
      ? `Requests: ${metric.used}${metric.limit ? ` / ${metric.limit}` : ''}${metric.derived ? ' (derived)' : ''}`
      : `Tokens: ${metricText}`,
  ];
  if (status.kind === 'stale') tooltipParts.push('Status: Stale (last reading kept)');
  if (status.kind === 'needsAuth') tooltipParts.push('Status: Needs sign-in');
  if (status.kind === 'backoff') tooltipParts.push('Status: Rate limited (cooling down)');
  if (snapshot?.note) tooltipParts.push(`Note: ${snapshot.note}`);
  tooltipParts.push('Click to refresh');
  const fullTooltip = tooltipParts.join('\n');

  return (
    <div
      className="provider-cell"
      title={fullTooltip}
      onClick={onRefresh}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      style={{
        position: 'relative',
        width: 56,
        textAlign: 'center',
        animationDelay: staggerDelay,
        cursor: 'pointer',
      }}
    >
      <div style={{ position: 'relative', width: 48, height: 48, margin: '0 auto' }}>
        <ProviderRing
          fraction={metricRingFraction(metric)}
          status={status}
          accentColor={meta.accentColor}
        />
        <div
          style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: isHovered ? meta.accentColor : palette.textPrimary,
            fontFamily: typography.fontFamily,
            fontSize: typography.providerInitialPt,
            fontWeight: 600,
            opacity: textOpacity,
            transition: 'color 0.15s ease, transform 0.15s ease',
            transform: isHovered ? 'scale(1.1)' : 'scale(1)',
          }}
        >
          <ProviderMark id={id} size={19} />
        </div>
      </div>
      <div
        style={{
          color: isHovered ? meta.accentColor : palette.textSecondary,
          fontFamily: typography.fontFamily,
          fontSize: typography.percentLabelPt,
          fontWeight: 500,
          opacity: textOpacity,
          letterSpacing: '0.03em',
          marginTop: 1,
          transition: 'color 0.15s ease',
        }}
      >
        {metricText}
      </div>
    </div>
  );
}
