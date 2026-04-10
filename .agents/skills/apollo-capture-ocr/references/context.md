# Apollo Capture OCR Context

## Scope

Own screen capture, rectangular selection, OCR execution, and the payloads that flow from capture into preview and analysis.

## Primary docs

- `docs/architecture.md`
- `docs/ui.md`
- `docs/window-behavior.md`
- `docs/testing-strategy.md`
- `docs/phases/phase-6-integration.md`

## Primary backend files

- `src-tauri/src/application/ports/capture.rs`
- `src-tauri/src/infrastructure/capture.rs`
- `src-tauri/src/infrastructure/ocr.rs`
- `src-tauri/src/commands/capture.rs`

## Primary frontend files

- `src/composables/useWindowShell.ts`
- `src/SelectionWindow.vue`
- `src/PreviewWindow.vue`
- `src/components/analysis/CapturePreviewCard.vue`

## Tests

- `src-tauri/tests/ocr_contract.rs`
- `tests/unit/selectionGeometry.spec.ts`
- `tests/unit/useWindowShell.spec.ts`

## Adjacent skills

- Use `apollo-settings-shortcuts` when OCR language or shortcut persistence changes.
- Use `apollo-window-ui-shell` when the work is mostly about native window lifecycle or tray/app synchronization.
- Use `apollo-analysis-workflows` when the captured OCR text changes how prompts are composed or persisted.
