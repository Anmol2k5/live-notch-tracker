import { describe, expect, it } from 'vitest';
import { bandFor } from './usageBand';

describe('bandFor', () => {
  it('maps the frame 21% cell to ample', () => {
    expect(bandFor(0.21)).toBe('ample');
  });
  it('maps the frame 52% cell to watch', () => {
    expect(bandFor(0.52)).toBe('watch');
  });
  it('maps the frame 73% cell to critical', () => {
    expect(bandFor(0.73)).toBe('critical');
  });
  it('treats the 50% boundary as watch (frame over prose spec)', () => {
    expect(bandFor(0.5)).toBe('watch');
  });
  it('treats the 70% boundary as critical (frame over prose spec)', () => {
    expect(bandFor(0.7)).toBe('critical');
  });
});
