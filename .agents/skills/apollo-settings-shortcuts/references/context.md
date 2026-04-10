# Apollo Settings Shortcuts Context

## Scope

Own user settings, shortcut bindings, OCR and output language preferences, preferred provider and model, and the runtime path that turns saved bindings into registered global shortcuts.

## Primary docs

- `AGENTS.md`

## Primary backend files

- `src-tauri/src/domain/entities/user_settings.rs`
- `src-tauri/src/domain/entities/shortcut_binding.rs`
- `src-tauri/src/application/state.rs`
- `src-tauri/src/commands/settings.rs`
- `src-tauri/src/commands/capture.rs`
- `src-tauri/src/infrastructure/persistence.rs`

## Primary frontend files

- `src/components/surfaces/SettingsSurface.vue`
- `src/components/surfaces/HomeSurface.vue`
- `src/composables/useApolloDesktop.ts`

## Tests

- `src-tauri/tests/settings_contract.rs`
- `src-tauri/src/application/state.rs`
- `tests/unit/SettingsSurface.spec.ts`
- `tests/unit/HomeSurface.spec.ts`
- `tests/unit/useApolloDesktop.spec.ts`

## Adjacent skills

- Use `apollo-provider-integrations` when the available provider or model choices or catalog semantics change.
- Use `apollo-capture-ocr` when OCR language changes need end-to-end validation with capture flows.
- Use `apollo-window-ui-shell` when shortcut-triggered navigation or settings editing behavior is the main concern.
