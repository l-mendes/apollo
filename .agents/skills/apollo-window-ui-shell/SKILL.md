---
name: apollo-window-ui-shell
description: Multi-window Vue and Tauri shell behavior for Apollo, including tray, app, preview, response, and selection windows plus surface navigation and window events. Use when Codex changes `src/App.vue`, `src/TrayWindowApp.vue`, window entrypoints, `src/composables/useWindowShell.ts`, tray/app synchronization, or window-focused UI tests.
---

# Apollo Window UI Shell

## Overview

This domain owns the desktop shell that stitches Vue surfaces to native Tauri windows. Keep components declarative and keep window lifecycle, focus, appearance, and event wiring in shell composables.

## Workflow

1. Decide whether the change belongs to a surface component, a window entrypoint, or the window shell/composable.
2. Preserve the main shell invariants: the tray is the anchor window, the app window hides on close instead of exiting, and surface state syncs between windows via events.
3. Keep components prop-driven and emit-driven. Avoid direct `invoke` or native window calls inside leaf components.
4. Prefer event-driven communication for preview, response, and selection windows instead of cross-importing component state.
5. Re-check platform-specific window sizing behavior in `src-tauri/src/lib.rs` when changing tray size or app-window creation.

## Implementation Notes

- `useWindowShell.ts` is the coordination hub for labels, events, placement, and appearance.
- `useDesktopCapabilities.ts` handles runtime bootstrap and quit actions; keep it separate from domain-specific commands.
- Surface components should implement loading, ready, and error states explicitly because those states are part of the documented UI contract.
- The response window renders a compact chat from shell events; follow-ups from that window must sync back to the app window without exposing composed prompt internals.
- History double-click opens the selected session timeline in the response window rather than rendering conversation detail inside `HistorySurface.vue`.
- Response-window follow-ups should clear the composer immediately, show the submitted message and thinking state, then unlock input only after the provider response is fully synced.
- Selection window placement depends on cursor monitor resolution and is easy to break on multi-monitor or HiDPI setups.
- `syncTrayWindowAppearance` only calls `setShadow(false)`. All other tray properties (size, alwaysOnTop, decorations, skipTaskbar) are declared in `tauri.conf.json` and do not need to be re-applied at runtime.
- On Linux, WebKitGTK has a natural preferred height of ~200 px that overrides `tauri.conf.json`. The fix lives in `src-tauri/src/lib.rs` inside the `.setup` closure: `with_webview` calls `gtk::prelude::WidgetExt::set_size_request(300, 48)` on the underlying widget. Do not add size-management workarounds back to `syncTrayWindowAppearance` or the setup block.

## Validation

- Run `npx vitest run tests/unit/useWindowShell.spec.ts tests/unit/ResponseWindow.spec.ts tests/unit/FloatingTrayBar.spec.ts tests/unit/HomeSurface.spec.ts`.
- Run `npm run typecheck`.
- Run `npm run build` if window-entry composition changes broadly.

## References

- Read `references/context.md` for the domain map, entry points, and adjacent skills.
