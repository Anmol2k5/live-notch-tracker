/**
 * Strict TypeScript definitions for the provider usage models.
 * Replaces the any/unknown mappings with discriminated unions and exact typing.
 */

export type ProviderStatus =
  | { kind: 'ok' }
  | { kind: 'stale' }
  | { kind: 'backoff' }
  | { kind: 'needsAuth' }
  | { kind: 'error' }
  | { kind: 'absent' }
  | { kind: 'none' };

export type ProviderStatusKind = ProviderStatus['kind'];

export interface LimitWindow {
  id: string;
  label: string;
  used: number; // 0.0 to 1.0 fraction used
  resets_at: number | null; // epoch ms or null
  count: number | null; // null for percentage-based windows, number for pure counts
  derived: boolean; // whether the limit is derived locally (e.g. counting transcript logs)
}

/**
 * The snapshot received from the Rust backend via Tauri events.
 * The Rust backend uses a string for status, which we parse into a discriminated union.
 */
export interface RawUsageSnapshot {
  status: string; // From Rust
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

/**
 * Type guard for the status string from Rust.
 */
export function parseStatus(rawStatus: string): ProviderStatus {
  switch (rawStatus) {
    case 'ok':
    case 'stale':
    case 'backoff':
    case 'needsAuth':
    case 'error':
    case 'absent':
    case 'none':
      return { kind: rawStatus };
    default:
      // Default to error if the backend sends an unknown status to maintain boundary discipline
      return { kind: 'error' };
  }
}

/**
 * Transforms the raw Rust snapshot into our strict frontend type.
 */
export function parseUsageSnapshot(raw: RawUsageSnapshot): UsageSnapshot {
  return {
    ...raw,
    status: parseStatus(raw.status),
  };
}
