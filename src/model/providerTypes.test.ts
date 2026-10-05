import { describe, expect, it } from 'vitest';
import {
  formatMetricLabel,
  metricRingFraction,
  parseStatus,
  parseUsageSnapshot,
  PROVIDER_METADATA,
  resolveMetric,
  LimitWindow,
  UsageMetric,
} from './providerTypes';

describe('Provider domain model and metric discipline', () => {
  it('guarantees unique, unambiguous provider shortCodes (no three Cs)', () => {
    const codes = [
      PROVIDER_METADATA.claude.shortCode,
      PROVIDER_METADATA.cursor.shortCode,
      PROVIDER_METADATA.codex.shortCode,
      PROVIDER_METADATA.antigravity.shortCode,
    ];
    // None should simply be "C"
    expect(codes).not.toContain('C');
    expect(codes).toEqual(['CL', 'CU', 'CX', 'AG']);
    // All unique
    const unique = new Set(codes);
    expect(unique.size).toBe(4);
  });

  describe('resolveMetric', () => {
    it('resolves percentage windows accurately', () => {
      const win: LimitWindow = {
        id: 'session',
        label: 'Session',
        used: 0.73,
        resets_at: 1720000000,
        count: null,
        derived: false,
      };
      const metric = resolveMetric(win);
      expect(metric.kind).toBe('percentage');
      if (metric.kind === 'percentage') {
        expect(metric.usedPercent).toBe(73);
        expect(metric.resetsAt).toBe(1720000000);
      }
    });

    it('resolves Antigravity derived request counts as kind count without limit', () => {
      const win: LimitWindow = {
        id: 'requests',
        label: 'Requests today',
        used: 0.0,
        resets_at: null,
        count: 42,
        derived: true,
      };
      const metric = resolveMetric(win);
      expect(metric.kind).toBe('count');
      if (metric.kind === 'count') {
        expect(metric.used).toBe(42);
        expect(metric.limit).toBeNull();
        expect(metric.derived).toBe(true);
      }
    });

    it('prefers explicit metric when already provided', () => {
      const win: LimitWindow = {
        id: 'tokens',
        label: 'Token Usage',
        used: 0.5,
        resets_at: null,
        metric: { kind: 'tokens', used: 1500000, limit: 3000000 },
      };
      const metric = resolveMetric(win);
      expect(metric.kind).toBe('tokens');
      if (metric.kind === 'tokens') {
        expect(metric.used).toBe(1500000);
        expect(metric.limit).toBe(3000000);
      }
    });
  });

  describe('formatMetricLabel', () => {
    it('formats percentage with percent sign', () => {
      const metric: UsageMetric = { kind: 'percentage', usedPercent: 73 };
      expect(formatMetricLabel(metric)).toBe('73%');
    });

    it('NEVER formats pure counts as percentage', () => {
      const countWithoutLimit: UsageMetric = { kind: 'count', used: 42, limit: null, derived: true };
      const formatted = formatMetricLabel(countWithoutLimit);
      expect(formatted).toBe('~42');
      expect(formatted).not.toContain('%');

      const nonDerivedCount: UsageMetric = { kind: 'count', used: 42, limit: null, derived: false };
      expect(formatMetricLabel(nonDerivedCount)).toBe('42');
      expect(formatMetricLabel(nonDerivedCount)).not.toContain('%');
    });

    it('formats count with limit as fraction', () => {
      const metric: UsageMetric = { kind: 'count', used: 42, limit: 100 };
      expect(formatMetricLabel(metric)).toBe('42/100');
    });

    it('formats token metrics compactly', () => {
      expect(formatMetricLabel({ kind: 'tokens', used: 1500000 })).toBe('1.5M');
      expect(formatMetricLabel({ kind: 'tokens', used: 45000 })).toBe('45k');
      expect(formatMetricLabel({ kind: 'tokens', used: 850 })).toBe('850');
    });
  });

  describe('metricRingFraction', () => {
    it('returns clamped fraction for percentage', () => {
      expect(metricRingFraction({ kind: 'percentage', usedPercent: 73 })).toBeCloseTo(0.73, 2);
      expect(metricRingFraction({ kind: 'percentage', usedPercent: 150 })).toBe(1);
      expect(metricRingFraction({ kind: 'percentage', usedPercent: -10 })).toBe(0);
    });

    it('returns null for count without limit so ring draws only track', () => {
      expect(metricRingFraction({ kind: 'count', used: 42, limit: null })).toBeNull();
    });

    it('returns fraction for count with limit', () => {
      expect(metricRingFraction({ kind: 'count', used: 25, limit: 100 })).toBeCloseTo(0.25, 2);
    });
  });

  describe('parseStatus and parseUsageSnapshot', () => {
    it('parses valid status kinds', () => {
      expect(parseStatus('ok')).toEqual({ kind: 'ok' });
      expect(parseStatus('stale')).toEqual({ kind: 'stale' });
      expect(parseStatus('needsAuth')).toEqual({ kind: 'needsAuth' });
      expect(parseStatus('backoff')).toEqual({ kind: 'backoff' });
      expect(parseStatus('absent')).toEqual({ kind: 'absent' });
    });

    it('gracefully handles unknown status as error', () => {
      const parsed = parseStatus('unexpected_server_state');
      expect(parsed.kind).toBe('error');
    });

    it('parses raw snapshot and ensures windows have resolved metrics', () => {
      const raw = {
        status: 'ok',
        windows: [
          { id: '1', label: 'L1', used: 0.45, resets_at: 1000 },
          { id: '2', label: 'L2', used: 0, resets_at: null, count: 88, derived: true },
        ],
        fetched_at: 12345,
        note: 'test note',
        backoff_until: 0,
      };
      const snap = parseUsageSnapshot(raw);
      expect(snap.status).toEqual({ kind: 'ok' });
      expect(snap.windows[0].metric).toEqual({ kind: 'percentage', usedPercent: 45, resetsAt: 1000 });
      expect(snap.windows[1].metric).toEqual({ kind: 'count', used: 88, limit: null, resetsAt: null, derived: true });
    });
  });
});
