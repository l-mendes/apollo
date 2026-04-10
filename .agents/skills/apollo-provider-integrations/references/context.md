# Apollo Provider Integrations Context

## Scope

Own provider adapters, transport and command execution helpers, model catalog loading, runtime profiles, and registry-based resolution of HTTP and CLI providers.

## Primary docs

- `AGENTS.md`

## Primary backend files

- `src-tauri/resources/provider-models.json`
- `src-tauri/src/infrastructure/provider_catalog.rs`
- `src-tauri/src/infrastructure/provider_registry.rs`
- `src-tauri/src/infrastructure/providers/http/adapters.rs`
- `src-tauri/src/infrastructure/providers/http/transport.rs`
- `src-tauri/src/infrastructure/providers/cli/executor.rs`
- `src-tauri/src/infrastructure/providers/cli/command_runner.rs`
- `src-tauri/src/infrastructure/providers/runtime_profiles.rs`
- `src-tauri/src/application/ports/provider.rs`

## Frontend touchpoints

- `src/composables/useApolloDesktop.ts`
- `src/components/surfaces/SettingsSurface.vue`

## Tests

- `src-tauri/tests/http_provider_adapters.rs`
- `src-tauri/tests/cli_provider_execution.rs`
- `src-tauri/tests/cli_providers_contract.rs`
- `src-tauri/tests/providers_contract.rs`
- `src-tauri/tests/provider_registry_resolution.rs`
- `src-tauri/tests/model_selection_contract.rs`

## Adjacent skills

- Use `apollo-analysis-workflows` when orchestration changes after the registry resolves a provider.
- Use `apollo-settings-shortcuts` when the visible provider and model preference UI changes.
- Use `apollo-persistence-bootstrap` when app bootstrap wiring or registry composition changes provider behavior.
