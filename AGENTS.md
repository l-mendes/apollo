# AGENTS.md

## Missao

Manter o Apollo evoluindo, sempre priorizando clareza arquitetural, TDD e baixo acoplamento.
Manter este arquivo e as skills objetivas e atualizadas para garantir alinhamento e autonomia.

## Fonte de contexto

- `AGENTS.md` e a fonte canonica de contexto do projeto.
- Nao usamos mais fluxo de desenvolvimento guiado por fases.
- Novas funcionalidades nao exigem gerar documentacao por fase.
- Quando a arquitetura, os fluxos principais ou as invariantes do produto mudarem, atualize este arquivo e apenas as skills impactadas.

## Resumo do sistema

Apollo e um app desktop cross-platform em Tauri para apoio ao estudo de idiomas com captura contextual de tela, OCR, analise por IA e historico local de conversas.

Fluxo principal atual:

- a tray window e a ancora da experiencia desktop;
- o usuario captura uma regiao da tela;
- o backend extrai texto via OCR;
- os casos de uso montam o prompt com configuracao base, OCR, notas e contexto anterior;
- o provider selecionado responde por HTTP ou CLI;
- sessoes e mensagens ficam persistidas em SQLite;
- a UI permite revisar historico e continuar a conversa reaproveitando a mesma sessao, provider e modelo.

## Stack e runtime

- Tauri 2 para shell desktop e comandos nativos.
- Rust 2024 no backend.
- Vue 3 + TypeScript no frontend.
- TailwindCSS na camada visual.
- SQLite com migrations versionadas para persistencia local.
- `tracing` para logging estruturado local.

## Arquitetura

- `src-tauri/src/domain`: entidades, value objects e regras puras.
- `src-tauri/src/application`: casos de uso, DTOs, estado e contratos.
- `src-tauri/src/infrastructure`: SQLite, filesystem, OCR, providers HTTP/CLI, logging e paths.
- `src-tauri/src/commands`: fronteira Tauri exposta ao frontend.
- `src`: UI Vue, composables e componentes.

## Fronteiras e invariantes

- Toda integracao com provider de IA, OCR ou CLI entra por contrato e adapter.
- Casos de uso dependem de ports; detalhes de transporte, processo, banco e filesystem ficam em `infrastructure`.
- A UI nao conhece detalhes de provider, OCR ou persistencia; ela conversa com o backend por comandos Tauri e composables.
- `useWindowShell.ts` concentra coordenacao entre janelas e eventos de shell.
- A tray continua sendo a janela ancora.
- A janela principal deve esconder ao fechar, nao encerrar a app.
- Follow-ups devem reutilizar a sessao original, o provider e o modelo da conversa.
- Mensagens de conversa devem ser carregadas em ordem de criacao.

## Persistencia e dados principais

- Configuracoes do usuario, atalhos, sessoes, mensagens e metadados de captura vivem em SQLite local.
- O bootstrap da app prepara banco, executa migrations e monta o `AppState`.
- O catalogo manual de providers/modelos fica em `src-tauri/resources/provider-models.json`.
- Mudancas de contrato entre Rust e frontend devem atualizar DTOs e interfaces TypeScript no mesmo fluxo.

## Regras de colaboracao

- Iniciar novas features com objetivo e criterio de aceite.
- Comecar pelo teste quando houver comportamento novo no backend Rust.
- Preservar a separacao entre `domain`, `application`, `infrastructure` e `presentation`.
- Toda integracao com provider de IA, OCR ou CLI deve entrar por contrato e adapter.
- Atualizar `AGENTS.md` e as skills impactadas apenas quando o contexto real do sistema mudar.

## Estrategia de testes

- Backend Rust: priorizar TDD para comportamento novo ou alterado.
- `src-tauri/tests/`: contratos, integracao e fluxos do backend.
- `tests/unit/`: comportamento de UI, composables e coordenacao de janelas com Vitest.
- Alteracoes de prompt, historico, providers, OCR, settings ou janelas devem vir acompanhadas dos testes mais proximos ao contrato afetado.

## Limites por camada

- `src-tauri/src/domain`: entidades, value objects e regras puras.
- `src-tauri/src/application`: casos de uso, orchestrators e contratos de entrada/saida.
- `src-tauri/src/infrastructure`: SQLite, files, Tesseract, execucao de CLI, HTTP e logging.
- `src`: UI Vue, composables e componentes.
- `src-tauri`: composicao da aplicacao e comandos Tauri.

## Padrao de entrega

1. Confirmar objetivo e criterio de aceite.
2. Escrever ou ajustar testes.
3. Implementar o minimo para passar.
4. Refatorar mantendo o contrato.
5. Atualizar a documentacao objetiva somente se o contexto compartilhado do sistema tiver mudado.
