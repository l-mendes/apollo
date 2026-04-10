export interface SelectionMonitorGeometry {
  positionX: number;
  positionY: number;
  width: number;
  height: number;
  scaleFactor: number;
}

export interface SelectionPayload {
  logical_x: number;
  logical_y: number;
  logical_width: number;
  logical_height: number;
  physical_x: number;
  physical_y: number;
  physical_width: number;
  physical_height: number;
  monitor_logical_x: number;
  monitor_logical_y: number;
  monitor_logical_width: number;
  monitor_logical_height: number;
  monitor_physical_x: number;
  monitor_physical_y: number;
  monitor_physical_width: number;
  monitor_physical_height: number;
}

interface BuildSelectionPayloadParams {
  startX: number;
  startY: number;
  endX: number;
  endY: number;
  startScreenX?: number;
  startScreenY?: number;
  endScreenX?: number;
  endScreenY?: number;
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
  startScreenX,
  startScreenY,
  endScreenX,
  endScreenY,
  monitor
}: BuildSelectionPayloadParams): SelectionPayload {
  const monitorLogicalX = Math.round(
    toLogicalOffset(monitor.positionX, monitor.scaleFactor)
  );
  const monitorLogicalY = Math.round(
    toLogicalOffset(monitor.positionY, monitor.scaleFactor)
  );
  const monitorLogicalWidth = Math.round(
    toLogicalOffset(monitor.width, monitor.scaleFactor)
  );
  const monitorLogicalHeight = Math.round(
    toLogicalOffset(monitor.height, monitor.scaleFactor)
  );
  const logicalStartX = Math.round(
    startScreenX ?? monitorLogicalX + startX
  );
  const logicalStartY = Math.round(
    startScreenY ?? monitorLogicalY + startY
  );
  const logicalEndX = Math.round(endScreenX ?? monitorLogicalX + endX);
  const logicalEndY = Math.round(endScreenY ?? monitorLogicalY + endY);
  const physicalStartX = Math.round(
    monitor.positionX + (logicalStartX - monitorLogicalX) * (monitor.scaleFactor || 1)
  );
  const physicalStartY = Math.round(
    monitor.positionY + (logicalStartY - monitorLogicalY) * (monitor.scaleFactor || 1)
  );
  const physicalEndX = Math.round(
    monitor.positionX + (logicalEndX - monitorLogicalX) * (monitor.scaleFactor || 1)
  );
  const physicalEndY = Math.round(
    monitor.positionY + (logicalEndY - monitorLogicalY) * (monitor.scaleFactor || 1)
  );

  return {
    logical_x: Math.min(logicalStartX, logicalEndX),
    logical_y: Math.min(logicalStartY, logicalEndY),
    logical_width: Math.abs(logicalEndX - logicalStartX),
    logical_height: Math.abs(logicalEndY - logicalStartY),
    physical_x: Math.min(physicalStartX, physicalEndX),
    physical_y: Math.min(physicalStartY, physicalEndY),
    physical_width: Math.abs(physicalEndX - physicalStartX),
    physical_height: Math.abs(physicalEndY - physicalStartY),
    monitor_logical_x: monitorLogicalX,
    monitor_logical_y: monitorLogicalY,
    monitor_logical_width: monitorLogicalWidth,
    monitor_logical_height: monitorLogicalHeight,
    monitor_physical_x: monitor.positionX,
    monitor_physical_y: monitor.positionY,
    monitor_physical_width: monitor.width,
    monitor_physical_height: monitor.height
  };
}
