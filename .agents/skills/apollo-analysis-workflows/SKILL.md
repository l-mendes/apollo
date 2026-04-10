---
name: apollo-analysis-workflows
description: Prompt composition, capture analysis orchestration, provider dispatch, and response persistence for Apollo. Use when Codex changes `AnalyzeCapture`, `ContinueConversation`, prompt-building rules, or analysis-related Tauri commands across `src-tauri/src/application/use_cases/*`, `src-tauri/src/application/services/prompt_builder.rs`, and `src-tauri/src/commands/analysis.rs`.
---

# Apollo Analysis Workflows

## Overview

This domain owns how Apollo transforms captured text and follow-up prompts into normalized provider requests and persisted local history. It should stay in application services, not leak provider, OCR, or UI details across layers.

## Workflow

1. Decide whether the change affects first-turn analysis, follow-up conversation, or both.
2. Start with Rust tests when the request or response contract or prompt composition changes.
3. Keep orchestration in `AnalyzeCaptureService` and `ContinueConversationService`; provider details remain behind `ProviderRegistry`, `AiProvider`, and `CliProviderExecutor`.
4. Update `PromptBuilder` when the human-readable prompt format changes. Keep it deterministic and easy to assert in tests.
5. Preserve session identity rules: first analysis may create a new session, while follow-ups reuse the existing session id.
6. Persist session snapshots and conversation turns consistently; if transaction boundaries change, update repository and workflow tests together.

## Implementation Notes

- `ProviderChannel` decides whether the service calls HTTP adapters or CLI executors.
- `compose_user_turn` defines what is persisted as the user message for analysis-originated turns.
- `ContinueConversation` intentionally builds an `AnalyzeCaptureRequest`-shaped provider request to reuse normalized provider behavior.
- If the output contract changes, update both Rust DTOs and TypeScript interfaces in the frontend.

## Validation

- Run `cargo test --manifest-path src-tauri/Cargo.toml analysis_workflows`.
- Run `cargo test --manifest-path src-tauri/Cargo.toml conversation_contract`.
- Run `npx vitest run tests/unit/useApolloDesktop.spec.ts`.

## References

- Read `references/context.md` for the domain map, entry points, and adjacent skills.
