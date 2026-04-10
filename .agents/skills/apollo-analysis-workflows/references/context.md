# Apollo Analysis Workflows Context

## Scope

Own capture analysis orchestration, prompt rendering, provider dispatch from application services, and the normalized response flow that feeds history and the UI.

## Primary docs

- `docs/contracts.md`
- `docs/architecture.md`
- `docs/testing-strategy.md`
- `docs/errors-and-telemetry.md`
- `docs/phases/phase-6-integration.md`

## Primary backend files

- `src-tauri/src/application/use_cases/analyze_capture.rs`
- `src-tauri/src/application/use_cases/analyze_capture_service.rs`
- `src-tauri/src/application/use_cases/continue_conversation.rs`
- `src-tauri/src/application/use_cases/continue_conversation_service.rs`
- `src-tauri/src/application/services/prompt_builder.rs`
- `src-tauri/src/commands/analysis.rs`

## Tests

- `src-tauri/tests/analysis_workflows.rs`
- `src-tauri/tests/conversation_contract.rs`
- `tests/unit/useApolloDesktop.spec.ts`

## Adjacent skills

- Use `apollo-history-conversation` when the main change is timeline rendering, session browsing, or message ordering.
- Use `apollo-provider-integrations` when adapters, runtime profiles, or normalized provider payloads change.
- Use `apollo-capture-ocr` when the input text generation pipeline changes before analysis begins.
