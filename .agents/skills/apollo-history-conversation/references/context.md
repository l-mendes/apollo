# Apollo History Conversation Context

## Scope

Own persisted sessions, persisted conversation messages, follow-up continuation from history, and the History surface that lets users inspect prior interactions.

## Primary docs

- `AGENTS.md`

## Primary backend files

- `src-tauri/src/commands/history.rs`
- `src-tauri/src/infrastructure/persistence.rs`
- `src-tauri/src/domain/entities/interaction_session.rs`
- `src-tauri/src/domain/entities/conversation_message.rs`

## Primary frontend files

- `src/components/surfaces/HistorySurface.vue`
- `src/composables/useApolloDesktop.ts`

## Tests

- `src-tauri/tests/persistence_repository.rs`
- `src-tauri/tests/history_contract.rs`
- `src-tauri/tests/conversation_contract.rs`
- `tests/unit/HistorySurface.spec.ts`
- `tests/unit/useApolloDesktop.spec.ts`

## Adjacent skills

- Use `apollo-analysis-workflows` when the follow-up orchestration or prompt-building logic changes.
- Use `apollo-persistence-bootstrap` when the schema, migrations, or repository bootstrap are the main concern.
- Use `apollo-window-ui-shell` when the issue is navigation between tray/app surfaces rather than history data itself.
