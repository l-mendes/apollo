<script setup lang="ts">
import { Camera } from "lucide-vue-next";
import { computed } from "vue";

import {
  providerLabel,
  type ShortcutBinding
} from "@/composables/useApolloDesktop";
import { useApolloStore } from "@/store/apollo";

const emit = defineEmits<{
  capture: [];
}>();

const store = useApolloStore();

const loading = computed(
  () =>
    store.state.settings.loading || store.state.settings.providerCatalogLoading
);
const settings = computed(() => store.state.settings.draft);
const errorText = computed(() => store.getters.homeErrorText as string | null);
const isAnalyzing = computed(() => store.state.analysis.loading);
const analysisErrorText = computed(() => store.state.analysis.error);

const providerText = computed(() => {
  if (!settings.value) {
    return "Provider indisponivel";
  }

  return providerLabel(settings.value.preferred_provider);
});

const SHORTCUT_LABELS: Record<string, string> = {
  capture_screen: "Capturar tela",
  open_settings: "Abrir configuracoes",
  open_history: "Abrir historico"
};

const isMac =
  typeof navigator !== "undefined" &&
  /Mac|iPhone|iPad|iPod/i.test(navigator.platform);

function formatAccelerator(accelerator: string): string {
  return accelerator
    .split("+")
    .map((part) => {
      const trimmed = part.trim();
      if (trimmed === "CmdOrCtrl" || trimmed === "CommandOrControl") {
        return isMac ? "Cmd" : "Ctrl";
      }
      if (trimmed === "Cmd" || trimmed === "Command") return "Cmd";
      if (trimmed === "Ctrl" || trimmed === "Control") return "Ctrl";
      if (trimmed === "Alt" || trimmed === "Option") return isMac ? "Opt" : "Alt";
      if (trimmed === "Shift") return "Shift";
      return trimmed.toUpperCase();
    })
    .join(" + ");
}

function shortcutLabel(action: string): string {
  return SHORTCUT_LABELS[action] ?? action;
}

const enabledShortcuts = computed<ShortcutBinding[]>(() =>
  (settings.value?.shortcuts ?? []).filter(
    (shortcut: ShortcutBinding) => shortcut.enabled
  )
);

function openSettings() {
  store.commit("setActiveSurface", "settings");
}

function openHistory() {
  store.commit("setActiveSurface", "history");
}
</script>

<template>
  <div
    v-if="loading"
    class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6 text-sm text-slate-200"
    data-testid="home-loading"
  >
    Carregando workspace principal, modelos e preferencias locais.
  </div>

  <div
    v-else-if="!settings"
    class="rounded-xl border border-red-400/30 bg-red-500/10 p-6 text-sm text-red-100"
    data-testid="home-error"
  >
    {{ errorText ?? "Nao foi possivel montar a home do Apollo com os dados atuais." }}
  </div>

  <div
    v-else
    class="space-y-6"
    data-testid="home-ready"
  >
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="flex flex-wrap gap-2">
        <button
          class="flex items-center gap-1.5 rounded-lg border border-apollo-app-border bg-apollo-app-card px-4 py-2 text-sm text-slate-100 transition hover:border-apollo-app-accent hover:text-white"
          type="button"
          @click="openSettings"
        >
          Ajustar prompt
        </button>
        <button
          class="flex items-center gap-1.5 rounded-lg border border-apollo-app-border bg-apollo-app-card px-4 py-2 text-sm text-slate-100 transition hover:border-apollo-app-accent hover:text-white"
          type="button"
          @click="openHistory"
        >
          Ver historico
        </button>
      </div>
      <div class="flex flex-wrap gap-2">
        <button
          data-testid="capture-button"
          class="flex items-center gap-1.5 rounded-lg bg-apollo-app-accent px-4 py-2 text-sm font-semibold text-slate-950 transition hover:opacity-90 disabled:cursor-not-allowed disabled:bg-apollo-app-hover disabled:text-slate-400"
          type="button"
          :disabled="isAnalyzing"
          @click="emit('capture')"
        >
          <Camera class="h-4 w-4" />
          {{ isAnalyzing ? "Processando..." : "Capturar Tela" }}
        </button>
      </div>
    </div>

    <div
      v-if="errorText"
      class="rounded-xl border border-amber-300/25 bg-amber-300/10 px-5 py-4 text-sm text-amber-50"
    >
      {{ errorText }}
    </div>

    <div
      v-if="analysisErrorText"
      class="rounded-xl border border-red-400/30 bg-red-500/10 px-5 py-4 text-sm text-red-100"
      data-testid="home-analysis-error"
    >
      {{ analysisErrorText }}
    </div>

    <div class="grid gap-4 md:grid-cols-3">
      <div class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-5">
        <p class="text-xs font-medium uppercase text-apollo-app-muted">Provider ativo</p>
        <p class="mt-2 text-lg font-semibold text-white">{{ providerText }}</p>
      </div>
      <div class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-5">
        <p class="text-xs font-medium uppercase text-apollo-app-muted">Modelo selecionado</p>
        <p class="mt-2 text-lg font-semibold text-white">{{ settings.preferred_model }}</p>
      </div>
      <div class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-5">
        <p class="text-xs font-medium uppercase text-apollo-app-muted">Atalhos ativos</p>
        <p class="mt-2 text-lg font-semibold text-white">{{ enabledShortcuts.length }}</p>
      </div>
    </div>

    <div
      class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-5"
      data-testid="home-shortcuts"
    >
      <p class="text-xs font-medium uppercase text-apollo-app-muted">Atalhos configurados</p>

      <div
        v-if="enabledShortcuts.length === 0"
        class="mt-3 text-sm text-apollo-app-muted"
      >
        Nenhum atalho ativo no momento.
      </div>

      <ul
        v-else
        class="mt-3 divide-y divide-apollo-app-border"
      >
        <li
          v-for="shortcut in enabledShortcuts"
          :key="shortcut.action"
          class="flex items-center justify-between gap-4 py-2.5"
        >
          <span class="text-sm text-slate-100">{{ shortcutLabel(shortcut.action) }}</span>
          <kbd
            class="rounded-md border border-apollo-app-border bg-apollo-app-shell px-2.5 py-1 font-mono text-xs text-slate-100"
          >
            {{ formatAccelerator(shortcut.accelerator) }}
          </kbd>
        </li>
      </ul>
    </div>
  </div>
</template>
