import { describe, expect, it } from "vitest";

import { buildSelectionPayload } from "@/composables/selectionGeometry";

describe("selectionGeometry", () => {
  it("returns logical and physical selection coordinates for secondary hi-dpi monitors", () => {
    const payload = buildSelectionPayload({
      startX: 100,
      startY: 40,
      endX: 420,
      endY: 240,
      monitor: {
        positionX: 3840,
        positionY: 0,
        width: 3840,
        height: 2160,
        scaleFactor: 2
      }
    });

    expect(payload).toEqual({
      logical_x: 2020,
      logical_y: 40,
      logical_width: 320,
      logical_height: 200,
      physical_x: 4040,
      physical_y: 80,
      physical_width: 640,
      physical_height: 400,
      monitor_logical_x: 1920,
      monitor_logical_y: 0,
      monitor_logical_width: 1920,
      monitor_logical_height: 1080,
      monitor_physical_x: 3840,
      monitor_physical_y: 0,
      monitor_physical_width: 3840,
      monitor_physical_height: 2160
    });
  });

  it("supports monitors positioned with negative offsets", () => {
    const payload = buildSelectionPayload({
      startX: 50,
      startY: 60,
      endX: 10,
      endY: 20,
      monitor: {
        positionX: -1920,
        positionY: 0,
        width: 1920,
        height: 1080,
        scaleFactor: 1
      }
    });

    expect(payload).toEqual({
      logical_x: -1910,
      logical_y: 20,
      logical_width: 40,
      logical_height: 40,
      physical_x: -1910,
      physical_y: 20,
      physical_width: 40,
      physical_height: 40,
      monitor_logical_x: -1920,
      monitor_logical_y: 0,
      monitor_logical_width: 1920,
      monitor_logical_height: 1080,
      monitor_physical_x: -1920,
      monitor_physical_y: 0,
      monitor_physical_width: 1920,
      monitor_physical_height: 1080
    });
  });

  it("prefers global screen coordinates when available to avoid viewport drift", () => {
    const payload = buildSelectionPayload({
      startX: 94,
      startY: 34,
      endX: 414,
      endY: 234,
      startScreenX: 2020,
      startScreenY: 40,
      endScreenX: 2340,
      endScreenY: 240,
      monitor: {
        positionX: 3840,
        positionY: 0,
        width: 3840,
        height: 2160,
        scaleFactor: 2
      }
    });

    expect(payload).toEqual({
      logical_x: 2020,
      logical_y: 40,
      logical_width: 320,
      logical_height: 200,
      physical_x: 4040,
      physical_y: 80,
      physical_width: 640,
      physical_height: 400,
      monitor_logical_x: 1920,
      monitor_logical_y: 0,
      monitor_logical_width: 1920,
      monitor_logical_height: 1080,
      monitor_physical_x: 3840,
      monitor_physical_y: 0,
      monitor_physical_width: 3840,
      monitor_physical_height: 2160
    });
  });
});
