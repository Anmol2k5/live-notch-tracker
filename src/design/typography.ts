import { px } from './design';

export const typography = {
  fontFamily: "'Inter', 'Segoe UI', system-ui, sans-serif",
  percentLabelPt: Math.round(px(40)),
  providerInitialPt: Math.round(px(42)),
  tooltipHeaderPt: px(44),
  tooltipRowPt: px(34),
} satisfies Record<string, string | number>;
