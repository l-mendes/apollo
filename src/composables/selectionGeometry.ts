export interface SelectionMonitorGeometry {
  positionX: number;
  positionY: number;
  scaleFactor: number;
}

export interface SelectionPayload {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface BuildSelectionPayloadParams {
  startX: number;
  startY: number;
  endX: number;
  endY: number;
  monitor: SelectionMonitorGeometry;
}

function toLogicalOffset(position: number, scaleFactor: number): number {
  return position / (scaleFactor || 1);
}

export function buildSelectionPayload({
  startX,
  startY,
  endX,
  endY,
  monitor
}: BuildSelectionPayloadParams): SelectionPayload {
  const x = Math.min(startX, endX);
  const y = Math.min(startY, endY);
  const width = Math.abs(endX - startX);
  const height = Math.abs(endY - startY);

  return {
    x: Math.round(toLogicalOffset(monitor.positionX, monitor.scaleFactor) + x),
    y: Math.round(toLogicalOffset(monitor.positionY, monitor.scaleFactor) + y),
    width: Math.round(width),
    height: Math.round(height)
  };
}
