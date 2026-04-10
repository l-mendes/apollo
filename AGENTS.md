# AGENTS.md

## Missao

Manter o Apollo evoluindo em pequenas fases, sempre priorizando clareza arquitetural, TDD e baixo acoplamento.

## Regras de colaboracao

- Iniciar novas features somente apos registrar fase, objetivo e criterio de aceite.
- Comecar pelo teste quando houver comportamento novo no backend Rust.
- Preservar a separacao entre `domain`, `application`, `infrastructure` e `presentation`.
- Toda integracao com provider de IA, OCR ou CLI deve entrar por contrato e adapter.
- Toda alteracao relevante deve atualizar a documentacao da fase correspondente.

## Limites por camada

- `src-tauri/src/domain`: entidades, value objects e regras puras.
- `src-tauri/src/application`: casos de uso, orchestrators e contratos de entrada/saida.
- `src-tauri/src/infrastructure`: SQLite, files, Tesseract, execucao de CLI, HTTP e logging.
- `src`: UI Vue, composables e componentes.
- `src-tauri`: composicao da aplicacao e comandos Tauri.

## Padrao de entrega

1. Confirmar fase atual.
2. Escrever ou ajustar testes.
3. Implementar o minimo para passar.
4. Refatorar mantendo o contrato.
5. Atualizar documentacao objetiva.
