# Apollo

Apollo is a cross-platform desktop application for language learning support with contextual screen capture, OCR, AI provider integration, and conversation history.

## Stack

- Tauri 2
- Rust 2024
- Vue 3
- TypeScript
- TailwindCSS
- SQLite with versioned migrations

## Structure

```text
.
|- src/                      # Vue 3 UI
|- src-tauri/
|  |- src/
|  |  |- domain/             # Domain rules and types
|  |  |- application/        # Use cases, state, and DTOs
|  |  |- infrastructure/     # SQLite, paths, migrations, logging
|  |  |- commands/           # Tauri commands exposed to the frontend
```

## Commands

```bash
npm install
npm run test
npm run tauri:dev
```

For direct Rust commands without the root-level shortcuts:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

## Conventions

- Spec-driven development with TDD.
- Explicit separation between domain, application, infrastructure, and UI.
- Every external integration must go through an adapter.
- No UI coupling to provider, OCR, or persistence details.
