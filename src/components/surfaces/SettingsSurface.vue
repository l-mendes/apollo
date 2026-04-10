<script setup lang="ts">
import { Globe, Terminal } from "lucide-vue-next";
import { computed } from "vue";

import {
  PROVIDER_OPTIONS,
  cloneSettings,
  providerLabel,
  type ProviderModel,
  type ProviderKind,
  type ReasoningEffort,
  type UserSettings
} from "@/composables/useApolloDesktop";
import { useApolloStore } from "@/store/apollo";

const emit = defineEmits<{
  save: [];
}>();

const store = useApolloStore();

const loading = computed(
  () =>
    store.state.settings.loading || store.state.settings.providerCatalogLoading
);
const saving = computed(() => store.state.settings.saving);
const errorText = computed(
  () =>
    (store.getters.settingsPanelErrorText as string | null) ??
    store.state.settings.providerCatalogError
);
const settings = computed(() => store.state.settings.draft);
const modelsByProvider = computed(() => store.state.settings.providerCatalog);
const hasUnsavedChanges = computed(
  () => store.getters.hasUnsavedSettings as boolean
);

const availableModels = computed(() => {
  if (!settings.value) {
    return [];
  }

  return modelsByProvider.value[settings.value.preferred_provider] ?? [];
});

function updateDraft(nextSettings: UserSettings) {
  store.commit("patchSettingsState", {
    draft: nextSettings
  });
}

function changeProvider(providerKind: ProviderKind) {
  if (!settings.value) {
    return;
  }

  const models = modelsByProvider.value[providerKind] as ProviderModel[] | undefined;
  const fallbackModel =
    models?.find((model: ProviderModel) => model.is_default)?.model_key ??
    models?.[0]?.model_key ??
    settings.value.preferred_model;

  updateDraft({
    ...cloneSettings(settings.value),
    preferred_provider: providerKind,
    preferred_model: fallbackModel
  });
}

function changeModel(modelKey: string) {
  if (!settings.value) {
    return;
  }

  updateDraft({
    ...cloneSettings(settings.value),
    preferred_model: modelKey
  });
}

function changeReasoningEffort(reasoningEffort: ReasoningEffort) {
  if (!settings.value) {
    return;
  }

  updateDraft({
    ...cloneSettings(settings.value),
    reasoning_effort: reasoningEffort
  });
}

function changeBasePrompt(basePrompt: string) {
  if (!settings.value) {
    return;
  }

  updateDraft({
    ...cloneSettings(settings.value),
    base_prompt: basePrompt
  });
}

function changeOcrLanguage(ocrLanguage: string) {
  if (!settings.value) {
    return;
  }

  updateDraft({
    ...cloneSettings(settings.value),
    ocr_language: ocrLanguage
  });
}

function changeOutputLanguage(outputLanguage: string) {
  if (!settings.value) {
    return;
  }

  updateDraft({
    ...cloneSettings(settings.value),
    output_language: outputLanguage
  });
}

const OCR_LANGUAGE_OPTIONS = [
  { value: "por", label: "Português (por)" },
  { value: "eng", label: "English (eng)" },
  { value: "spa", label: "Español (spa)" },
  { value: "fra", label: "Français (fra)" },
  { value: "deu", label: "Deutsch (deu)" },
  { value: "ita", label: "Italiano (ita)" },
  { value: "chi_sim", label: "中文 Simplificado (chi_sim)" },
  { value: "jpn", label: "日本語 (jpn)" }
];

const OUTPUT_LANGUAGE_OPTIONS = [
  "Português",
  "English",
  "Español",
  "Français",
  "Deutsch",
  "Italiano",
  "中文",
  "日本語"
];

const REASONING_EFFORT_OPTIONS: Array<{
  value: ReasoningEffort;
  label: string;
  description: string;
}> = [
  {
    value: "low",
    label: "Low",
    description: "Mais rapido e economico para perguntas simples."
  },
  {
    value: "medium",
    label: "Medium",
    description: "Equilibrio padrao entre velocidade e qualidade."
  },
  {
    value: "high",
    label: "High",
    description: "Mais raciocinio para explicacoes complexas."
  },
  {
    value: "xhigh",
    label: "XHigh",
    description: "Maximo esforco quando o provider suportar."
  }
];

function changeShortcutValue(
  index: number,
  field: "action" | "accelerator",
  value: string
) {
  if (!settings.value) {
    return;
  }

  const nextSettings = cloneSettings(settings.value);
  nextSettings.shortcuts[index] = {
    ...nextSettings.shortcuts[index],
    [field]: value
  };

  updateDraft(nextSettings);
}

function changeShortcutEnabled(index: number, enabled: boolean) {
  if (!settings.value) {
    return;
  }

  const nextSettings = cloneSettings(settings.value);
  nextSettings.shortcuts[index] = {
    ...nextSettings.shortcuts[index],
    enabled
  };

  updateDraft(nextSettings);
}
</script>

<template>
  <div
    v-if="loading"
    class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6 text-sm text-slate-200"
    data-testid="settings-loading"
  >
    Carregando preferencias persistidas e o catalogo de modelos disponiveis.
  </div>

  <div
    v-else-if="!settings"
    class="rounded-xl border border-red-400/30 bg-red-500/10 p-6 text-sm text-red-100"
    data-testid="settings-error"
  >
    {{ errorText ?? "Nao foi possivel carregar as configuracoes atuais." }}
  </div>

  <div
    v-else
    class="space-y-6"
    data-testid="settings-ready"
  >
    <div class="flex items-center justify-between">
      <span
        class="rounded-full border px-3 py-1 text-xs"
        :class="
          hasUnsavedChanges
            ? 'border-amber-300/30 bg-amber-300/10 text-amber-50'
            : 'border-emerald-400/20 bg-emerald-400/10 text-emerald-100'
        "
      >
        {{ hasUnsavedChanges ? "Alteracoes locais" : "Sincronizado" }}
      </span>
      <button
        class="rounded-lg bg-apollo-app-accent px-5 py-2.5 text-sm font-semibold text-slate-950 transition hover:opacity-90 disabled:cursor-not-allowed disabled:bg-apollo-app-hover disabled:text-slate-400"
        type="button"
        :disabled="saving"
        @click="emit('save')"
      >
        {{ saving ? "Salvando..." : "Salvar configuracoes" }}
      </button>
    </div>

    <div
      v-if="errorText"
      class="rounded-xl border border-amber-300/25 bg-amber-300/10 px-5 py-4 text-sm text-amber-50"
    >
      {{ errorText }}
    </div>

    <div class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6">
      <p class="text-sm font-semibold text-white">Fonte de IA</p>
      <p class="mt-1 text-sm text-apollo-app-muted">Escolha o provider para todas as analises deste workspace.</p>

      <div class="mt-5 grid grid-cols-2 gap-3 sm:grid-cols-3 xl:grid-cols-4">
        <button
          v-for="provider in PROVIDER_OPTIONS"
          :key="provider.kind"
          class="flex flex-col items-start gap-2 rounded-xl border p-4 text-left transition"
          :class="
            settings.preferred_provider === provider.kind
              ? 'border-apollo-app-accent bg-apollo-app-selected text-white'
              : 'border-apollo-app-border bg-apollo-app-shell text-apollo-app-muted hover:border-apollo-app-selectedBorder hover:text-white'
          "
          type="button"
          @click="changeProvider(provider.kind)"
        >
          <Globe v-if="provider.channel === 'Http'" class="h-5 w-5 shrink-0" />
          <Terminal v-else class="h-5 w-5 shrink-0" />
          <span class="text-sm font-medium leading-tight">{{ provider.label }}</span>
          <span
            class="rounded-full px-2 py-0.5 text-[10px] font-bold uppercase leading-none"
            :class="
              provider.channel === 'Http'
                ? 'bg-sky-400/15 text-sky-300'
                : 'bg-violet-400/15 text-violet-300'
            "
          >
            {{ provider.channel === "Http" ? "HTTP" : "CLI" }}
          </span>
        </button>
      </div>
    </div>

    <div class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6">
      <p class="text-sm font-semibold text-white">Modelo</p>
      <p class="mt-1 text-sm text-apollo-app-muted">Modelo preferido para o provider selecionado.</p>
      <div class="mt-5">
        <select
          class="w-full rounded-lg border border-apollo-app-border bg-apollo-app-shell px-4 py-2.5 text-sm text-white outline-none transition focus:border-apollo-app-accent"
          :value="settings.preferred_model"
          @change="changeModel(($event.target as HTMLSelectElement).value)"
        >
          <option
            v-for="model in availableModels"
            :key="model.model_key"
            :value="model.model_key"
          >
            {{ model.display_name }}
          </option>
          <option
            v-if="availableModels.length === 0"
            value=""
          >
            Nenhum modelo disponivel
          </option>
        </select>
      </div>
    </div>

    <div class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6">
      <p class="text-sm font-semibold text-white">Reasoning</p>
      <p class="mt-1 text-sm text-apollo-app-muted">Esforco de raciocinio usado por providers compativeis. Providers sem suporte ignoram esta preferencia.</p>
      <div class="mt-5 grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <button
          v-for="option in REASONING_EFFORT_OPTIONS"
          :key="option.value"
          class="rounded-xl border p-4 text-left transition"
          :class="
            settings.reasoning_effort === option.value
              ? 'border-apollo-app-accent bg-apollo-app-selected text-white'
              : 'border-apollo-app-border bg-apollo-app-shell text-apollo-app-muted hover:border-apollo-app-selectedBorder hover:text-white'
          "
          type="button"
          @click="changeReasoningEffort(option.value)"
        >
          <span class="text-sm font-semibold">{{ option.label }}</span>
          <span class="mt-2 block text-xs leading-5 text-apollo-app-muted">{{ option.description }}</span>
        </button>
      </div>
    </div>

    <div class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6">
      <p class="text-sm font-semibold text-white">Prompt Base</p>
      <p class="mt-1 text-sm text-apollo-app-muted">O prompt base e usado como contexto principal em todas as analises.</p>
      <textarea
        class="mt-4 min-h-40 w-full rounded-lg border border-apollo-app-border bg-apollo-app-shell px-4 py-3 text-sm leading-6 text-white outline-none transition focus:border-apollo-app-accent"
        :value="settings.base_prompt"
        @input="changeBasePrompt(($event.target as HTMLTextAreaElement).value)"
      />
    </div>

    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
      <div class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6">
        <p class="text-sm font-semibold text-white">Idioma de Entrada (OCR)</p>
        <p class="mt-1 text-sm text-apollo-app-muted">Idioma do texto que sera extraido da imagem pelo Tesseract.</p>
        <div class="mt-4">
          <select
            class="w-full rounded-lg border border-apollo-app-border bg-apollo-app-shell px-4 py-2.5 text-sm text-white outline-none transition focus:border-apollo-app-accent"
            :value="settings.ocr_language"
            @change="changeOcrLanguage(($event.target as HTMLSelectElement).value)"
          >
            <option
              v-for="option in OCR_LANGUAGE_OPTIONS"
              :key="option.value"
              :value="option.value"
            >
              {{ option.label }}
            </option>
          </select>
        </div>
      </div>

      <div class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6">
        <p class="text-sm font-semibold text-white">Idioma de Saida (IA)</p>
        <p class="mt-1 text-sm text-apollo-app-muted">A IA sempre respondera neste idioma, independente do idioma da entrada.</p>
        <div class="mt-4">
          <select
            class="w-full rounded-lg border border-apollo-app-border bg-apollo-app-shell px-4 py-2.5 text-sm text-white outline-none transition focus:border-apollo-app-accent"
            :value="settings.output_language"
            @change="changeOutputLanguage(($event.target as HTMLSelectElement).value)"
          >
            <option
              v-for="lang in OUTPUT_LANGUAGE_OPTIONS"
              :key="lang"
              :value="lang"
            >
              {{ lang }}
            </option>
          </select>
        </div>
      </div>
    </div>

    <div>
      <p class="text-lg font-semibold text-white">Atalhos de Teclado</p>
      <p class="mt-2 text-sm text-apollo-app-muted">Personalize os atalhos para corresponder ao seu fluxo de trabalho.</p>

      <div class="mt-5 space-y-3">
        <div
          v-for="(shortcut, index) in settings.shortcuts"
          :key="`${shortcut.action}-${index}`"
          class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6"
        >
          <div class="flex items-start justify-between gap-4">
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-3">
                <input
                  class="min-w-0 flex-1 border-0 bg-transparent text-base font-semibold text-white outline-none placeholder:text-apollo-app-muted"
                  :value="shortcut.action"
                  placeholder="Nome do atalho"
                  @input="changeShortcutValue(index, 'action', ($event.target as HTMLInputElement).value)"
                />
                <label class="inline-flex shrink-0 items-center gap-2 text-xs text-apollo-app-muted">
                  <input
                    class="h-4 w-4 rounded border-apollo-app-border bg-apollo-app-shell"
                    type="checkbox"
                    :checked="shortcut.enabled"
                    @change="changeShortcutEnabled(index, ($event.target as HTMLInputElement).checked)"
                  />
                  Ativo
                </label>
              </div>
              <p class="mt-1 text-sm text-apollo-app-muted">{{ providerLabel(settings.preferred_provider) }} workflow</p>
            </div>
          </div>

          <div class="mt-4 flex items-center gap-2">
            <span
              v-for="(key, keyIndex) in shortcut.accelerator.split('+')"
              :key="keyIndex"
              class="rounded-lg border border-apollo-app-border bg-apollo-app-shell px-3 py-1.5 text-sm font-medium text-white"
            >
              {{ key.trim() }}
            </span>
            <input
              class="ml-auto w-40 rounded-lg border border-apollo-app-border bg-apollo-app-shell px-3 py-1.5 text-sm text-white outline-none transition focus:border-apollo-app-accent"
              :value="shortcut.accelerator"
              placeholder="Ex: Ctrl+E"
              @input="changeShortcutValue(index, 'accelerator', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
