<script setup lang="ts">
import { Check, Loader2, X } from "lucide-vue-next";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

import {
  emitPreviewCancel,
  emitPreviewConfirm,
  hideCurrentWindow,
  listenForPreviewAnalysisStatus,
  listenForPreviewUpdate,
  type PreviewAnalysisStatus,
  type PreviewAnalysisStatusPayload,
  type PreviewUpdatePayload
} from "@/composables/useWindowShell";

const imageDataUrl = ref<string | null>(null);
const imageWidth = ref(0);
const imageHeight = ref(0);
const hasCapture = ref(false);
const userNotes = ref("");

const analysisStatus = ref<PreviewAnalysisStatus | null>(null);
const analysisMessage = ref("");

const isProcessing = computed(
  () => analysisStatus.value === "ocr" || analysisStatus.value === "analyzing"
);

const statusLabel = computed(() => {
  switch (analysisStatus.value) {
    case "ocr":
      return "Extraindo texto...";
    case "analyzing":
      return "Analisando com o provider...";
    default:
      return "";
  }
});

let unlistenPreview: (() => void) | null = null;
let unlistenStatus: (() => void) | null = null;

onMounted(async () => {
  unlistenPreview = await listenForPreviewUpdate(
    (payload: PreviewUpdatePayload) => {
      imageDataUrl.value = payload.image_data_url;
      imageWidth.value = payload.image_width;
      imageHeight.value = payload.image_height;
      hasCapture.value = payload.has_capture;
      analysisStatus.value = null;
      analysisMessage.value = "";
      if (payload.has_capture) {
        userNotes.value = "";
      }
    }
  );

  unlistenStatus = await listenForPreviewAnalysisStatus(
    (payload: PreviewAnalysisStatusPayload) => {
      analysisStatus.value = payload.status;
      analysisMessage.value = payload.message;

      if (payload.status === "done") {
        void hideCurrentWindow();
      }
    }
  );
});

onBeforeUnmount(() => {
  unlistenPreview?.();
  unlistenStatus?.();
});

async function confirmCapture() {
  analysisStatus.value = "ocr";
  analysisMessage.value = "";
  await emitPreviewConfirm({ user_notes: userNotes.value });
}

async function cancelCapture() {
  await emitPreviewCancel();
  imageDataUrl.value = null;
  hasCapture.value = false;
  userNotes.value = "";
  analysisStatus.value = null;
  analysisMessage.value = "";
  await hideCurrentWindow();
}

function dismissError() {
  analysisStatus.value = null;
  analysisMessage.value = "";
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
        <p
          class="text-sm font-semibold text-white"
          data-tauri-drag-region
        >
          Preview da Captura
        </p>
        <p
          v-if="hasCapture && !isProcessing"
          class="mt-0.5 text-xs text-apollo-app-muted"
          data-tauri-drag-region
        >
          {{ imageWidth }} × {{ imageHeight }} px
        </p>
        <p
          v-if="isProcessing"
          class="mt-0.5 flex items-center gap-1.5 text-xs text-apollo-app-accent"
        >
          <Loader2 class="h-3 w-3 animate-spin" />
          {{ statusLabel }}
        </p>
      </div>
      <button
        class="rounded-lg p-1 text-apollo-app-muted transition hover:bg-apollo-app-hover hover:text-white"
        type="button"
        :disabled="isProcessing"
        @click="cancelCapture"
      >
        <X class="h-4 w-4" />
      </button>
    </div>

    <div class="flex flex-1 flex-col gap-4 overflow-y-auto p-4">
      <div
        v-if="!hasCapture"
        class="flex flex-1 items-center justify-center rounded-xl border border-dashed border-apollo-app-border bg-apollo-app-shell px-6 py-10 text-center"
      >
        <p class="text-sm text-apollo-app-muted">
          Aguardando captura. Use Capturar Tela para selecionar uma área.
        </p>
      </div>

      <template v-else>
        <div
          class="flex items-center justify-center overflow-hidden rounded-xl border border-apollo-app-border bg-apollo-app-shell p-2"
        >
          <img
            v-if="imageDataUrl"
            :src="imageDataUrl"
            alt="Pré-visualização da área capturada"
            class="max-h-56 max-w-full rounded-lg object-contain"
          />
        </div>

        <label class="grid gap-2 text-sm text-slate-200">
          <span class="text-xs font-semibold uppercase text-apollo-app-muted">
            Notas adicionais
          </span>
          <textarea
            v-model="userNotes"
            :disabled="isProcessing"
            class="min-h-24 rounded-lg border border-apollo-app-border bg-apollo-app-shell px-3 py-2 text-sm leading-6 text-white outline-none transition focus:border-apollo-app-accent disabled:opacity-50"
            placeholder="Ex.: foco em registro informal, diferença de nuance, explicação rápida em português."
          />
        </label>

        <div
          v-if="analysisStatus === 'error' && analysisMessage"
          class="rounded-xl border border-red-400/30 bg-red-500/10 px-4 py-3 text-sm text-red-100"
        >
          <p>{{ analysisMessage }}</p>
          <button
            class="mt-2 text-xs font-medium text-red-300 underline transition hover:text-white"
            type="button"
            @click="dismissError"
          >
            Tentar novamente
          </button>
        </div>

        <div
          v-if="isProcessing"
          class="flex items-center justify-center gap-2 rounded-xl border border-apollo-app-border bg-apollo-app-shell px-4 py-4"
        >
          <Loader2 class="h-5 w-5 animate-spin text-apollo-app-accent" />
          <span class="text-sm text-slate-200">{{ statusLabel }}</span>
        </div>

        <div
          v-if="!isProcessing && analysisStatus !== 'error'"
          class="flex shrink-0 items-center justify-between gap-3"
        >
          <button
            class="flex flex-1 items-center justify-center gap-2 rounded-lg border border-apollo-app-border bg-apollo-app-shell px-4 py-2.5 text-sm font-medium text-slate-100 transition hover:border-red-400/40 hover:text-white"
            type="button"
            @click="cancelCapture"
          >
            <X class="h-4 w-4" />
            Cancelar
          </button>
          <button
            class="flex flex-1 items-center justify-center gap-2 rounded-lg bg-apollo-app-accent px-4 py-2.5 text-sm font-semibold text-slate-950 transition hover:opacity-90"
            type="button"
            @click="confirmCapture"
          >
            <Check class="h-4 w-4" />
            Analisar
          </button>
        </div>
      </template>
    </div>
  </div>
</template>
