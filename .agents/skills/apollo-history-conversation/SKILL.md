---
name: apollo-history-conversation
description: Session history, persisted conversation messages, follow-up flows, and the History surface for Apollo. Use when Codex changes session listing, message ordering, conversation loading, or history UI behavior across `src-tauri/src/commands/history.rs`, `src-tauri/src/infrastructure/persistence.rs`, `src/components/surfaces/HistorySurface.vue`, and the history/conversation tests.
---

# Apollo History Conversation

## Overview

This domain owns how Apollo stores, loads, and presents sessions plus conversation turns after an analysis has been run. Treat session metadata and conversation messages as separate but coordinated records.

## Workflow

1. Start by deciding whether the change is about storage, command exposure, or History surface rendering.
2. Keep sessions in `interaction_sessions` and turn-by-turn messages in `conversation_messages`; do not collapse both concerns into one structure.
3. Preserve ordering guarantees when loading messages. The UI assumes the backend returns the conversation in creation order.
4. Keep `HistorySurface.vue` prop-driven and list-only. Fetching, selection, deletion, clearing, and response-window opening stay in composables and Tauri commands.
5. Preserve the follow-up invariant: reuse the original session id, provider, and model so the local timeline remains coherent.

## Implementation Notes

- `list_history` returns session summaries for the left rail; `load_conversation_messages` feeds the detailed timeline.
- `delete_history_session` and `clear_history` must remove sessions and their persisted conversation messages.
- The History surface renders only the session list. Double-clicking a session opens the reusable response chat window with the selected timeline.
- Chat timeline rendering is shared through `ConversationChat`; map persisted first-turn capture messages back to the visible OCR text instead of showing composed prompt internals.
- Follow-up UX should optimistically show the submitted user message, clear the composer, show the thinking state, and only unlock input when continuation finishes.
- When changing history rendering, update the empty, loading, and error states explicitly; the project documents those UI states as part of the product contract.

## Validation

- Run `cargo test --manifest-path src-tauri/Cargo.toml --test persistence_repository`.
- Run `cargo test --manifest-path src-tauri/Cargo.toml history_contract`.
- Run `npx vitest run tests/unit/HistorySurface.spec.ts tests/unit/ResponseWindow.spec.ts tests/unit/chatMessages.spec.ts tests/unit/useApolloDesktop.spec.ts`.

## References

- Read `references/context.md` for the domain map, entry points, and adjacent skills.
