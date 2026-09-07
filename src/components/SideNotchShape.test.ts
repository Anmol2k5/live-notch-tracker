import { describe, expect, it } from 'vitest';
import { notchPath } from './SideNotchShape';

describe('notchPath', () => {
  it('starts and ends flush on the screen edge', () => {
    const { top, bottom, edgeX } = notchPath(70, 301);
    expect(top[0]).toBeCloseTo(edgeX, 10);
    expect(bottom[0]).toBeCloseTo(edgeX, 10);
  });
  it('is vertically symmetric', () => {
    const { top, bottom } = notchPath(70, 301);
    expect(top[1] + bottom[1]).toBeCloseTo(301, 6);
  });
  it('never draws past the screen edge', () => {
    const { samples, edgeX } = notchPath(70, 301);
    for (const [x] of samples) {
      expect(x).toBeLessThanOrEqual(edgeX + 1e-9);
    }
  });
});
