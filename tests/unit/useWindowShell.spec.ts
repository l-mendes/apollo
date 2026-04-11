import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  invokeMock: vi.fn(),
  emitToMock: vi.fn(),
  emitMock: vi.fn(),
  showMock: vi.fn(),
  unminimizeMock: vi.fn(),
  focusMock: vi.fn(),
  setPositionMock: vi.fn(),
  outerPositionMock: vi.fn(),
  outerSizeMock: vi.fn(),
  destroyMock: vi.fn(),
  onceMock: vi.fn(),
  listenMock: vi.fn(),
  onCloseRequestedMock: vi.fn(),
  hideMock: vi.fn(),
  setAlwaysOnTopMock: vi.fn(),
  setDecorationsMock: vi.fn(),
  setSkipTaskbarMock: vi.fn(),
  setShadowMock: vi.fn(),
  setSizeMock: vi.fn(),
  setMinSizeMock: vi.fn(),
  cursorPositionMock: vi.fn(),
  currentMonitorMock: vi.fn(),
  monitorFromPointMock: vi.fn(),
  getByLabelMock: vi.fn(),
  webviewGetByLabelMock: vi.fn(),
  createdWindows: [] as Array<Record<string, unknown>>
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invokeMock
}));

const currentWindowMock = {
  label: "tray",
  emitTo: mocks.emitToMock,
  listen: mocks.listenMock,
  onCloseRequested: mocks.onCloseRequestedMock,
  show: mocks.showMock,
  unminimize: mocks.unminimizeMock,
  setFocus: mocks.focusMock,
  hide: mocks.hideMock,
  setAlwaysOnTop: mocks.setAlwaysOnTopMock,
  setDecorations: mocks.setDecorationsMock,
  setSkipTaskbar: mocks.setSkipTaskbarMock,
  setShadow: mocks.setShadowMock,
  setSize: mocks.setSizeMock,
  setMinSize: mocks.setMinSizeMock
};

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => currentWindowMock,
  cursorPosition: mocks.cursorPositionMock,
  currentMonitor: mocks.currentMonitorMock,
  monitorFromPoint: mocks.monitorFromPointMock,
  LogicalSize: class LogicalSize {
    constructor(
      public width: number,
      public height: number
    ) {}
  }
}));

vi.mock("@tauri-apps/api/dpi", () => ({
  PhysicalPosition: class PhysicalPosition {
    constructor(
      public x: number,
      public y: number
    ) {}
  },
  PhysicalSize: class PhysicalSize {
    constructor(
      public width: number,
      public height: number
    ) {}
  }
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: class MockWebviewWindow {
    label: string;

    constructor(label: string, options?: Record<string, unknown>) {
      this.label = label;
      mocks.createdWindows.push({ label, options });
    }

    static getByLabel(label: string) {
      return mocks.webviewGetByLabelMock(label);
    }

    show = mocks.showMock;
    unminimize = mocks.unminimizeMock;
    setFocus = mocks.focusMock;
    setPosition = mocks.setPositionMock;
    setSize = mocks.setSizeMock;
    outerPosition = mocks.outerPositionMock;
    outerSize = mocks.outerSizeMock;
    destroy = mocks.destroyMock;
    emit = mocks.emitMock;
    once = mocks.onceMock;
  }
}));

import {
  APP_WINDOW_LABEL,
  APP_WINDOW_URL,
  NAVIGATE_SURFACE_EVENT,
  RESPONSE_WINDOW_LABEL,
  SURFACE_CHANGED_EVENT,
  emitSurfaceChanged,
  listenForAppCloseToHide,
  listenForSurfaceChanged,
  listenForSurfaceNavigation,
  openAppWindow,
  openResponseWindow,
  openSelectionWindow,
  revealCurrentWindow,
  syncAppWindowAppearance,
  syncTrayWindowAppearance
} from "@/composables/useWindowShell";

describe("useWindowShell", () => {
  beforeEach(() => {
    mocks.invokeMock.mockReset().mockResolvedValue(undefined);
    mocks.emitToMock.mockReset().mockResolvedValue(undefined);
    mocks.emitMock.mockReset().mockResolvedValue(undefined);
    mocks.showMock.mockReset().mockResolvedValue(undefined);
    mocks.unminimizeMock.mockReset().mockResolvedValue(undefined);
    mocks.focusMock.mockReset().mockResolvedValue(undefined);
    mocks.setPositionMock.mockReset().mockResolvedValue(undefined);
    mocks.outerPositionMock.mockReset().mockResolvedValue({ x: 1920, y: 0 });
    mocks.outerSizeMock
      .mockReset()
      .mockResolvedValue({ width: 2560, height: 1440 });
    mocks.destroyMock.mockReset().mockResolvedValue(undefined);
    mocks.onceMock
      .mockReset()
      .mockImplementation(async (eventName: string, handler: () => void) => {
        if (eventName === "tauri://created") {
          handler();
        }

        return () => {};
      });
    mocks.listenMock.mockReset().mockResolvedValue(() => {});
    mocks.onCloseRequestedMock.mockReset().mockResolvedValue(() => {});
    mocks.hideMock.mockReset().mockResolvedValue(undefined);
    mocks.setAlwaysOnTopMock.mockReset().mockResolvedValue(undefined);
    mocks.setDecorationsMock.mockReset().mockResolvedValue(undefined);
    mocks.setSkipTaskbarMock.mockReset().mockResolvedValue(undefined);
    mocks.setShadowMock.mockReset().mockResolvedValue(undefined);
    mocks.setSizeMock.mockReset().mockResolvedValue(undefined);
    mocks.setMinSizeMock.mockReset().mockResolvedValue(undefined);
    mocks.cursorPositionMock.mockReset().mockResolvedValue({ x: 2200, y: 180 });
    mocks.currentMonitorMock.mockReset().mockResolvedValue({
      position: { x: 0, y: 0 },
      size: { width: 1920, height: 1080 },
      scaleFactor: 1
    });
    mocks.monitorFromPointMock.mockReset().mockResolvedValue({
      position: { x: 1920, y: 0 },
      size: { width: 2560, height: 1440 },
      scaleFactor: 1
    });
    mocks.webviewGetByLabelMock.mockReset().mockResolvedValue({
      show: mocks.showMock,
      unminimize: mocks.unminimizeMock,
      setFocus: mocks.focusMock,
      setPosition: mocks.setPositionMock,
      setSize: mocks.setSizeMock,
      setAlwaysOnTop: mocks.setAlwaysOnTopMock,
      outerPosition: mocks.outerPositionMock,
      outerSize: mocks.outerSizeMock,
      destroy: mocks.destroyMock,
      emit: mocks.emitMock
    });
    mocks.createdWindows.length = 0;
  });

  it("opens and focuses the app window while forwarding the requested surface", async () => {
    await openAppWindow("history");

    expect(mocks.webviewGetByLabelMock).toHaveBeenCalledWith(APP_WINDOW_LABEL);
    expect(mocks.showMock).toHaveBeenCalled();
    expect(mocks.unminimizeMock).toHaveBeenCalled();
    expect(mocks.emitMock).toHaveBeenCalledWith(NAVIGATE_SURFACE_EVENT, {
      surface: "history"
    });
    expect(mocks.focusMock).toHaveBeenCalled();
  });

  it("creates the app window when it is not available yet", async () => {
    mocks.webviewGetByLabelMock.mockResolvedValueOnce(null);

    await openAppWindow("settings");

    expect(mocks.createdWindows).toHaveLength(1);
    expect(mocks.createdWindows[0]).toEqual(
      expect.objectContaining({
        label: APP_WINDOW_LABEL,
        options: expect.objectContaining({
          url: expect.stringContaining(APP_WINDOW_URL),
          width: 800,
          height: 640
        })
      })
    );
    expect(mocks.emitMock).toHaveBeenCalledWith(NAVIGATE_SURFACE_EVENT, {
      surface: "settings"
    });
  });

  it("opens an existing response window without forcing always-on-top", async () => {
    await openResponseWindow();

    expect(mocks.webviewGetByLabelMock).toHaveBeenCalledWith(
      RESPONSE_WINDOW_LABEL
    );
    expect(mocks.setAlwaysOnTopMock).not.toHaveBeenCalled();
    expect(mocks.showMock).toHaveBeenCalled();
    expect(mocks.focusMock).toHaveBeenCalled();
  });

  it("creates the response window without always-on-top", async () => {
    mocks.webviewGetByLabelMock.mockResolvedValueOnce(null);

    await openResponseWindow();

    expect(mocks.createdWindows).toHaveLength(1);
    expect(mocks.createdWindows[0]).toEqual(
      expect.objectContaining({
        label: RESPONSE_WINDOW_LABEL,
        options: expect.objectContaining({
          decorations: false,
          transparent: true,
          skipTaskbar: true
        })
      })
    );
  });

  it("broadcasts surface changes back to the tray window", async () => {
    await emitSurfaceChanged("settings");

    expect(mocks.emitToMock).toHaveBeenCalledWith(
      "tray",
      SURFACE_CHANGED_EVENT,
      {
        surface: "settings"
      }
    );
  });

  it("registers navigation and surface listeners on the current window", async () => {
    const navigationHandler = vi.fn();
    const surfaceHandler = vi.fn();

    await listenForSurfaceNavigation(navigationHandler);
    await listenForSurfaceChanged(surfaceHandler);

    expect(mocks.listenMock).toHaveBeenNthCalledWith(
      1,
      NAVIGATE_SURFACE_EVENT,
      expect.any(Function)
    );
    expect(mocks.listenMock).toHaveBeenNthCalledWith(
      2,
      SURFACE_CHANGED_EVENT,
      expect.any(Function)
    );
  });

  it("prepares tray and app window appearance through the window API", async () => {
    await syncTrayWindowAppearance();
    await syncAppWindowAppearance();

    expect(mocks.setAlwaysOnTopMock).toHaveBeenCalled();
    expect(mocks.setDecorationsMock).toHaveBeenCalled();
    expect(mocks.setSkipTaskbarMock).toHaveBeenCalled();
    expect(mocks.setSizeMock).toHaveBeenCalled();
    expect(mocks.setMinSizeMock).toHaveBeenCalled();
  });

  it("reveals and focuses the current app window", async () => {
    await revealCurrentWindow();

    expect(mocks.showMock).toHaveBeenCalled();
    expect(mocks.unminimizeMock).toHaveBeenCalled();
    expect(mocks.focusMock).toHaveBeenCalled();
  });

  it("intercepts close requests to hide the app window instead of destroying it", async () => {
    await listenForAppCloseToHide();

    expect(mocks.onCloseRequestedMock).toHaveBeenCalledWith(
      expect.any(Function)
    );
  });

  it("positions the selection window on the monitor where the cursor is active", async () => {
    mocks.webviewGetByLabelMock
      .mockResolvedValueOnce({
        show: mocks.showMock,
        unminimize: mocks.unminimizeMock,
        setFocus: mocks.focusMock,
        setPosition: mocks.setPositionMock,
        setSize: mocks.setSizeMock,
        outerPosition: mocks.outerPositionMock,
        outerSize: mocks.outerSizeMock,
        destroy: mocks.destroyMock,
        emit: mocks.emitMock
      })
      .mockResolvedValueOnce(null);

    await openSelectionWindow();

    expect(mocks.destroyMock).toHaveBeenCalled();
    expect(mocks.monitorFromPointMock).toHaveBeenCalledWith(2200, 180);
    expect(mocks.createdWindows[0]).toEqual(
      expect.objectContaining({
        label: "selection",
        options: expect.objectContaining({
          x: 1920,
          y: 0,
          width: 2560,
          height: 1440
        })
      })
    );
    expect(mocks.setPositionMock).toHaveBeenCalledWith(
      expect.objectContaining({ x: 1920, y: 0 })
    );
    expect(mocks.setSizeMock).toHaveBeenCalledWith(
      expect.objectContaining({ width: 2560, height: 1440 })
    );
    expect(mocks.focusMock).toHaveBeenCalled();
  });
});
