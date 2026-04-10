---
name: apollo-provider-integrations
description: HTTP and CLI provider adapters, model catalog loading, provider registry resolution, runtime profiles, and normalized provider errors for Apollo. Use when Codex changes OpenAI, Anthropic, Ollama, Claude CLI, Codex CLI, Copilot CLI, model selection, auth, timeout, or provider registration behavior in `src-tauri/src/infrastructure/providers/**`, `src-tauri/src/infrastructure/provider_catalog.rs`, `src-tauri/src/infrastructure/provider_registry.rs`, and related tests.
---

# Apollo Provider Integrations

## Overview

This domain owns how Apollo talks to remote HTTP providers and local CLI providers while preserving one normalized application contract. Keep adapters behind ports and keep model availability explicit through the manual catalog plus runtime profiles.

## Workflow

1. Decide whether the change belongs to the HTTP path, the CLI path, the manual model catalog, or the registry and composition layer.
2. Add or update the provider profile first: endpoint, auth policy, timeout, CLI arguments, and prompt mode.
3. Keep provider-specific request and response parsing inside the adapter. The application layer should only see `NormalizedResponse` and `ApplicationError`.
4. Update the manual catalog and frontend provider options together when a provider or model choice changes.
5. Test error normalization deliberately. Authentication, timeout, missing CLI, and invalid configuration are first-class product behaviors.

## Implementation Notes

- `ManualProviderModelCatalog` is the source of truth for provider and model selections exposed to the UI.
- `InMemoryProviderRegistry` is the composition boundary that wires the catalog to concrete adapters.
- HTTP providers share `HttpTransport`; CLI providers share `GenericCliProviderExecutor` and `ProcessCommandRunner`.
- CLI runtime profiles define how selected model keys and reasoning effort are translated into provider-specific command arguments.
- Runtime profiles can read environment variables, but missing credentials must still normalize into clear `ApplicationErrorKind` values.

## Validation

- Run `cargo test --manifest-path src-tauri/Cargo.toml http_provider_adapters`.
- Run `cargo test --manifest-path src-tauri/Cargo.toml cli_provider_execution`.
- Run `cargo test --manifest-path src-tauri/Cargo.toml provider_registry_resolution`.

## References

- Read `references/context.md` for the domain map, entry points, and adjacent skills.
