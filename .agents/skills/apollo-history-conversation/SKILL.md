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
4. Keep `HistorySurface.vue` prop-driven. Fetching, selection, and persistence stay in composables and Tauri commands.
5. Preserve the invariant from Fase 6: follow-ups reuse the original session id, provider, and model so the local timeline remains coherent.

## Implementation Notes

- `list_history` returns session summaries for the left rail; `load_conversation_messages` feeds the detailed timeline.
- The selected session card is a summary, not the full conversation. Avoid bloating it with infrastructure details.
- When changing history rendering, update the empty, loading, and error states explicitly; the project documents those UI states as part of the product contract.

## Validation

- Run `cargo test --manifest-path src-tauri/Cargo.toml persistence_repository`.
- Run `cargo test --manifest-path src-tauri/Cargo.toml history_contract`.
- Run `npx vitest run tests/unit/HistorySurface.spec.ts tests/unit/useApolloDesktop.spec.ts`.

## References

- Read `references/context.md` for the domain map, entry points, and adjacent skills.
