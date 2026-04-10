<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";

import type {
  ChatMessage,
  ChatMessageRole
} from "@/components/chat/chat.types";

interface DisplayMessage extends ChatMessage {
  pending?: boolean;
}

const props = withDefaults(
  defineProps<{
    messages: ChatMessage[];
    loading?: boolean;
    errorText?: string | null;
    emptyText?: string;
    pendingUserMessage?: string | null;
    pendingAssistantText?: string;
    composerValue?: string;
    composerPlaceholder?: string;
    composerDisabled?: boolean;
    composerLoading?: boolean;
    composerSubmitLabel?: string;
    showComposer?: boolean;
  }>(),
  {
    loading: false,
    errorText: null,
    emptyText: "Nenhuma mensagem registrada ainda.",
    pendingUserMessage: null,
    pendingAssistantText: "Pensando...",
    composerValue: "",
    composerPlaceholder: "Continue a conversa...",
    composerDisabled: false,
    composerLoading: false,
    composerSubmitLabel: "Enviar",
    showComposer: false
  }
);

const emit = defineEmits<{
  "update:composerValue": [value: string];
  submit: [];
}>();

const scrollContainer = ref<HTMLElement | null>(null);

const canSubmit = computed(
  () => !props.composerDisabled && Boolean(props.composerValue.trim())
);
const displayMessages = computed<DisplayMessage[]>(() => {
  const nextMessages: DisplayMessage[] = [...props.messages];
  const pendingUserMessage = props.pendingUserMessage?.trim();

  if (pendingUserMessage) {
    nextMessages.push({
      id: "pending-user-message",
      role: "user",
      content: pendingUserMessage,
      pending: true
    });
  }

  if (props.composerLoading) {
    nextMessages.push({
      id: "pending-assistant-message",
      role: "assistant",
      content: props.pendingAssistantText,
      pending: true
    });
  }

  return nextMessages;
});

function roleLabel(role: ChatMessageRole): string {
  if (role === "assistant") {
    return "Apollo";
  }

  if (role === "system") {
    return "Sistema";
  }

  return "Você";
}

function bubbleClass(role: ChatMessageRole): string {
  if (role === "assistant") {
    return "border-emerald-300/20 bg-emerald-300/10 text-slate-50";
  }

  if (role === "system") {
    return "border-apollo-app-border bg-apollo-app-shell text-slate-100";
  }

  return "border-apollo-app-selectedBorder bg-apollo-app-selected text-white";
}

function rowClass(role: ChatMessageRole): string {
  return role === "user" ? "justify-end" : "justify-start";
}

function handleComposerInput(event: Event) {
  emit("update:composerValue", (event.target as HTMLTextAreaElement).value);
}

function handleSubmit() {
  if (!canSubmit.value) {
    return;
  }

  emit("submit");
}

function handleComposerKeydown(event: KeyboardEvent) {
  if (event.key !== "Enter" || event.shiftKey) {
    return;
  }

  event.preventDefault();
  handleSubmit();
}

async function scrollToLatestMessage() {
  await nextTick();
  const element = scrollContainer.value;

  if (!element) {
    return;
  }

  element.scrollTop = element.scrollHeight;
}

watch(
  () => [displayMessages.value.length, props.composerLoading],
  () => {
    void scrollToLatestMessage();
  },
  { immediate: true }
);
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <div ref="scrollContainer" class="min-h-0 flex-1 overflow-y-auto pr-1">
      <div class="space-y-3">
        <div
          v-if="props.errorText"
          class="rounded-lg border border-red-400/30 bg-red-500/10 px-4 py-3 text-sm text-red-100"
        >
          {{ props.errorText }}
        </div>

        <div
          v-if="props.loading && displayMessages.length === 0"
          class="rounded-lg border border-apollo-app-border bg-apollo-app-shell px-4 py-3 text-sm text-slate-200"
        >
          Carregando mensagens da conversa.
        </div>

        <div
          v-else-if="displayMessages.length === 0"
          class="rounded-lg border border-dashed border-apollo-app-border bg-apollo-app-shell px-4 py-3 text-sm text-slate-300"
        >
          {{ props.emptyText }}
        </div>

        <div
          v-for="message in displayMessages"
          :key="message.id"
          class="flex"
          :class="rowClass(message.role)"
        >
          <article
            class="w-full max-w-[85%] rounded-2xl border px-4 py-3 shadow-sm"
            :class="[
              bubbleClass(message.role),
              message.pending ? 'opacity-90' : ''
            ]"
          >
            <p
              class="text-[11px] font-semibold uppercase tracking-[0.18em] text-apollo-app-muted"
            >
              {{ roleLabel(message.role) }}
            </p>
            <p class="mt-2 whitespace-pre-wrap text-sm leading-6">
              {{ message.content }}
            </p>
            <div
              v-if="message.pending && message.role === 'assistant'"
              class="mt-3 flex gap-1"
              aria-hidden="true"
            >
              <span
                class="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-200/80"
              />
              <span
                class="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-200/60 [animation-delay:120ms]"
              />
              <span
                class="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-200/40 [animation-delay:240ms]"
              />
            </div>
          </article>
        </div>
      </div>
    </div>

    <form
      v-if="props.showComposer"
      class="mt-4 space-y-3 border-t border-apollo-app-border pt-4"
      @submit.prevent="handleSubmit"
    >
      <textarea
        data-testid="chat-composer"
        class="min-h-24 w-full rounded-xl border border-apollo-app-border bg-apollo-app-shell px-4 py-3 text-sm leading-6 text-white outline-none transition focus:border-apollo-app-accent disabled:cursor-not-allowed disabled:opacity-60"
        :disabled="props.composerDisabled"
        :placeholder="props.composerPlaceholder"
        :value="props.composerValue"
        @keydown="handleComposerKeydown"
        @input="handleComposerInput"
      />

      <div class="flex justify-end">
        <button
          data-testid="chat-submit"
          class="rounded-lg bg-apollo-app-accent px-5 py-2.5 text-sm font-semibold text-slate-950 transition hover:opacity-90 disabled:cursor-not-allowed disabled:bg-apollo-app-hover disabled:text-slate-400"
          type="submit"
          :disabled="!canSubmit"
        >
          {{ props.composerSubmitLabel }}
        </button>
      </div>
    </form>
  </div>
</template>
