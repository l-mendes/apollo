<script setup lang="ts">
import { Trash2 } from "lucide-vue-next";
import { computed } from "vue";

import {
  providerLabel,
  type InteractionSession
} from "@/composables/useApolloDesktop";
import { useApolloStore } from "@/store/apollo";

const emit = defineEmits<{
  "clear-history": [];
  "delete-session": [sessionId: string];
  "open-session-chat": [sessionId: string];
}>();

const store = useApolloStore();

const loading = computed(() => store.state.history.loading);
const errorText = computed(
  () => store.getters.historyPanelErrorText as string | null
);
const sessions = computed(() => store.state.history.items);
const selectedSession = computed(
  () => store.getters.selectedSession as InteractionSession | null
);

function selectSession(sessionId: string) {
  store.commit("patchHistoryState", {
    selectedHistoryId: sessionId
  });
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

function sessionPreview(session: InteractionSession): string {
  return (
    session.response_text ??
    session.ocr_text ??
    session.user_notes ??
    "Sem conteudo registrado."
  );
}

function openSessionChat(sessionId: string) {
  selectSession(sessionId);
  emit("open-session-chat", sessionId);
}

function deleteSession(sessionId: string) {
  emit("delete-session", sessionId);
}

function openHome() {
  store.commit("setActiveSurface", "home");
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

  <section
    v-else
    class="rounded-xl border border-apollo-app-border bg-apollo-app-card"
    data-testid="history-ready"
  >
    <div
      class="flex flex-wrap items-center justify-between gap-3 border-b border-apollo-app-border px-5 py-4"
    >
      <div>
        <p class="text-sm font-semibold text-white">Sessoes salvas</p>
        <p class="mt-1 text-sm text-apollo-app-muted">
          Clique duas vezes em uma sessao para abrir o chat em uma janela
          separada.
        </p>
      </div>

      <button
        data-testid="clear-history-button"
        class="rounded-lg border border-red-400/30 bg-red-500/10 px-4 py-2 text-sm font-medium text-red-100 transition hover:bg-red-500/20 disabled:cursor-not-allowed disabled:opacity-50"
        type="button"
        :disabled="sessions.length === 0"
        @click="emit('clear-history')"
      >
        Limpar historico
      </button>
    </div>

    <div
      v-if="errorText"
      class="mx-5 mt-4 rounded-xl border border-amber-300/25 bg-amber-300/10 px-4 py-3 text-sm text-amber-50"
    >
      {{ errorText }}
    </div>

    <ul class="divide-y divide-apollo-app-border">
      <li
        v-for="session in sessions"
        :key="session.id"
        class="group grid gap-4 px-5 py-4 transition hover:bg-apollo-app-hover md:grid-cols-[minmax(0,1fr)_auto]"
        :class="selectedSession?.id === session.id ? 'bg-apollo-app-hover' : ''"
        data-testid="history-session"
        @click="selectSession(session.id)"
        @dblclick="openSessionChat(session.id)"
      >
        <button
          class="min-w-0 text-left"
          type="button"
          @click="selectSession(session.id)"
          @dblclick.stop="openSessionChat(session.id)"
        >
          <div class="flex flex-wrap items-center gap-2">
            <p class="text-sm font-semibold text-white">
              {{ providerLabel(session.provider_kind) }}
            </p>
            <span
              class="rounded-full border border-apollo-app-border bg-apollo-app-shell px-2 py-0.5 text-[11px] text-apollo-app-muted"
            >
              {{ sourceLabel(session.source_kind) }}
            </span>
          </div>

          <p class="mt-1 text-xs text-apollo-app-muted">
            {{ session.model_key }}
          </p>
          <p class="mt-2 line-clamp-2 text-sm leading-6 text-slate-300">
            {{ sessionPreview(session) }}
          </p>
        </button>

        <div class="flex items-center justify-end">
          <button
            data-testid="delete-session-button"
            class="inline-flex h-9 w-9 items-center justify-center rounded-lg border border-apollo-app-border text-apollo-app-muted transition hover:border-red-400/40 hover:bg-red-500/10 hover:text-red-100"
            :aria-label="`Excluir sessao ${session.id}`"
            type="button"
            @click.stop="deleteSession(session.id)"
            @dblclick.stop
          >
            <Trash2 class="h-4 w-4" aria-hidden="true" />
          </button>
        </div>
      </li>
    </ul>
  </section>
</template>
