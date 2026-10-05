/**
 * Strict TypeScript definitions for the provider usage models.
 * Normalized provider domain model preventing arbitrary metrics from being treated as fake percentages.
 */

export type ProviderId = 'claude' | 'cursor' | 'codex' | 'antigravity';

export type ProviderStatus =
  | { kind: 'ok' }
  | { kind: 'stale' }
  | { kind: 'backoff' }
  | { kind: 'needsAuth' }
  | { kind: 'error'; message?: string }
  | { kind: 'absent' }
  | { kind: 'none' };

export type ProviderStatusKind = ProviderStatus['kind'];

export type UsageMetric =
  | {
      kind: 'percentage';
      usedPercent: number; // 0 to 100
      resetsAt?: number | null; // epoch ms
    }
  | {
      kind: 'count';
      used: number;
      limit?: number | null; // null if no published denominator
      resetsAt?: number | null;
      derived?: boolean;
    }
  | {
      kind: 'tokens';
      used: number;
      limit?: number | null;
      resetsAt?: number | null;
    };

export interface LimitWindow {
  id: string;
  label: string;
  used: number; // 0.0 to 1.0 fraction used
  resets_at: number | null;
  count?: number | null;
  derived?: boolean;
  metric?: UsageMetric | null;
}

/**
 * Raw snapshot from Rust.
 */
export interface RawUsageSnapshot {
  status: string;
  windows: LimitWindow[];
  fetched_at: number;
  note: string;
  backoff_until: number;
}

export interface UsageSnapshot {
  status: ProviderStatus;
  windows: LimitWindow[];
  fetched_at: number;
  note: string;
  backoff_until: number;
}

export interface ProviderMeta {
  id: ProviderId;
  displayName: string;
  shortCode: string; // CL, CU, CX, AG (unambiguous, never just 'C')
  accentColor: string;
}

export const PROVIDER_METADATA: Record<ProviderId, ProviderMeta> = {
  claude: {
    id: 'claude',
    displayName: 'Claude Code',
    shortCode: 'CL',
    accentColor: '#D97706',
  },
  cursor: {
    id: 'cursor',
    displayName: 'Cursor',
    shortCode: 'CU',
    accentColor: '#3B82F6',
  },
  codex: {
    id: 'codex',
    displayName: 'Codex / ChatGPT',
    shortCode: 'CX',
    accentColor: '#10B981',
  },
  antigravity: {
    id: 'antigravity',
    displayName: 'Antigravity',
    shortCode: 'AG',
    accentColor: '#8B5CF6',
  },
};

/**
 * Type guard for the status string from Rust.
 */
export function parseStatus(rawStatus: string): ProviderStatus {
  switch (rawStatus) {
    case 'ok':
    case 'stale':
    case 'backoff':
    case 'needsAuth':
    case 'absent':
    case 'none':
      return { kind: rawStatus };
    case 'error':
      return { kind: 'error' };
    default:
      return { kind: 'error', message: rawStatus };
  }
}

/**
 * Transforms the raw Rust snapshot into our strict frontend type.
 */
export function parseUsageSnapshot(raw: RawUsageSnapshot): UsageSnapshot {
  return {
    ...raw,
    status: parseStatus(raw.status),
    windows: (raw.windows || []).map((w) => ({
      ...w,
      metric: resolveMetric(w),
    })),
  };
}

/**
 * Resolves the primary UsageMetric for a LimitWindow with type safety.
 * Crucial rule: a count window without a limit is NEVER converted into a fake percentage!
 */
export function resolveMetric(window: LimitWindow): UsageMetric {
  if (window.metric) {
    return window.metric;
  }
  if (window.count != null) {
    return {
      kind: 'count',
      used: window.count,
      limit: null,
      resetsAt: window.resets_at,
      derived: window.derived,
    };
  }
  return {
    kind: 'percentage',
    usedPercent: Math.round(window.used * 100),
    resetsAt: window.resets_at,
  };
}

/**
 * Formats a metric for display in the notch cell.
 * Strict guarantee: Count without denominator displays e.g. "~42" or "42", NEVER "42%"!
 */
export function formatMetricLabel(metric: UsageMetric): string {
  switch (metric.kind) {
    case 'percentage':
      return `${Math.round(metric.usedPercent)}%`;
    case 'count':
      if (metric.limit != null && metric.limit > 0) {
        return `${metric.used}/${metric.limit}`;
      }
      return metric.derived ? `~${metric.used}` : `${metric.used}`;
    case 'tokens':
      if (metric.used >= 1_000_000) {
        return `${(metric.used / 1_000_000).toFixed(1)}M`;
      }
      if (metric.used >= 1_000) {
        return `${(metric.used / 1_000).toFixed(0)}k`;
      }
      return `${metric.used}`;
  }
}

/**
 * Computes the ring arc fraction (0.0 to 1.0) for a metric.
 * For counts without a published limit, returns null so the ring draws only its track!
 */
export function metricRingFraction(metric: UsageMetric): number | null {
  switch (metric.kind) {
    case 'percentage':
      return Math.min(1, Math.max(0, metric.usedPercent / 100));
    case 'count':
      if (metric.limit != null && metric.limit > 0) {
        return Math.min(1, Math.max(0, metric.used / metric.limit));
      }
      return null; // No published denominator: draw only track, no fake arc!
    case 'tokens':
      if (metric.limit != null && metric.limit > 0) {
        return Math.min(1, Math.max(0, metric.used / metric.limit));
      }
      return null;
  }
}
