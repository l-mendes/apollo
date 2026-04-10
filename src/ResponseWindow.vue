<script setup lang="ts">
import { X } from "lucide-vue-next";
import { onBeforeUnmount, onMounted, ref } from "vue";

import {
  listenForResponseUpdate,
  type ResponseUpdatePayload
} from "@/composables/useWindowShell";

const response = ref("");
const requestPrompt = ref("");

let unlistenResponse: (() => void) | null = null;

onMounted(async () => {
  unlistenResponse = await listenForResponseUpdate(
    (payload: ResponseUpdatePayload) => {
      response.value = payload.response;
      requestPrompt.value = payload.request_prompt;
    }
  );
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
      <p
        class="text-sm font-semibold text-white"
        data-tauri-drag-region
      >
        Resposta do Apollo
      </p>
      <button
        class="rounded-lg p-1 text-apollo-app-muted transition hover:bg-apollo-app-hover hover:text-white"
        type="button"
        @click="closeWindow"
      >
        <X class="h-4 w-4" />
      </button>
    </div>

    <div class="flex-1 space-y-5 overflow-y-auto p-4">
      <div v-if="!response" class="pt-8 text-center">
        <p class="text-sm text-apollo-app-muted">
          Aguardando resposta da analise.
        </p>
      </div>

      <div v-if="requestPrompt">
        <p class="mb-2 text-xs font-semibold uppercase tracking-wide text-apollo-app-muted">
          Prompt enviado
        </p>
        <p class="whitespace-pre-wrap text-sm leading-6 text-apollo-app-subtle">{{ requestPrompt }}</p>
      </div>

      <div v-if="response">
        <p class="mb-2 text-xs font-semibold uppercase tracking-wide text-apollo-app-muted">
          Resposta
        </p>
        <p class="whitespace-pre-wrap text-sm leading-6 text-slate-200">{{ response }}</p>
      </div>
    </div>
  </div>
</template>
