---
name: apollo-settings-shortcuts
description: User preferences, provider/model defaults, prompt settings, OCR and output languages, and global shortcut behavior for Apollo. Use when Codex changes `UserSettings`, shortcut parsing or registration, settings Tauri commands, `SettingsSurface.vue`, or runtime shortcut resolution in `AppState`.
---

# Apollo Settings Shortcuts

## Overview

This domain owns persisted user preferences and the runtime behavior derived from them, especially global shortcuts and language/provider defaults. Keep the Rust and TypeScript contracts aligned and treat shortcut normalization as a backend concern.

## Workflow

1. Start from the contract: `UserSettings`, `ShortcutBinding`, and their TypeScript mirrors must move together.
2. Keep persistence in repositories, runtime caches in `AppState`, and native registration in `commands/capture.rs`.
3. Preserve the invariant that saving settings also updates the runtime OCR language used by shortcut-triggered capture flows.
4. Treat accelerator parsing and matching as identity-based backend logic, not as a string-comparison feature in the UI.
5. Keep `SettingsSurface.vue` and `HomeSurface.vue` prop-driven. They should emit semantic changes, not know Tauri details.

## Implementation Notes

- Provider, model, and reasoning effort controls feed the persisted `UserSettings` contract used by analysis requests.
- Shortcut registration should fail safely: warn on invalid registrations, but keep the application responsive.
- The frontend can format accelerators for display, but the backend decides what shortcut actually fired.
- If you introduce new user preferences, update migrations, repository queries, Rust entities, and TypeScript interfaces in the same change.

## Validation

- Run `cargo test --manifest-path src-tauri/Cargo.toml settings_contract`.
- Run `cargo test --manifest-path src-tauri/Cargo.toml`.
- Run `npx vitest run tests/unit/SettingsSurface.spec.ts tests/unit/HomeSurface.spec.ts tests/unit/useApolloDesktop.spec.ts`.

## References

- Read `references/context.md` for the domain map, entry points, and adjacent skills.
