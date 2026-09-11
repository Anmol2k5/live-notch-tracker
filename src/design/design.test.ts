import { describe, expect, it } from 'vitest';
import { px, scale } from './design';

describe('design scale', () => {
  it('anchors the 44pt ring to the 117px frame measurement', () => {
    expect(scale).toBeCloseTo(44 / 117, 10);
    expect(px(117)).toBeCloseTo(44, 10);
  });
  it('maps the 70pt body depth to ~186 frame px', () => {
    expect(px(186)).toBeCloseTo(70, 0);
  });
});
