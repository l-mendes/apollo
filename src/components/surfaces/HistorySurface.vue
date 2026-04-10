<script setup lang="ts">
import { computed } from "vue";

import {
  providerLabel,
  type ConversationMessage,
  type InteractionSession
} from "@/composables/useApolloDesktop";

const props = defineProps<{
  loading: boolean;
  errorText: string | null;
  sessions: InteractionSession[];
  selectedSessionId: string | null;
  conversationMessages: ConversationMessage[];
  conversationLoading: boolean;
  conversationErrorText: string | null;
  continuePrompt: string;
  continueLoading: boolean;
  continueErrorText: string | null;
}>();

const emit = defineEmits<{
  "select-session": [sessionId: string];
  "update:continue-prompt": [prompt: string];
  "continue-conversation": [];
  "open-home": [];
}>();

const selectedSession = computed(
  () =>
    props.sessions.find((session) => session.id === props.selectedSessionId) ?? props.sessions[0] ?? null
);

function sourceLabel(sourceKind: InteractionSession["source_kind"]): string {
  if (sourceKind === "ScreenCapture") {
    return "Captura de tela";
  }

  if (sourceKind === "ManualText") {
    return "Texto manual";
  }

  return "Arquivo importado";
}

function roleLabel(role: ConversationMessage["role"]): string {
  if (role === "Assistant") {
    return "Apollo";
  }

  if (role === "System") {
    return "Sistema";
  }

  return "Usuario";
}
</script>

<template>
  <div
    v-if="props.loading"
    class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6 text-sm text-slate-200"
    data-testid="history-loading"
  >
    Carregando sessoes persistidas para montar o historico local.
  </div>

  <div
    v-else-if="props.sessions.length === 0 && props.errorText"
    class="rounded-xl border border-red-400/30 bg-red-500/10 p-6 text-sm text-red-100"
    data-testid="history-error"
  >
    {{ props.errorText }}
  </div>

  <div
    v-else-if="props.sessions.length === 0"
    class="rounded-xl border border-dashed border-apollo-app-border bg-apollo-app-card p-6 text-sm text-slate-300"
    data-testid="history-empty"
  >
    <p>Nenhuma sessao foi registrada ainda. Execute uma analise para começar o historico.</p>
    <button
      class="mt-4 rounded-lg border border-apollo-app-border bg-apollo-app-card px-4 py-2 text-sm text-slate-100 transition hover:border-apollo-app-accent hover:text-white"
      type="button"
      @click="emit('open-home')"
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
        v-if="props.errorText"
        class="rounded-xl border border-amber-300/25 bg-amber-300/10 px-4 py-3 text-sm text-amber-50"
      >
        {{ props.errorText }}
      </div>

      <button
        v-for="session in props.sessions"
        :key="session.id"
        class="w-full rounded-xl border px-4 py-3 text-left transition"
        :class="
          selectedSession?.id === session.id
            ? 'border-apollo-app-selectedBorder bg-apollo-app-selected text-white'
            : 'border-apollo-app-border bg-apollo-app-card text-slate-200 hover:border-apollo-app-selectedBorder hover:bg-apollo-app-hover'
        "
        type="button"
        @click="emit('select-session', session.id)"
      >
        <p class="text-sm font-semibold">{{ providerLabel(session.provider_kind) }}</p>
        <p class="mt-1 text-xs text-apollo-app-muted">{{ session.model_key }} · {{ sourceLabel(session.source_kind) }}</p>
        <p class="mt-2 max-h-10 overflow-hidden text-xs leading-5 text-slate-300">
          {{ session.response_text ?? session.ocr_text ?? 'Sem conteudo registrado.' }}
        </p>
      </button>
    </aside>

    <section class="space-y-5">
      <template v-if="selectedSession">
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div>
            <p class="text-xs font-medium text-apollo-app-muted uppercase">Sessao selecionada</p>
            <h3 class="mt-2 text-xl font-semibold text-white">{{ providerLabel(selectedSession.provider_kind) }}</h3>
            <p class="mt-1 text-sm text-apollo-app-muted">{{ selectedSession.model_key }} · {{ sourceLabel(selectedSession.source_kind) }}</p>
          </div>
          <button
            class="rounded-lg border border-apollo-app-border bg-apollo-app-card px-4 py-2 text-sm text-slate-100 transition hover:border-apollo-app-accent hover:text-white"
            type="button"
            @click="emit('open-home')"
          >
            Nova analise
          </button>
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <div class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-5">
            <p class="text-xs font-medium text-apollo-app-muted uppercase">OCR</p>
            <p class="mt-2 whitespace-pre-wrap text-sm leading-6 text-slate-200">{{ selectedSession.ocr_text ?? 'Nao houve OCR persistido.' }}</p>
          </div>
          <div class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-5">
            <p class="text-xs font-medium text-apollo-app-muted uppercase">Notas</p>
            <p class="mt-2 whitespace-pre-wrap text-sm leading-6 text-slate-200">{{ selectedSession.user_notes ?? 'Sem notas adicionais.' }}</p>
          </div>
        </div>

        <div class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-5">
          <p class="text-xs font-medium text-apollo-app-muted uppercase">Prompt registrado</p>
          <p class="mt-2 whitespace-pre-wrap text-sm leading-6 text-slate-200">
            {{ selectedSession.request_prompt ?? 'O prompt efetivo nao foi encontrado nesta sessao.' }}
          </p>
        </div>

        <div class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-5">
          <p class="text-xs font-medium text-apollo-app-muted uppercase">Resposta</p>
          <p class="mt-2 whitespace-pre-wrap text-sm leading-6 text-slate-50">
            {{ selectedSession.response_text ?? 'Sem resposta persistida.' }}
          </p>
        </div>

        <div class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-5">
          <div class="flex items-center justify-between gap-3">
            <div>
              <p class="text-sm font-semibold text-white">Conversa continua</p>
              <p class="mt-1 text-sm text-apollo-app-muted">Use o historico persistido da sessao para seguir a conversa sem perder contexto.</p>
            </div>
            <span class="rounded-lg border border-apollo-app-border bg-apollo-app-shell px-3 py-1 text-xs text-apollo-app-muted">
              {{ props.conversationMessages.length }} turnos
            </span>
          </div>

          <div
            v-if="props.conversationErrorText"
            class="mt-4 rounded-lg border border-amber-300/25 bg-amber-300/10 px-4 py-3 text-sm text-amber-50"
          >
            {{ props.conversationErrorText }}
          </div>

          <div
            v-else-if="props.conversationLoading"
            class="mt-4 rounded-lg border border-apollo-app-border bg-apollo-app-shell px-4 py-3 text-sm text-slate-200"
          >
            Carregando mensagens persistidas da sessao.
          </div>

          <div
            v-else
            class="mt-4 space-y-3"
          >
            <div
              v-if="props.conversationMessages.length === 0"
              class="rounded-lg border border-dashed border-apollo-app-border bg-apollo-app-shell px-4 py-3 text-sm text-slate-300"
            >
              Esta sessao ainda nao possui turnos adicionais persistidos.
            </div>

            <div
              v-for="message in props.conversationMessages"
              :key="message.id"
              class="rounded-lg border px-4 py-3"
              :class="
                message.role === 'Assistant'
                  ? 'border-emerald-300/20 bg-emerald-300/10'
                  : message.role === 'System'
                    ? 'border-apollo-app-border bg-apollo-app-shell'
                    : 'border-apollo-app-selectedBorder bg-apollo-app-selected'
              "
            >
              <p class="text-xs font-medium text-apollo-app-muted uppercase">{{ roleLabel(message.role) }}</p>
              <p class="mt-2 whitespace-pre-wrap text-sm leading-6 text-slate-50">{{ message.content }}</p>
            </div>
          </div>

          <label class="mt-4 grid gap-2 text-sm text-slate-200">
            <span class="font-medium text-slate-100">Follow-up</span>
            <textarea
              data-testid="continue-prompt"
              class="min-h-28 rounded-lg border border-apollo-app-border bg-apollo-app-shell px-4 py-3 text-sm leading-6 text-white outline-none transition focus:border-apollo-app-accent"
              placeholder="Ex.: aprofunde a nuance, reescreva em tom informal, compare com outra expressao."
              :value="props.continuePrompt"
              @input="emit('update:continue-prompt', ($event.target as HTMLTextAreaElement).value)"
            />
          </label>

          <div
            v-if="props.continueErrorText"
            class="mt-4 rounded-lg border border-red-400/30 bg-red-500/10 px-4 py-3 text-sm text-red-100"
          >
            {{ props.continueErrorText }}
          </div>

          <div class="mt-4 flex justify-end">
            <button
              data-testid="continue-button"
              class="rounded-lg bg-apollo-app-accent px-5 py-2.5 text-sm font-semibold text-slate-950 transition hover:opacity-90 disabled:cursor-not-allowed disabled:bg-apollo-app-hover disabled:text-slate-400"
              type="button"
              :disabled="props.continueLoading || !props.continuePrompt.trim()"
              @click="emit('continue-conversation')"
            >
              {{ props.continueLoading ? 'Continuando...' : 'Continuar conversa' }}
            </button>
          </div>
        </div>
      </template>
    </section>
  </div>
</template>