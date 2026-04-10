---
name: apollo-persistence-bootstrap
description: SQLite bootstrap, migrations, repository wiring, bootstrap snapshot, filesystem paths, and application runtime composition for Apollo. Use when Codex changes database initialization, schema evolution, repository mappings, `BootstrapSnapshot`, `AppState`, or Tauri startup composition in `src-tauri/src/lib.rs`.
---

# Apollo Persistence Bootstrap

## Overview

This domain owns how Apollo boots its local runtime, prepares SQLite, runs migrations, exposes bootstrap metadata, and wires concrete repositories and adapters into `AppState`. Prefer schema-first changes and keep startup composition explicit.

## Workflow

1. Start with the storage contract: migration files, repository queries, and domain/application structs must stay synchronized.
2. Keep SQLite setup in infrastructure. `AppState::new` composes concrete implementations, but it should not hide schema drift.
3. Update `BootstrapSnapshot` and related frontend contracts whenever startup metadata changes.
4. Preserve initialization order in `src-tauri/src/lib.rs`: logging, plugin wiring, database bootstrap, `AppState`, then window-specific runtime adjustments.
5. When changing repository behavior, update in-memory or temp-file tests before editing production paths.

## Implementation Notes

- `SqliteAppRepository::initialize` is the safe entry point for schema readiness.
- `run_migrations` is part of the public startup contract; migration names surface in the bootstrap summary.
- `paths.rs`, `database.rs`, and `migrations.rs` form one concern even though the modules are split.
- If a schema change impacts user-visible data, update the appropriate domain skill too instead of hiding the downstream contract break here.

## Validation

- Run `cargo test --manifest-path src-tauri/Cargo.toml persistence_repository`.
- Run `cargo test --manifest-path src-tauri/Cargo.toml`.
- Run `npx vitest run tests/unit/useDesktopCapabilities.spec.ts`.

## References

- Read `references/context.md` for the domain map, entry points, and adjacent skills.
