import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { invoke } from "@tauri-apps/api/core";
import {
  LogicalSize,
  cursorPosition,
  currentMonitor,
  getCurrentWindow,
  monitorFromPoint
} from "@tauri-apps/api/window";

import type {
  ConversationMessage,
  NormalizedResponse,
  ProviderKind,
  ReasoningEffort,
  SurfaceId
} from "@/composables/useApolloDesktop";

export const TRAY_WINDOW_LABEL = "tray";
export const APP_WINDOW_LABEL = "app";
export const NAVIGATE_SURFACE_EVENT = "apollo:navigate-surface";
export const SURFACE_CHANGED_EVENT = "apollo:surface-changed";
export const APP_WINDOW_URL = "index.html?window=app";
const TRAY_WINDOW_SIZE = new LogicalSize(300, 48);

export type AppSurface = Exclude<SurfaceId, "none">;

type Unlisten = () => void;

interface NavigateSurfacePayload {
  surface: AppSurface;
}

interface SurfaceChangedPayload {
  surface: SurfaceId;
}

interface CursorMonitorPlacement {
  logicalX: number;
  logicalY: number;
  logicalWidth: number;
  logicalHeight: number;
  physicalX: number;
  physicalY: number;
  physicalWidth: number;
  physicalHeight: number;
}

async function logWindowDebug(
  source: string,
  event: string,
  data: Record<string, unknown>
): Promise<void> {
  try {
    console.info(`[apollo:${source}] ${event}`, data);
    await invoke("log_debug_window_event", {
      payload: {
        source,
        event,
        data
      }
    });
  } catch {
    // noop in web mode
  }
}

function resolveAppWindowUrl(surface?: AppSurface): string {
  const url = new URL(APP_WINDOW_URL, window.location.origin);

  if (surface) {
    url.searchParams.set("surface", surface);
  }

  return `${url.pathname}${url.search}`;
}

async function waitForWebviewWindow(window: WebviewWindow): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    const resolveOnce = () => {
      void logWindowDebug("selection", "window-created-event", {
        label: window.label
      });
      resolve();
    };
    const rejectOnce = (event: { payload?: unknown }) => {
      void logWindowDebug("selection", "window-created-error", {
        label: window.label,
        payload:
          typeof event.payload === "string" ? event.payload : "unknown-error"
      });
      reject(
        new Error(
          typeof event.payload === "string"
            ? event.payload
            : "failed to create app window"
        )
      );
    };

    void window.once("tauri://created", resolveOnce);
    void window.once("tauri://error", rejectOnce);
  });
}

function toLogicalPixels(value: number, scaleFactor: number): number {
  return Math.round(value / (scaleFactor || 1));
}

async function resolveCursorMonitorPlacement(): Promise<CursorMonitorPlacement | null> {
  try {
    const cursor = await cursorPosition().catch(() => null);
    const monitor = cursor
      ? await monitorFromPoint(cursor.x, cursor.y).catch(() => null)
      : await currentMonitor().catch(() => null);

    if (!monitor) {
      await logWindowDebug("selection", "cursor-monitor-missing", {
        cursor
      });
      return null;
    }

    const placement = {
      logicalX: toLogicalPixels(monitor.position.x, monitor.scaleFactor),
      logicalY: toLogicalPixels(monitor.position.y, monitor.scaleFactor),
      logicalWidth: toLogicalPixels(monitor.size.width, monitor.scaleFactor),
      logicalHeight: toLogicalPixels(monitor.size.height, monitor.scaleFactor),
      physicalX: monitor.position.x,
      physicalY: monitor.position.y,
      physicalWidth: monitor.size.width,
      physicalHeight: monitor.size.height
    };

    await logWindowDebug("selection", "cursor-monitor-resolved", {
      cursor,
      monitorPosition: monitor.position,
      monitorSize: monitor.size,
      monitorScaleFactor: monitor.scaleFactor,
      placement
    });

    return placement;
  } catch {
    await logWindowDebug("selection", "cursor-monitor-error", {});
    return null;
  }
}

async function placeSelectionWindowOnCursorMonitor(
  window: WebviewWindow
): Promise<void> {
  try {
    const placement = await resolveCursorMonitorPlacement();

    if (!placement) {
      return;
    }

    const { PhysicalPosition, PhysicalSize } =
      await import("@tauri-apps/api/dpi");

    await window.setPosition(
      new PhysicalPosition(placement.physicalX, placement.physicalY)
    );
    await window.setSize(
      new PhysicalSize(placement.physicalWidth, placement.physicalHeight)
    );
    await logWindowDebug("selection", "window-repositioned", {
      label: window.label,
      placement
    });
  } catch {
    // noop in web mode
  }
}

async function resolveAppWindow(surface: AppSurface): Promise<WebviewWindow> {
  const existingWindow = await WebviewWindow.getByLabel(APP_WINDOW_LABEL);

  if (existingWindow) {
    return existingWindow;
  }

  const createdWindow = new WebviewWindow(APP_WINDOW_LABEL, {
    title: "Apollo",
    url: resolveAppWindowUrl(surface),
    width: 800,
    height: 640,
    minWidth: 800,
    minHeight: 640,
    resizable: true,
    decorations: true,
    transparent: false,
    alwaysOnTop: false,
    skipTaskbar: false,
    visible: true,
    center: true
  });

  await waitForWebviewWindow(createdWindow);

  return createdWindow;
}

export function currentWindowLabel(): string {
  try {
    return getCurrentWindow().label;
  } catch {
    return APP_WINDOW_LABEL;
  }
}

export async function syncTrayWindowAppearance(): Promise<void> {
  try {
    const window = getCurrentWindow();

    // Clear the minimum size first so GTK/OS does not block a resize below its
    // internal default (~200 px). Setting it before setSize avoids a race where
    // setMinSize wins and clamps the height back up.
    await window.setMinSize(null);
    await Promise.allSettled([
      window.setSize(TRAY_WINDOW_SIZE),
      window.setAlwaysOnTop(true),
      window.setDecorations(false),
      window.setSkipTaskbar(true),
      window.setShadow(false)
    ]);
    // Re-apply the minimum only after the window has been sized correctly.
    await window.setMinSize(TRAY_WINDOW_SIZE);
  } catch {
    // noop in web mode
  }
}

export async function syncAppWindowAppearance(): Promise<void> {
  try {
    const window = getCurrentWindow();
    const defaultSize = new LogicalSize(800, 640);

    await Promise.allSettled([
      window.setDecorations(true),
      window.setAlwaysOnTop(false),
      window.setSkipTaskbar(false),
      window.setSize(defaultSize),
      window.setMinSize(defaultSize)
    ]);
  } catch {
    // noop in web mode
  }
}

export async function openAppWindow(surface: AppSurface): Promise<void> {
  try {
    const appWindow = await resolveAppWindow(surface);

    await appWindow.show();
    await appWindow.unminimize();
    await appWindow.emit<NavigateSurfacePayload>(NAVIGATE_SURFACE_EVENT, {
      surface
    });
    await appWindow.setFocus();
  } catch {
    // noop in web mode
  }
}

export async function emitSurfaceChanged(surface: SurfaceId): Promise<void> {
  try {
    await getCurrentWindow().emitTo<SurfaceChangedPayload>(
      TRAY_WINDOW_LABEL,
      SURFACE_CHANGED_EVENT,
      {
        surface
      }
    );
  } catch {
    // noop in web mode
  }
}

export async function hideCurrentWindow(): Promise<void> {
  try {
    await getCurrentWindow().hide();
  } catch {
    // noop in web mode
  }
}

export async function listenForSurfaceNavigation(
  handler: (surface: AppSurface) => void
): Promise<Unlisten> {
  try {
    return await getCurrentWindow().listen<NavigateSurfacePayload>(
      NAVIGATE_SURFACE_EVENT,
      ({ payload }) => handler(payload.surface)
    );
  } catch {
    return () => {};
  }
}

export async function listenForSurfaceChanged(
  handler: (surface: SurfaceId) => void
): Promise<Unlisten> {
  try {
    return await getCurrentWindow().listen<SurfaceChangedPayload>(
      SURFACE_CHANGED_EVENT,
      ({ payload }) => handler(payload.surface)
    );
  } catch {
    return () => {};
  }
}

export async function listenForAppCloseToHide(
  handler?: () => void
): Promise<Unlisten> {
  try {
    const window = getCurrentWindow();

    return await window.onCloseRequested(async (event) => {
      event.preventDefault();
      await emitSurfaceChanged("none");
      await window.hide();
      handler?.();
    });
  } catch {
    return () => {};
  }
}

export const PREVIEW_WINDOW_LABEL = "preview";
export const RESPONSE_WINDOW_LABEL = "response";
export const SELECTION_WINDOW_LABEL = "selection";

export const OCR_RESULT_EVENT = "apollo:ocr-result";
export const SHORTCUT_ACTION_EVENT = "apollo:shortcut-action";
export const PREVIEW_UPDATE_EVENT = "apollo:preview-update";
export const RESPONSE_UPDATE_EVENT = "apollo:response-update";
export const START_AREA_CAPTURE_EVENT = "apollo:start-area-capture";
export const SELECTION_RESULT_EVENT = "apollo:selection-result";
export const SELECTION_CANCELLED_EVENT = "apollo:selection-cancelled";
export const PREVIEW_CONFIRM_EVENT = "apollo:preview-confirm";
export const PREVIEW_CANCEL_EVENT = "apollo:preview-cancel";
export const PREVIEW_ANALYSIS_STATUS_EVENT = "apollo:preview-analysis-status";
export const RESPONSE_CONVERSATION_SYNC_EVENT =
  "apollo:response-conversation-sync";

export interface PreviewUpdatePayload {
  image_data_url: string | null;
  image_width: number;
  image_height: number;
  has_capture: boolean;
}

export interface SelectionResultPayload {
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

export interface PreviewConfirmPayload {
  user_notes: string;
}

export type PreviewAnalysisStatus = "ocr" | "analyzing" | "done" | "error";

export interface PreviewAnalysisStatusPayload {
  status: PreviewAnalysisStatus;
  message: string;
}

export interface ResponseUpdatePayload {
  session_id: string;
  provider_kind: ProviderKind;
  model_key: string;
  reasoning_effort: ReasoningEffort;
  display_messages: Array<{
    id: string;
    role: "assistant" | "system" | "user";
    content: string;
  }>;
  conversation_messages: ConversationMessage[];
}

export interface ResponseConversationSyncPayload {
  session_id: string;
  prompt: string;
  response: NormalizedResponse;
  appended_messages: ConversationMessage[];
}

export async function openPreviewWindow(): Promise<void> {
  try {
    const existing = await WebviewWindow.getByLabel(PREVIEW_WINDOW_LABEL);

    if (existing) {
      await existing.show();
      await existing.setFocus();
      try {
        const { LogicalPosition } = await import("@tauri-apps/api/dpi");
        await existing.setPosition(new LogicalPosition(24, 24));
      } catch {
        // noop in web mode
      }
      return;
    }

    const created = new WebviewWindow(PREVIEW_WINDOW_LABEL, {
      url: "index.html?window=preview",
      title: "Apollo Preview",
      width: 380,
      height: 520,
      decorations: false,
      transparent: true,
      alwaysOnTop: true,
      skipTaskbar: true,
      visible: true,
      center: false,
      x: 24,
      y: 24
    });

    await waitForWebviewWindow(created);
  } catch {
    // noop in web mode
  }
}

export async function openSelectionWindow(): Promise<void> {
  try {
    const existing = await WebviewWindow.getByLabel(SELECTION_WINDOW_LABEL);

    if (existing) {
      const [outerPosition, outerSize] = await Promise.allSettled([
        existing.outerPosition(),
        existing.outerSize()
      ]);
      await logWindowDebug("selection", "destroying-existing-window", {
        outerPosition:
          outerPosition.status === "fulfilled" ? outerPosition.value : null,
        outerSize: outerSize.status === "fulfilled" ? outerSize.value : null
      });
      await existing.destroy();
    }

    const placement = await resolveCursorMonitorPlacement();

    const options = {
      url: "index.html?window=selection",
      title: "Apollo Selection",
      width: placement?.logicalWidth ?? 800,
      height: placement?.logicalHeight ?? 600,
      x: placement?.logicalX,
      y: placement?.logicalY,
      decorations: false,
      transparent: true,
      alwaysOnTop: true,
      skipTaskbar: true,
      visible: true,
      shadow: false,
      focus: true
    };
    await logWindowDebug("selection", "creating-window", {
      options,
      placement
    });
    const created = new WebviewWindow(SELECTION_WINDOW_LABEL, options);

    await waitForWebviewWindow(created);
    await placeSelectionWindowOnCursorMonitor(created);
    await created.setFocus();
    const [outerPosition, outerSize] = await Promise.allSettled([
      created.outerPosition(),
      created.outerSize()
    ]);
    await logWindowDebug("selection", "created-window-shown", {
      outerPosition:
        outerPosition.status === "fulfilled" ? outerPosition.value : null,
      outerSize: outerSize.status === "fulfilled" ? outerSize.value : null
    });
  } catch (error) {
    await logWindowDebug("selection", "open-selection-error", {
      message: error instanceof Error ? error.message : String(error)
    });
  }
}

export async function openResponseWindow(): Promise<void> {
  try {
    const existing = await WebviewWindow.getByLabel(RESPONSE_WINDOW_LABEL);

    if (existing) {
      await existing.show();
      await existing.setFocus();
      return;
    }

    const created = new WebviewWindow(RESPONSE_WINDOW_LABEL, {
      url: "index.html?window=response",
      title: "Apollo Response",
      width: 480,
      height: 560,
      decorations: false,
      transparent: true,
      skipTaskbar: true,
      visible: true,
      center: true
    });

    await waitForWebviewWindow(created);
  } catch {
    // noop in web mode
  }
}

export async function emitToPreviewWindow(
  payload: PreviewUpdatePayload
): Promise<void> {
  try {
    const win = await WebviewWindow.getByLabel(PREVIEW_WINDOW_LABEL);
    await win?.emit(PREVIEW_UPDATE_EVENT, payload);
  } catch {
    // noop in web mode
  }
}

export async function emitPreviewAnalysisStatus(
  payload: PreviewAnalysisStatusPayload
): Promise<void> {
  try {
    const win = await WebviewWindow.getByLabel(PREVIEW_WINDOW_LABEL);
    await win?.emit(PREVIEW_ANALYSIS_STATUS_EVENT, payload);
  } catch {
    // noop in web mode
  }
}

export async function hidePreviewWindow(): Promise<void> {
  try {
    const win = await WebviewWindow.getByLabel(PREVIEW_WINDOW_LABEL);
    await win?.hide();
  } catch {
    // noop in web mode
  }
}

export async function emitToResponseWindow(
  payload: ResponseUpdatePayload
): Promise<void> {
  try {
    const win = await WebviewWindow.getByLabel(RESPONSE_WINDOW_LABEL);
    await win?.emit(RESPONSE_UPDATE_EVENT, payload);
  } catch {
    // noop in web mode
  }
}

export async function hideResponseWindow(): Promise<void> {
  try {
    const win = await WebviewWindow.getByLabel(RESPONSE_WINDOW_LABEL);
    await win?.hide();
  } catch {
    // noop in web mode
  }
}

export async function listenForOcrResult(
  handler: (text: string) => void
): Promise<Unlisten> {
  try {
    return await getCurrentWindow().listen<string>(
      OCR_RESULT_EVENT,
      ({ payload }) => handler(payload)
    );
  } catch {
    return () => {};
  }
}

export async function listenForShortcutAction(
  handler: (action: string) => void
): Promise<Unlisten> {
  try {
    return await getCurrentWindow().listen<string>(
      SHORTCUT_ACTION_EVENT,
      ({ payload }) => handler(payload)
    );
  } catch {
    return () => {};
  }
}

export async function listenForPreviewUpdate(
  handler: (payload: PreviewUpdatePayload) => void
): Promise<Unlisten> {
  try {
    return await getCurrentWindow().listen<PreviewUpdatePayload>(
      PREVIEW_UPDATE_EVENT,
      ({ payload }) => handler(payload)
    );
  } catch {
    return () => {};
  }
}

export async function listenForResponseUpdate(
  handler: (payload: ResponseUpdatePayload) => void
): Promise<Unlisten> {
  try {
    return await getCurrentWindow().listen<ResponseUpdatePayload>(
      RESPONSE_UPDATE_EVENT,
      ({ payload }) => handler(payload)
    );
  } catch {
    return () => {};
  }
}

export async function emitResponseConversationSync(
  payload: ResponseConversationSyncPayload
): Promise<void> {
  try {
    await getCurrentWindow().emitTo(
      APP_WINDOW_LABEL,
      RESPONSE_CONVERSATION_SYNC_EVENT,
      payload
    );
  } catch {
    // noop in web mode
  }
}

export async function listenForResponseConversationSync(
  handler: (payload: ResponseConversationSyncPayload) => void
): Promise<Unlisten> {
  try {
    return await getCurrentWindow().listen<ResponseConversationSyncPayload>(
      RESPONSE_CONVERSATION_SYNC_EVENT,
      ({ payload }) => handler(payload)
    );
  } catch {
    return () => {};
  }
}

export async function emitSelectionResult(
  payload: SelectionResultPayload
): Promise<void> {
  try {
    const { emit } = await import("@tauri-apps/api/event");
    await emit(SELECTION_RESULT_EVENT, payload);
  } catch {
    // noop in web mode
  }
}

export async function emitSelectionCancelled(): Promise<void> {
  try {
    const { emit } = await import("@tauri-apps/api/event");
    await emit(SELECTION_CANCELLED_EVENT);
  } catch {
    // noop in web mode
  }
}

export async function listenForSelectionResult(
  handler: (payload: SelectionResultPayload) => void
): Promise<Unlisten> {
  try {
    return await getCurrentWindow().listen<SelectionResultPayload>(
      SELECTION_RESULT_EVENT,
      ({ payload }) => handler(payload)
    );
  } catch {
    return () => {};
  }
}

export async function listenForSelectionCancelled(
  handler: () => void
): Promise<Unlisten> {
  try {
    return await getCurrentWindow().listen(SELECTION_CANCELLED_EVENT, () =>
      handler()
    );
  } catch {
    return () => {};
  }
}

export async function listenForStartAreaCapture(
  handler: () => void
): Promise<Unlisten> {
  try {
    return await getCurrentWindow().listen(START_AREA_CAPTURE_EVENT, () =>
      handler()
    );
  } catch {
    return () => {};
  }
}

export async function emitPreviewConfirm(
  payload: PreviewConfirmPayload
): Promise<void> {
  try {
    const { emit } = await import("@tauri-apps/api/event");
    await emit(PREVIEW_CONFIRM_EVENT, payload);
  } catch {
    // noop in web mode
  }
}

export async function emitPreviewCancel(): Promise<void> {
  try {
    const { emit } = await import("@tauri-apps/api/event");
    await emit(PREVIEW_CANCEL_EVENT);
  } catch {
    // noop in web mode
  }
}

export async function listenForPreviewConfirm(
  handler: (payload: PreviewConfirmPayload) => void
): Promise<Unlisten> {
  try {
    return await getCurrentWindow().listen<PreviewConfirmPayload>(
      PREVIEW_CONFIRM_EVENT,
      ({ payload }) => handler(payload)
    );
  } catch {
    return () => {};
  }
}

export async function listenForPreviewCancel(
  handler: () => void
): Promise<Unlisten> {
  try {
    return await getCurrentWindow().listen(PREVIEW_CANCEL_EVENT, () =>
      handler()
    );
  } catch {
    return () => {};
  }
}

export async function listenForPreviewAnalysisStatus(
  handler: (payload: PreviewAnalysisStatusPayload) => void
): Promise<Unlisten> {
  try {
    return await getCurrentWindow().listen<PreviewAnalysisStatusPayload>(
      PREVIEW_ANALYSIS_STATUS_EVENT,
      ({ payload }) => handler(payload)
    );
  } catch {
    return () => {};
  }
}
