# Apollo Persistence Bootstrap Context

## Scope

Own database bootstrap, migrations, SQLite repository initialization, app metadata/bootstrap summary, and the runtime composition of concrete adapters inside `AppState`.

## Primary docs

- `AGENTS.md`

## Primary backend files

- `src-tauri/migrations/`
- `src-tauri/src/infrastructure/database.rs`
- `src-tauri/src/infrastructure/migrations.rs`
- `src-tauri/src/infrastructure/paths.rs`
- `src-tauri/src/infrastructure/persistence.rs`
- `src-tauri/src/application/bootstrap_snapshot.rs`
- `src-tauri/src/application/state.rs`
- `src-tauri/src/lib.rs`

## Frontend touchpoints

- `src/composables/useDesktopCapabilities.ts`

## Tests

- `src-tauri/tests/persistence_repository.rs`
- `src-tauri/src/application/bootstrap_snapshot.rs`
- `src-tauri/src/infrastructure/database.rs`
- `src-tauri/src/infrastructure/migrations.rs`
- `tests/unit/useDesktopCapabilities.spec.ts`

## Adjacent skills

- Use `apollo-history-conversation` when repository changes affect how sessions or messages are loaded.
- Use `apollo-settings-shortcuts` when repository changes affect persisted preferences or shortcut bindings.
- Use `apollo-provider-integrations` when app bootstrap wiring or registry composition changes provider behavior.
