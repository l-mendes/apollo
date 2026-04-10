# Apollo

Apollo e um aplicativo desktop multiplataforma para apoio ao aprendizado de idiomas com captura contextual de tela, OCR, integracao com providers de IA e historico de conversas.

## Status

Esta entrega avanca pela `Fase 6 - Integracao ponta a ponta`, com continuidade de conversa conectada entre UI, comandos Tauri e persistencia local.

## Stack

- Tauri 2
- Rust 2024
- Vue 3
- TypeScript
- TailwindCSS
- SQLite com migrations versionadas

## Estrutura

```text
.
|- src/                      # UI Vue 3
|- src-tauri/
|  |- src/
|  |  |- domain/             # Regras e tipos de dominio
|  |  |- application/        # Casos de uso, estado e DTOs
|  |  |- infrastructure/     # SQLite, paths, migrations, logging
|  |  |- commands/           # Comandos Tauri expostos ao frontend
|- docs/
|  |- architecture.md
|  |- phases/
```

## Comandos

```bash
npm install
npm run test
npm run tauri:dev
```

Para comandos Rust diretos sem os atalhos da raiz:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

## Convencoes

- Desenvolvimento guiado por fases e TDD.
- Separacao explicita entre dominio, aplicacao, infraestrutura e UI.
- Toda integracao externa deve entrar por adapter.
- Sem acoplamento da UI a detalhes de provider, OCR ou persistencia.

## Documentacao

- [Arquitetura](./docs/architecture.md)
- [Contratos do backend](./docs/contracts.md)
- [Erros e telemetria](./docs/errors-and-telemetry.md)
- [UI](./docs/ui.md)
- [Comportamento das janelas](./docs/window-behavior.md)
- [Fase 0](./docs/phases/phase-0-bootstrap.md)
- [Fase 1](./docs/phases/phase-1-tests.md)
- [Fase 2](./docs/phases/phase-2-contracts.md)
- [Fase 3](./docs/phases/phase-3-backend.md)
- [Fase 4](./docs/phases/phase-4-providers.md)
- [Fase 5](./docs/phases/phase-5-frontend.md)
- [Fase 6](./docs/phases/phase-6-integration.md)
- [Mapa das proximas fases](./docs/phases/roadmap.md)
- [Plano incremental das fases](./docs/phases/implementation-plan.md)
- [Guia de agentes](./AGENTS.md)
