export const scale = 44 / 117;

export function px(pixels: number): number {
  return pixels * scale;
}

export function roundToPhysical(logical: number, scaleFactor: number): number {
  return Math.round(logical * scaleFactor) / scaleFactor;
}
