# Apollo Window UI Shell Context

## Scope

Own the multi-window shell around the Apollo experience: tray, app, preview, response, and selection windows plus surface navigation and window event wiring.

## Primary docs

- `AGENTS.md`

## Primary frontend files

- `src/App.vue`
- `src/TrayWindowApp.vue`
- `src/PreviewWindow.vue`
- `src/ResponseWindow.vue`
- `src/SelectionWindow.vue`
- `src/composables/useWindowShell.ts`
- `src/composables/useDesktopCapabilities.ts`
- `src/components/tray/FloatingTrayBar.vue`
- `src/components/surfaces/HomeSurface.vue`
- `src/components/surfaces/HistorySurface.vue`
- `src/components/surfaces/SettingsSurface.vue`

## Primary backend files

- `src-tauri/src/lib.rs`
- `src-tauri/src/commands/system.rs`

## Tests

- `tests/unit/useWindowShell.spec.ts`
- `tests/unit/FloatingTrayBar.spec.ts`
- `tests/unit/HomeSurface.spec.ts`
- `tests/unit/HistorySurface.spec.ts`
- `tests/unit/SettingsSurface.spec.ts`

## Adjacent skills

- Use `apollo-capture-ocr` when the main problem is selection overlay geometry or OCR capture flow.
- Use `apollo-history-conversation` when the issue is in the history data or follow-up conversation logic rather than surface navigation.
- Use `apollo-settings-shortcuts` when shortcut-triggered navigation or settings editing behavior changes.
