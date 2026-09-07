export type NotchEdge = 'right';

export interface WorkArea {
  x: number;
  y: number;
  width: number;
  height: number;
  scaleFactor: number;
}

export interface PanelRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export const BODY_DEPTH_PT = 70;
export const CELL_PITCH_PT = 401 / 4;

export function panelHeightForCells(cellCount: number): number {
  return cellCount * CELL_PITCH_PT;
}

function roundToPhysical(logical: number, scaleFactor: number): number {
  return Math.round(logical * scaleFactor) / scaleFactor;
}

export function anchorNotch(work: WorkArea, cellCount: number, edge: NotchEdge = 'right'): PanelRect {
  if (edge !== 'right') {
    throw new Error(`unsupported edge in fixture slice: ${edge}`);
  }
  const toLogical = (physical: number): number => physical / work.scaleFactor;
  const workRight = toLogical(work.x + work.width);
  const workTop = toLogical(work.y);
  const workHeight = toLogical(work.height);
  const width = roundToPhysical(BODY_DEPTH_PT, work.scaleFactor);
  const height = roundToPhysical(panelHeightForCells(cellCount), work.scaleFactor);
  const x = roundToPhysical(workRight - width, work.scaleFactor);
  const y = roundToPhysical(workTop + (workHeight - height) / 2, work.scaleFactor);
  return { x, y, width, height };
}
