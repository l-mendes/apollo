---
name: apollo-capture-ocr
description: Screen capture, rectangular selection, preview-to-OCR flow, and Tesseract integration for Apollo. Use when Codex changes capture ports/adapters, OCR execution, capture-related Tauri commands, selection geometry, or monitor-aware window coordination across `src-tauri/src/infrastructure/capture.rs`, `src-tauri/src/infrastructure/ocr.rs`, `src-tauri/src/commands/capture.rs`, `src/composables/useWindowShell.ts`, and `tests/unit/selectionGeometry.spec.ts`.
---

# Apollo Capture OCR

## Overview

Keep capture and OCR separated behind `ScreenCapturePort` and `OcrEngine`. Treat monitor math, filesystem writes, base64 conversion, and Tesseract execution as infrastructure concerns, with Tauri commands acting only as a bridge to the UI.

## Workflow

1. Decide which boundary changed first: area selection, screen capture, OCR execution, shortcut-triggered capture, or preview/selection window coordination.
2. Start from tests when backend behavior changes. Prefer Rust tests for adapter behavior and Vitest for geometry or window-event behavior.
3. Keep `src-tauri/src/commands/capture.rs` thin. Put screen, filesystem, base64, and process logic in infrastructure modules.
4. Preserve the distinction between logical coordinates used by the selection overlay and physical coordinates used to place the real native window on the active monitor.
5. Normalize failures with `ApplicationErrorKind`; do not surface raw crate or OS errors directly to the frontend.
6. Coordinate OCR language changes with the settings domain instead of hardcoding values locally.

## Implementation Notes

- Treat `capture_screen_region_sync` and `extract_text_sync` as the synchronous core that can be wrapped by `spawn_blocking`.
- Preserve the contract of `CaptureRegionResponse`: file path, width, height, and a ready-to-render `data:` URL.
- When changing selection behavior, inspect both `useWindowShell.ts` and `tests/unit/selectionGeometry.spec.ts`; the bug surface is usually in monitor placement, not in the Vue surface.
- Keep preview and response orchestration event-driven. Window events are part of the shell, but capture payload shape belongs to this domain.

## Validation

- Run `cargo test --manifest-path src-tauri/Cargo.toml ocr_contract`.
- Run `cargo test --manifest-path src-tauri/Cargo.toml`.
- Run `npx vitest run tests/unit/selectionGeometry.spec.ts`.

## References

- Read `references/context.md` for the domain map, entry points, and adjacent skills.
