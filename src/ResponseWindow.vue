<script setup lang="ts">
import { X } from "lucide-vue-next";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

import ConversationChat from "@/components/chat/ConversationChat.vue";
import type { ChatMessage } from "@/components/chat/chat.types";
import {
  commandErrorMessage,
  continueConversation,
  providerLabel,
  type ConversationMessage,
  type ProviderKind
} from "@/composables/useApolloDesktop";
import {
  emitResponseConversationSync,
  listenForResponseUpdate,
  type ResponseUpdatePayload
} from "@/composables/useWindowShell";

const sessionId = ref<string | null>(null);
const providerKind = ref<ProviderKind | null>(null);
const modelKey = ref("");
const messages = ref<ChatMessage[]>([]);
const conversationMessages = ref<ConversationMessage[]>([]);
const continuePrompt = ref("");
const continueLoading = ref(false);
const continueError = ref<string | null>(null);
const pendingFollowUp = ref<string | null>(null);

let unlistenResponse: (() => void) | null = null;

const providerText = computed(() => {
  if (!providerKind.value || !modelKey.value) {
    return "Sem conversa ativa";
  }

  return `${providerLabel(providerKind.value)} · ${modelKey.value}`;
});

function syncFromPayload(payload: ResponseUpdatePayload) {
  sessionId.value = payload.session_id;
  providerKind.value = payload.provider_kind;
  modelKey.value = payload.model_key;
  messages.value = payload.display_messages.map((message) => ({ ...message }));
  conversationMessages.value = payload.conversation_messages.map((message) => ({
    ...message
  }));
  continuePrompt.value = "";
  continueLoading.value = false;
  continueError.value = null;
  pendingFollowUp.value = null;
}

function mapConversationRole(
  role: ConversationMessage["role"]
): ChatMessage["role"] {
  if (role === "Assistant") {
    return "assistant";
  }

  if (role === "System") {
    return "system";
  }

  return "user";
}

onMounted(async () => {
  unlistenResponse = await listenForResponseUpdate(syncFromPayload);
});

onBeforeUnmount(() => {
  unlistenResponse?.();
});

async function closeWindow() {
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().hide();
  } catch {
    // noop in web mode
  }
}

async function submitFollowUp() {
  const prompt = continuePrompt.value.trim();

  if (
    !prompt ||
    continueLoading.value ||
    !sessionId.value ||
    !providerKind.value ||
    !modelKey.value
  ) {
    return;
  }

  continueLoading.value = true;
  continueError.value = null;
  pendingFollowUp.value = prompt;
  continuePrompt.value = "";

  try {
    const result = await continueConversation({
      session_id: sessionId.value,
      provider_kind: providerKind.value,
      model_key: modelKey.value,
      prompt,
      existing_messages: conversationMessages.value
    });
    const appendedDisplayMessages = result.appended_messages.map((message) => ({
      id: message.id,
      role: mapConversationRole(message.role),
      content: message.content
    }));

    conversationMessages.value = [
      ...conversationMessages.value,
      ...result.appended_messages
    ];
    messages.value = [...messages.value, ...appendedDisplayMessages];
    pendingFollowUp.value = null;

    await emitResponseConversationSync({
      session_id: sessionId.value,
      prompt,
      response: result.response,
      appended_messages: result.appended_messages
    });
  } catch (error) {
    pendingFollowUp.value = null;
    continuePrompt.value = prompt;
    continueError.value = commandErrorMessage(
      error,
      "Nao foi possivel continuar a conversa nesta janela."
    );
  } finally {
    continueLoading.value = false;
  }
}
</script>

<template>
  <div
    class="flex h-screen flex-col overflow-hidden rounded-2xl border border-apollo-app-border bg-apollo-app-card shadow-2xl"
    data-tauri-drag-region
  >
    <div
      class="flex shrink-0 items-center justify-between border-b border-apollo-app-border px-4 py-3"
      data-tauri-drag-region
    >
      <div data-tauri-drag-region>
        <p class="text-sm font-semibold text-white" data-tauri-drag-region>
          Chat do Apollo
        </p>
        <p class="mt-0.5 text-xs text-apollo-app-muted" data-tauri-drag-region>
          {{ providerText }}
        </p>
      </div>

      <button
        class="rounded-lg p-1 text-apollo-app-muted transition hover:bg-apollo-app-hover hover:text-white"
        type="button"
        @click="closeWindow"
      >
        <X class="h-4 w-4" />
      </button>
    </div>

    <div class="min-h-0 flex-1 p-4">
      <ConversationChat
        :messages="messages"
        :error-text="continueError"
        :pending-user-message="pendingFollowUp"
        :composer-value="continuePrompt"
        :composer-disabled="continueLoading || !sessionId"
        :composer-loading="continueLoading"
        composer-placeholder="Continue a conversa a partir desta resposta..."
        composer-submit-label="Enviar mensagem"
        empty-text="Aguardando a primeira resposta da analise."
        show-composer
        @update:composer-value="continuePrompt = $event"
        @submit="submitFollowUp"
      />
    </div>
  </div>
</template>
