# AGENTS.md

## Mission

Keep Apollo evolving with architectural clarity, TDD, and low coupling as the default.
Keep this file and the impacted skills concise and up to date so contributors stay aligned and autonomous.

## Source Of Truth

- `AGENTS.md` is the canonical shared context for this project.
- We no longer use a phase-driven development workflow.
- New features do not require per-phase documentation.
- Shared documentation in this repository should be written in English.
- When architecture, core flows, or product invariants change, update this file and only the impacted skills.

## System Summary

Apollo is a cross-platform Tauri desktop app that helps with language study through contextual screen capture, OCR, AI analysis, and local conversation history.

Current main flow:

- the tray window is the anchor of the desktop experience;
- the user captures a region of the screen;
- the backend extracts text through OCR;
- use cases build the prompt from the base configuration, OCR output, user notes, and prior context;
- the selected provider answers through HTTP or CLI;
- sessions and messages are persisted in SQLite;
- the response window presents the captured OCR text and model answer as a compact chat;
- the UI lets the user review history and continue the conversation with the same session, provider, and model.

## Stack And Runtime

- Tauri 2 for the desktop shell and native commands.
- Rust 2024 in the backend.
- Vue 3 + TypeScript in the frontend.
- TailwindCSS in the visual layer.
- `xcap` for native per-monitor screen capture.
- SQLite with versioned migrations for local persistence.
- `tracing` for structured local logging.

## Architecture

- `src-tauri/src/domain`: entities, value objects, and pure rules.
- `src-tauri/src/application`: use cases, DTOs, state, and contracts.
- `src-tauri/src/infrastructure`: SQLite, filesystem, OCR, HTTP and CLI providers, logging, and paths.
- `src-tauri/src/commands`: Tauri-facing command boundary exposed to the frontend.
- `src`: Vue UI, composables, and components.

## Boundaries And Invariants

- Every AI provider, OCR, or CLI integration must enter through a contract and adapter.
- Use cases depend on ports; transport, process, database, and filesystem details stay in `infrastructure`.
- The UI must not know provider, OCR, or persistence details; it talks to the backend through Tauri commands and composables.
- `useWindowShell.ts` owns window coordination and shell events.
- The tray remains the anchor window.
- The main window should hide on close instead of terminating the app.
- Area capture uses a transparent overlay window on the active monitor, with selection geometry resolved in the frontend and the rectangle sent to the backend.
- The backend captures the full monitor image through `xcap` and crops the chosen region locally to avoid logical-versus-physical coordinate drift.
- The selection overlay must disappear visually before capture so preview images never include borders, backdrop tint, or helper hints.
- Follow-ups must reuse the original session, provider, and model.
- Conversation messages must load in creation order.
- Chat-style conversation surfaces should reuse the shared `ConversationChat` component and keep prompt internals hidden from the user-facing timeline.
- During follow-up processing, chat surfaces should clear the composer, show the submitted message plus a thinking state, and keep input disabled until the provider response completes.

## Persistence And Core Data

- User settings, shortcuts, sessions, messages, and capture metadata live in local SQLite.
- App bootstrap prepares the database, runs migrations, and builds `AppState`.
- The manual provider and model catalog lives in `src-tauri/resources/provider-models.json`.
- Contract changes between Rust and the frontend must update Rust DTOs and TypeScript interfaces in the same flow.

## Collaboration Rules

- Start new features with a clear objective and acceptance criteria.
- Start with tests when backend Rust behavior changes.
- Preserve the separation between `domain`, `application`, `infrastructure`, and `presentation`.
- Every AI provider, OCR, or CLI integration must enter through a contract and adapter.
- Update `AGENTS.md` and the impacted skills only when the shared system context actually changes.

## Testing Strategy

- Backend Rust: prefer TDD for new or changed behavior.
- `src-tauri/tests/`: backend contracts, integration, and workflow coverage.
- `tests/unit/`: UI, composable, and window-coordination behavior with Vitest.
- Changes to prompts, history, providers, OCR, settings, capture, or windows should include the tests closest to the affected contract.

## Layer Limits

- `src-tauri/src/domain`: entities, value objects, and pure rules.
- `src-tauri/src/application`: use cases, orchestrators, and input-output contracts.
- `src-tauri/src/infrastructure`: SQLite, files, Tesseract, CLI execution, HTTP, and logging.
- `src`: Vue UI, composables, and components.
- `src-tauri`: app composition and Tauri commands.

## Delivery Pattern

1. Confirm the objective and acceptance criteria.
2. Write or adjust tests.
3. Implement the minimum needed to pass.
4. Refactor while preserving the contract.
5. Update concise documentation only when the shared system context has changed.
