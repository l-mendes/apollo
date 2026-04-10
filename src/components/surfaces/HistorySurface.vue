<script setup lang="ts">
import { computed } from "vue";

import ConversationChat from "@/components/chat/ConversationChat.vue";
import { buildChatMessages } from "@/components/chat/chatMessages";
import {
  providerLabel,
  type InteractionSession
} from "@/composables/useApolloDesktop";
import { useApolloStore } from "@/store/apollo";

const emit = defineEmits<{
  "continue-conversation": [prompt: string];
}>();

const store = useApolloStore();

const loading = computed(() => store.state.history.loading);
const errorText = computed(
  () => store.getters.historyPanelErrorText as string | null
);
const sessions = computed(() => store.state.history.items);
const conversationMessages = computed(
  () => store.state.history.conversationMessages
);
const conversationLoading = computed(
  () => store.state.history.conversationLoading
);
const conversationErrorText = computed(
  () => store.state.history.conversationError
);
const chatMessages = computed(() =>
  buildChatMessages(selectedSession.value, conversationMessages.value)
);
const continuePrompt = computed(() => store.state.history.continuePrompt);
const pendingFollowUp = computed(() => store.state.history.pendingFollowUp);
const continueLoading = computed(() => store.state.history.continueLoading);
const continueErrorText = computed(() => store.state.history.continueError);
const selectedSession = computed(
  () => store.getters.selectedSession as InteractionSession | null
);

function selectSession(sessionId: string) {
  store.commit("patchHistoryState", {
    selectedHistoryId: sessionId
  });
}

function updateContinuePrompt(prompt: string) {
  store.commit("patchHistoryState", {
    continuePrompt: prompt
  });
}

function submitFollowUp() {
  const prompt = continuePrompt.value.trim();

  if (!prompt || continueLoading.value) {
    return;
  }

  updateContinuePrompt("");
  emit("continue-conversation", prompt);
}

function openHome() {
  store.commit("setActiveSurface", "home");
}

function sourceLabel(sourceKind: InteractionSession["source_kind"]): string {
  if (sourceKind === "ScreenCapture") {
    return "Captura de tela";
  }

  if (sourceKind === "ManualText") {
    return "Texto manual";
  }

  return "Arquivo importado";
}
</script>

<template>
  <div
    v-if="loading"
    class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6 text-sm text-slate-200"
    data-testid="history-loading"
  >
    Carregando sessoes persistidas para montar o historico local.
  </div>

  <div
    v-else-if="sessions.length === 0 && errorText"
    class="rounded-xl border border-red-400/30 bg-red-500/10 p-6 text-sm text-red-100"
    data-testid="history-error"
  >
    {{ errorText }}
  </div>

  <div
    v-else-if="sessions.length === 0"
    class="rounded-xl border border-dashed border-apollo-app-border bg-apollo-app-card p-6 text-sm text-slate-300"
    data-testid="history-empty"
  >
    <p>
      Nenhuma sessao foi registrada ainda. Execute uma analise para começar o
      historico.
    </p>
    <button
      class="mt-4 rounded-lg border border-apollo-app-border bg-apollo-app-card px-4 py-2 text-sm text-slate-100 transition hover:border-apollo-app-accent hover:text-white"
      type="button"
      @click="openHome"
    >
      Ir para home
    </button>
  </div>

  <div
    v-else
    class="grid gap-6 xl:grid-cols-[280px_minmax(0,1fr)]"
    data-testid="history-ready"
  >
    <aside class="space-y-2">
      <div
        v-if="errorText"
        class="rounded-xl border border-amber-300/25 bg-amber-300/10 px-4 py-3 text-sm text-amber-50"
      >
        {{ errorText }}
      </div>

      <button
        v-for="session in sessions"
        :key="session.id"
        class="w-full rounded-xl border px-4 py-3 text-left transition"
        :class="
          selectedSession?.id === session.id
            ? 'border-apollo-app-selectedBorder bg-apollo-app-selected text-white'
            : 'border-apollo-app-border bg-apollo-app-card text-slate-200 hover:border-apollo-app-selectedBorder hover:bg-apollo-app-hover'
        "
        type="button"
        @click="selectSession(session.id)"
      >
        <p class="text-sm font-semibold">
          {{ providerLabel(session.provider_kind) }}
        </p>
        <p class="mt-1 text-xs text-apollo-app-muted">
          {{ session.model_key }} · {{ sourceLabel(session.source_kind) }}
        </p>
        <p
          class="mt-2 max-h-10 overflow-hidden text-xs leading-5 text-slate-300"
        >
          {{
            session.response_text ??
            session.ocr_text ??
            "Sem conteudo registrado."
          }}
        </p>
      </button>
    </aside>

    <section class="space-y-5">
      <template v-if="selectedSession">
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div>
            <p class="text-xs font-medium text-apollo-app-muted uppercase">
              Sessao selecionada
            </p>
            <h3 class="mt-2 text-xl font-semibold text-white">
              {{ providerLabel(selectedSession.provider_kind) }}
            </h3>
            <p class="mt-1 text-sm text-apollo-app-muted">
              {{ selectedSession.model_key }} ·
              {{ sourceLabel(selectedSession.source_kind) }}
            </p>
          </div>
          <button
            class="rounded-lg border border-apollo-app-border bg-apollo-app-card px-4 py-2 text-sm text-slate-100 transition hover:border-apollo-app-accent hover:text-white"
            type="button"
            @click="openHome"
          >
            Nova analise
          </button>
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <div
            class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-5"
          >
            <p class="text-xs font-medium text-apollo-app-muted uppercase">
              OCR
            </p>
            <p
              class="mt-2 whitespace-pre-wrap text-sm leading-6 text-slate-200"
            >
              {{ selectedSession.ocr_text ?? "Nao houve OCR persistido." }}
            </p>
          </div>
          <div
            class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-5"
          >
            <p class="text-xs font-medium text-apollo-app-muted uppercase">
              Notas
            </p>
            <p
              class="mt-2 whitespace-pre-wrap text-sm leading-6 text-slate-200"
            >
              {{ selectedSession.user_notes ?? "Sem notas adicionais." }}
            </p>
          </div>
        </div>

        <div
          class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-5"
        >
          <p class="text-xs font-medium text-apollo-app-muted uppercase">
            Prompt registrado
          </p>
          <p class="mt-2 whitespace-pre-wrap text-sm leading-6 text-slate-200">
            {{
              selectedSession.request_prompt ??
              "O prompt efetivo nao foi encontrado nesta sessao."
            }}
          </p>
        </div>

        <div
          class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-5"
        >
          <p class="text-xs font-medium text-apollo-app-muted uppercase">
            Resposta
          </p>
          <p class="mt-2 whitespace-pre-wrap text-sm leading-6 text-slate-50">
            {{ selectedSession.response_text ?? "Sem resposta persistida." }}
          </p>
        </div>

        <div
          class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-5"
        >
          <div class="flex items-center justify-between gap-3">
            <div>
              <p class="text-sm font-semibold text-white">Conversa continua</p>
              <p class="mt-1 text-sm text-apollo-app-muted">
                Use o historico persistido da sessao para seguir a conversa sem
                perder contexto.
              </p>
            </div>
            <span
              class="rounded-lg border border-apollo-app-border bg-apollo-app-shell px-3 py-1 text-xs text-apollo-app-muted"
            >
              {{ conversationMessages.length }} turnos
            </span>
          </div>

          <div class="mt-4 min-h-[24rem]">
            <ConversationChat
              :messages="chatMessages"
              :loading="conversationLoading"
              :error-text="conversationErrorText ?? continueErrorText"
              :pending-user-message="pendingFollowUp"
              :composer-value="continuePrompt"
              :composer-disabled="continueLoading"
              :composer-loading="continueLoading"
              composer-placeholder="Ex.: aprofunde a nuance, reescreva em tom informal, compare com outra expressao."
              composer-submit-label="Continuar conversa"
              empty-text="Esta sessao ainda nao possui turnos adicionais persistidos."
              show-composer
              @update:composer-value="updateContinuePrompt"
              @submit="submitFollowUp"
            />
          </div>
        </div>
      </template>
    </section>
  </div>
</template>
