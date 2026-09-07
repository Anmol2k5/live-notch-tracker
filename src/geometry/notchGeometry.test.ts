import { describe, expect, it } from 'vitest';
import { anchorNotch, panelHeightForCells, type WorkArea } from './notchGeometry';

const work: WorkArea = { x: 0, y: 0, width: 1920, height: 1040, scaleFactor: 1.25 };

describe('panelHeightForCells', () => {
  it('matches the recorded 401pt four-cell stack', () => {
    expect(panelHeightForCells(4)).toBeCloseTo(401, 0);
  });
});

describe('anchorNotch', () => {
  it('pins the right edge flush to the work area at 125% DPI', () => {
    const rect = anchorNotch(work, 3);
    expect(rect.x + rect.width).toBeCloseTo((work.x + work.width) / work.scaleFactor, 10);
  });
  it('lands every edge on whole physical pixels', () => {
    const rect = anchorNotch(work, 3);
    for (const edge of [rect.x, rect.y, rect.x + rect.width, rect.y + rect.height]) {
      expect(Math.abs(edge * work.scaleFactor - Math.round(edge * work.scaleFactor))).toBeLessThan(1e-9);
    }
  });
  it('centers vertically in the work area', () => {
    const rect = anchorNotch(work, 3);
    const workTop = work.y / work.scaleFactor;
    const workH = work.height / work.scaleFactor;
    expect(rect.y + rect.height / 2).toBeCloseTo(workTop + workH / 2, 6);
  });
});
