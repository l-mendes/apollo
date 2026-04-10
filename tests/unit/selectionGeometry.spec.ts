import { describe, expect, it } from "vitest";

import { buildSelectionPayload } from "@/composables/selectionGeometry";

describe("selectionGeometry", () => {
  it("keeps selection coordinates in logical pixels for secondary hi-dpi monitors", () => {
    const payload = buildSelectionPayload({
      startX: 100,
      startY: 40,
      endX: 420,
      endY: 240,
      monitor: {
        positionX: 3840,
        positionY: 0,
        scaleFactor: 2
      }
    });

    expect(payload).toEqual({
      x: 2020,
      y: 40,
      width: 320,
      height: 200
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
        scaleFactor: 1
      }
    });

    expect(payload).toEqual({
      x: -1910,
      y: 20,
      width: 40,
      height: 40
    });
  });
});
