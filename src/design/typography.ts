import { px } from './design';

export const typography = {
  percentLabelPt: Math.round(px(40)),
  providerInitialPt: Math.round(px(42)),
  tooltipHeaderPt: px(44),
  tooltipRowPt: px(34),
} satisfies Record<string, number>;
