<script setup lang="ts">
import { Bot, Globe, Keyboard, Languages, Terminal } from "lucide-vue-next";
import { computed, onBeforeUnmount, ref } from "vue";

import {
  PROVIDER_OPTIONS,
  cloneSettings,
  type ProviderModel,
  type ProviderKind,
  type ReasoningEffort,
  type UserSettings
} from "@/composables/useApolloDesktop";
import { useApolloStore } from "@/store/apollo";

const emit = defineEmits<{
  "shortcut-recording-change": [recording: boolean];
}>();

const store = useApolloStore();

const loading = computed(
  () =>
    store.state.settings.loading || store.state.settings.providerCatalogLoading
);
const errorText = computed(
  () =>
    (store.getters.settingsPanelErrorText as string | null) ??
    store.state.settings.providerCatalogError
);
const settings = computed(() => store.state.settings.draft);
const modelsByProvider = computed(() => store.state.settings.providerCatalog);
type SettingsContext = "providers" | "language" | "shortcuts";

const recordingShortcutIndex = ref<number | null>(null);
const shortcutCaptureErrors = ref<Record<number, string>>({});
const activeSettingsContext = ref<SettingsContext>("providers");

const isMac =
  typeof navigator !== "undefined" &&
  /Mac|iPhone|iPad|iPod/i.test(navigator.platform);

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

  const models = modelsByProvider.value[providerKind] as
    | ProviderModel[]
    | undefined;
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

const SETTINGS_CONTEXTS: Array<{
  id: SettingsContext;
  label: string;
  icon: typeof Bot;
}> = [
  {
    id: "providers",
    label: "Provedores de IA",
    icon: Bot
  },
  {
    id: "language",
    label: "Idioma",
    icon: Languages
  },
  {
    id: "shortcuts",
    label: "Atalhos",
    icon: Keyboard
  }
];

const MODIFIER_KEYS = new Set([
  "Alt",
  "Control",
  "Ctrl",
  "Meta",
  "OS",
  "Shift"
]);

const SPECIAL_KEY_LABELS: Record<string, string> = {
  " ": "Space",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  ArrowUp: "Up",
  Esc: "Escape"
};

const SHORTCUT_ACTION_LABELS: Record<string, string> = {
  capture_screen: "Capturar Tela",
  open_settings: "Abrir Configuracoes",
  open_history: "Abrir Historico"
};

const MODIFIER_ALIASES: Record<string, string> = {
  alt: "Alt",
  cmd: "Cmd",
  command: "Cmd",
  commandorcontrol: isMac ? "Cmd" : "Ctrl",
  control: "Ctrl",
  ctrl: "Ctrl",
  cmdorctrl: isMac ? "Cmd" : "Ctrl",
  meta: "Cmd",
  option: "Alt",
  shift: "Shift"
};

const MODIFIER_ORDER = ["Cmd", "Ctrl", "Shift", "Alt"];

function changeShortcutAccelerator(index: number, accelerator: string) {
  if (!settings.value) {
    return;
  }

  const nextSettings = cloneSettings(settings.value);
  nextSettings.shortcuts[index] = {
    ...nextSettings.shortcuts[index],
    accelerator
  };

  updateDraft(nextSettings);
}

function shortcutActionLabel(action: string): string {
  return SHORTCUT_ACTION_LABELS[action] ?? action;
}

function shortcutMainKey(event: KeyboardEvent): string | null {
  if (MODIFIER_KEYS.has(event.key)) {
    return null;
  }

  const mappedKey = SPECIAL_KEY_LABELS[event.key] ?? event.key;

  if (mappedKey.length === 1) {
    return mappedKey.toUpperCase();
  }

  return mappedKey;
}

function formatShortcutKey(key: string): string {
  const trimmed = key.trim();
  const alias = MODIFIER_ALIASES[trimmed.toLowerCase()];

  if (alias) {
    return alias;
  }

  return SPECIAL_KEY_LABELS[trimmed] ?? trimmed;
}

function shortcutDisplayParts(accelerator: string): string[] {
  return accelerator.split("+").map(formatShortcutKey).filter(Boolean);
}

function canonicalAccelerator(accelerator: string): string | null {
  const parts = accelerator
    .split("+")
    .map((part) => part.trim())
    .filter(Boolean);

  if (parts.length === 0) {
    return null;
  }

  const modifiers = new Set<string>();
  let mainKey = "";

  for (const part of parts) {
    const alias = MODIFIER_ALIASES[part.toLowerCase()];

    if (alias) {
      modifiers.add(alias);
    } else {
      mainKey = formatShortcutKey(part).toUpperCase();
    }
  }

  if (!mainKey) {
    return null;
  }

  return [
    ...MODIFIER_ORDER.filter((modifier) => modifiers.has(modifier)),
    mainKey
  ].join("+");
}

function acceleratorFromKeyboardEvent(event: KeyboardEvent): string | null {
  const mainKey = shortcutMainKey(event);

  if (!mainKey) {
    return null;
  }

  const parts: string[] = [];

  if (event.metaKey) {
    parts.push("Cmd");
  }

  if (event.ctrlKey) {
    parts.push("Ctrl");
  }

  if (event.shiftKey) {
    parts.push("Shift");
  }

  if (event.altKey) {
    parts.push("Alt");
  }

  parts.push(mainKey);

  return parts.join("+");
}

function shortcutConflictIndex(
  index: number,
  accelerator: string
): number | null {
  if (!settings.value) {
    return null;
  }

  const candidate = canonicalAccelerator(accelerator);

  if (!candidate) {
    return null;
  }

  const conflictIndex = settings.value.shortcuts.findIndex(
    (shortcut, shortcutIndex) =>
      shortcutIndex !== index &&
      canonicalAccelerator(shortcut.accelerator) === candidate
  );

  return conflictIndex >= 0 ? conflictIndex : null;
}

function setShortcutCaptureError(index: number, message: string | null) {
  const nextErrors = { ...shortcutCaptureErrors.value };

  if (message) {
    nextErrors[index] = message;
  } else {
    delete nextErrors[index];
  }

  shortcutCaptureErrors.value = nextErrors;
}

function isShortcutRecording(index: number): boolean {
  return recordingShortcutIndex.value === index;
}

function startShortcutRecording(index: number) {
  if (recordingShortcutIndex.value === null) {
    emit("shortcut-recording-change", true);
  }

  recordingShortcutIndex.value = index;
  setShortcutCaptureError(index, null);
}

function stopShortcutRecording(index: number) {
  if (recordingShortcutIndex.value === index) {
    recordingShortcutIndex.value = null;
    emit("shortcut-recording-change", false);
  }
}

function handleShortcutRecorderKeydown(index: number, event: KeyboardEvent) {
  if (!isShortcutRecording(index)) {
    return;
  }

  event.preventDefault();
  event.stopPropagation();

  if (event.key === "Escape") {
    stopShortcutRecording(index);
    setShortcutCaptureError(index, null);
    return;
  }

  const accelerator = acceleratorFromKeyboardEvent(event);

  if (!accelerator) {
    return;
  }

  if (!accelerator.includes("+")) {
    setShortcutCaptureError(
      index,
      "Use uma combinacao com Ctrl, Alt, Shift ou Cmd."
    );
    return;
  }

  const conflictIndex = shortcutConflictIndex(index, accelerator);

  if (conflictIndex !== null && settings.value) {
    setShortcutCaptureError(
      index,
      `Ja usado por ${shortcutActionLabel(
        settings.value.shortcuts[conflictIndex].action
      )}.`
    );
    return;
  }

  changeShortcutAccelerator(index, accelerator);
  setShortcutCaptureError(index, null);
  stopShortcutRecording(index);
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

const shortcutConflicts = computed<Record<number, number>>(() => {
  const conflicts: Record<number, number> = {};

  if (!settings.value) {
    return conflicts;
  }

  settings.value.shortcuts.forEach((shortcut, index) => {
    const conflictIndex = shortcutConflictIndex(index, shortcut.accelerator);

    if (conflictIndex !== null) {
      conflicts[index] = conflictIndex;
    }
  });

  return conflicts;
});

const hasShortcutValidationErrors = computed(
  () => Object.keys(shortcutConflicts.value).length > 0
);

function shortcutValidationMessage(index: number): string | null {
  const captureError = shortcutCaptureErrors.value[index];

  if (captureError) {
    return captureError;
  }

  const conflictIndex = shortcutConflicts.value[index];

  if (conflictIndex === undefined || !settings.value) {
    return null;
  }

  return `Ja atribuido a ${shortcutActionLabel(
    settings.value.shortcuts[conflictIndex].action
  )}.`;
}

onBeforeUnmount(() => {
  if (recordingShortcutIndex.value !== null) {
    emit("shortcut-recording-change", false);
  }
});
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
    class="flex h-full min-h-0 overflow-hidden"
    data-testid="settings-ready"
  >
    <aside
      class="h-full w-60 shrink-0 border-r border-apollo-app-border px-3 py-4 space-y-2"
      aria-label="Contextos de configuracao"
    >
      <button
        v-for="context in SETTINGS_CONTEXTS"
        :key="context.id"
        class="flex h-9 w-full items-center gap-3 rounded-md border px-3 text-left text-sm font-semibold transition last:mb-0"
        :class="
          activeSettingsContext === context.id
            ? 'border-apollo-app-accent bg-apollo-app-selected text-white'
            : 'border-transparent text-white hover:bg-apollo-app-hover'
        "
        type="button"
        @click="activeSettingsContext = context.id"
      >
        <component
          :is="context.icon"
          class="h-4 w-4 shrink-0"
          :class="
            activeSettingsContext === context.id
              ? 'text-apollo-app-accent'
              : 'text-slate-300'
          "
        />
        <span class="truncate">{{ context.label }}</span>
      </button>
    </aside>

    <div class="min-h-0 flex-1 overflow-hidden">
      <div
        v-if="errorText"
        class="mx-6 mt-5 rounded-xl border border-amber-300/25 bg-amber-300/10 px-5 py-4 text-sm text-amber-50"
      >
        {{ errorText }}
      </div>

      <section
        v-if="activeSettingsContext === 'providers'"
        class="h-full min-h-0 space-y-4 overflow-y-auto px-6 py-5"
        data-testid="settings-context-providers"
      >
        <div>
          <p class="text-2xl font-semibold text-white">Provedores de IA</p>
          <p class="mt-2 max-w-2xl text-sm leading-6 text-apollo-app-muted">
            Conecte o Apollo a providers remotos, modelos locais e prompts de
            analise.
          </p>
        </div>

        <div
          class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6"
        >
          <p class="text-sm font-semibold text-white">Provedor</p>
          <p class="mt-1 text-sm text-apollo-app-muted">
            Escolha o provider para todas as analises deste workspace.
          </p>

          <div class="mt-5 grid grid-cols-2 gap-3 xl:grid-cols-4">
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
              <Globe
                v-if="provider.channel === 'Http'"
                class="h-5 w-5 shrink-0"
              />
              <Terminal v-else class="h-5 w-5 shrink-0" />
              <span class="text-sm font-medium leading-tight">{{
                provider.label
              }}</span>
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

        <div
          class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6"
        >
          <p class="text-sm font-semibold text-white">Modelo</p>
          <p class="mt-1 text-sm text-apollo-app-muted">
            Modelo preferido para o provider selecionado.
          </p>
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
              <option v-if="availableModels.length === 0" value="">
                Nenhum modelo disponivel
              </option>
            </select>
          </div>
        </div>

        <div
          class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6"
        >
          <p class="text-sm font-semibold text-white">Reasoning</p>
          <p class="mt-1 text-sm text-apollo-app-muted">
            Esforco de raciocinio usado por providers compativeis. Providers sem
            suporte ignoram esta preferencia.
          </p>
          <div
            class="mt-5 grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4"
          >
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
              <span
                class="mt-2 block text-xs leading-5 text-apollo-app-muted"
                >{{ option.description }}</span
              >
            </button>
          </div>
        </div>

        <div
          class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6"
        >
          <p class="text-sm font-semibold text-white">Prompt Base</p>
          <p class="mt-1 text-sm text-apollo-app-muted">
            O prompt base e usado como contexto principal em todas as analises.
          </p>
          <textarea
            class="mt-4 min-h-40 w-full rounded-lg border border-apollo-app-border bg-apollo-app-shell px-4 py-3 text-sm leading-6 text-white outline-none transition focus:border-apollo-app-accent"
            :value="settings.base_prompt"
            @input="
              changeBasePrompt(($event.target as HTMLTextAreaElement).value)
            "
          />
        </div>
      </section>

      <section
        v-else-if="activeSettingsContext === 'language'"
        class="h-full min-h-0 space-y-4 overflow-y-auto px-6 py-5"
        data-testid="settings-context-language"
      >
        <div>
          <p class="text-2xl font-semibold text-white">Idioma</p>
          <p class="mt-2 max-w-2xl text-sm leading-6 text-apollo-app-muted">
            Defina os idiomas usados no OCR, nas respostas e futuramente na
            interface.
          </p>
        </div>

        <div class="mt-5 grid grid-cols-1 gap-3 sm:grid-cols-2">
          <div
            class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6"
          >
            <p class="text-sm font-semibold text-white">Idioma de Entrada</p>
            <p class="mt-1 text-sm text-apollo-app-muted">
              Idioma do texto que sera extraido da imagem pelo Tesseract.
            </p>
            <div class="mt-4">
              <select
                class="w-full rounded-lg border border-apollo-app-border bg-apollo-app-shell px-4 py-2.5 text-sm text-white outline-none transition focus:border-apollo-app-accent"
                :value="settings.ocr_language"
                @change="
                  changeOcrLanguage(($event.target as HTMLSelectElement).value)
                "
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

          <div
            class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6"
          >
            <p class="text-sm font-semibold text-white">Idioma de Saida</p>
            <p class="mt-1 text-sm text-apollo-app-muted">
              A IA sempre respondera neste idioma, independente do idioma da
              entrada.
            </p>
            <div class="mt-4">
              <select
                class="w-full rounded-lg border border-apollo-app-border bg-apollo-app-shell px-4 py-2.5 text-sm text-white outline-none transition focus:border-apollo-app-accent"
                :value="settings.output_language"
                @change="
                  changeOutputLanguage(
                    ($event.target as HTMLSelectElement).value
                  )
                "
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

        <div
          class="rounded-xl border border-dashed border-apollo-app-border bg-apollo-app-card/60 p-6"
        >
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div>
              <p class="text-sm font-semibold text-white">
                Idioma da Aplicacao
              </p>
              <p class="mt-1 text-sm text-apollo-app-muted">
                Escolha o idioma da interface do Apollo.
              </p>
            </div>
            <span
              class="rounded-full border border-apollo-app-border px-3 py-1 text-xs text-apollo-app-muted"
            >
              Futuro
            </span>
          </div>
          <button
            class="mt-4 w-full cursor-not-allowed rounded-lg border border-apollo-app-border bg-apollo-app-shell px-4 py-2.5 text-left text-sm text-apollo-app-muted"
            type="button"
            disabled
          >
            Seguir idioma do sistema
          </button>
        </div>
      </section>

      <section
        v-else
        class="h-full min-h-0 space-y-4 overflow-y-auto px-6 py-5"
        data-testid="settings-context-shortcuts"
      >
        <div>
          <p class="text-2xl font-semibold text-white">Atalhos</p>
          <p class="mt-2 max-w-2xl text-sm leading-6 text-apollo-app-muted">
            Personalize os atalhos para corresponder ao seu fluxo de trabalho.
          </p>
          <p
            v-if="hasShortcutValidationErrors"
            class="mt-3 rounded-lg border border-red-400/25 bg-red-500/10 px-4 py-3 text-sm text-red-100"
            data-testid="shortcut-conflict-summary"
          >
            Resolva os atalhos duplicados antes de salvar as configuracoes.
          </p>
        </div>

        <div
          v-for="(shortcut, index) in settings.shortcuts"
          :key="`${shortcut.action}-${index}`"
          class="rounded-xl border border-apollo-app-border bg-apollo-app-card p-6"
        >
          <div class="flex flex-wrap items-start justify-between gap-4">
            <div class="min-w-0">
              <p
                class="text-base font-semibold text-white"
                :data-testid="`shortcut-label-${index}`"
              >
                {{ shortcutActionLabel(shortcut.action) }}
              </p>
              <p class="mt-1 text-sm text-apollo-app-muted">Atalho do Apollo</p>
            </div>
            <label
              class="inline-flex shrink-0 items-center gap-2 text-xs text-apollo-app-muted"
            >
              <input
                class="h-4 w-4 rounded border-apollo-app-border bg-apollo-app-shell"
                type="checkbox"
                :checked="shortcut.enabled"
                @change="
                  changeShortcutEnabled(
                    index,
                    ($event.target as HTMLInputElement).checked
                  )
                "
              />
              Ativo
            </label>
          </div>

          <div
            class="mt-4 flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between"
          >
            <div class="text-xs text-apollo-app-muted">
              <span v-if="isShortcutRecording(index)"
                >Pressione a nova combinacao. Esc cancela.</span
              >
              <span v-else
                >Clique no campo e pressione a combinacao desejada.</span
              >
            </div>

            <div class="flex w-full flex-col gap-2 sm:w-72">
              <button
                class="flex min-h-11 w-full items-center justify-center gap-2 rounded-lg border px-3 py-2 text-sm outline-none transition"
                data-testid="shortcut-recorder"
                type="button"
                :aria-label="`Definir atalho para ${shortcutActionLabel(shortcut.action)}`"
                :class="
                  shortcutValidationMessage(index)
                    ? 'border-red-400/50 bg-red-500/10 text-red-50 focus:border-red-300'
                    : isShortcutRecording(index)
                      ? 'border-apollo-app-accent bg-apollo-app-selected text-white'
                      : 'border-apollo-app-border bg-apollo-app-shell text-white hover:border-apollo-app-selectedBorder focus:border-apollo-app-accent'
                "
                @click="startShortcutRecording(index)"
                @keydown="handleShortcutRecorderKeydown(index, $event)"
                @blur="stopShortcutRecording(index)"
              >
                <span v-if="isShortcutRecording(index)" class="font-medium">
                  Gravando...
                </span>
                <template v-else>
                  <span
                    v-for="(key, keyIndex) in shortcutDisplayParts(
                      shortcut.accelerator
                    )"
                    :key="keyIndex"
                    class="rounded-md border border-apollo-app-border bg-apollo-app-card px-2.5 py-1 text-xs font-semibold"
                  >
                    {{ key }}
                  </span>
                </template>
              </button>

              <p
                v-if="shortcutValidationMessage(index)"
                class="text-xs text-red-100"
                :data-testid="`shortcut-validation-${index}`"
              >
                {{ shortcutValidationMessage(index) }}
              </p>
            </div>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
