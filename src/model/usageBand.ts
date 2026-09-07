import { palette } from '../design/palette';

export type UsageBand = 'ample' | 'watch' | 'critical';

export function bandFor(usedFraction: number): UsageBand {
  if (usedFraction < 0.5) {
    return 'ample';
  }
  if (usedFraction < 0.7) {
    return 'watch';
  }
  return 'critical';
}

export function colorFor(band: UsageBand): string {
  switch (band) {
    case 'ample':
      return palette.ample;
    case 'watch':
      return palette.watch;
    case 'critical':
      return palette.critical;
  }
}
