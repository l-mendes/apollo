<script setup lang="ts">
import { ArrowLeft, History, House, Settings2 } from "lucide-vue-next";
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";

import HistorySurface from "@/components/surfaces/HistorySurface.vue";
import HomeSurface from "@/components/surfaces/HomeSurface.vue";
import SettingsSurface from "@/components/surfaces/SettingsSurface.vue";
import {
  analyzeCapture,
  applyGlobalShortcuts,
  captureScreenRegion,
  cloneSettings,
  commandErrorMessage,
  continueConversation,
  createEmptyProviderCatalog,
  listHistory,
  loadConversationMessages,
  listProviderModelsFor,
  runOcrOnImage,
  saveSettings,
  PROVIDER_OPTIONS,
  type AnalyzeCaptureResponse,
  type ConversationMessage,
  type ProviderKind,
  type UserSettings,
  loadSettings,
  type InteractionSession
} from "@/composables/useApolloDesktop";
import {
  emitPreviewAnalysisStatus,
  emitSurfaceChanged,
  emitToPreviewWindow,
  emitToResponseWindow,
  hidePreviewWindow,
  listenForAppCloseToHide,
  listenForOcrResult,
  listenForPreviewCancel,
  listenForPreviewConfirm,
  listenForSelectionCancelled,
  listenForSelectionResult,
  listenForShortcutAction,
  listenForStartAreaCapture,
  listenForSurfaceNavigation,
  openPreviewWindow,
  openResponseWindow,
  openSelectionWindow,
  syncAppWindowAppearance,
  type AppSurface
} from "@/composables/useWindowShell";
import {
  fetchBootstrapSnapshot,
  fetchHealthStatus,
  type BootstrapSnapshot,
  type HealthStatus
} from "@/composables/useDesktopCapabilities";

function resolveInitialSurface(): AppSurface {
  const surface = new URL(window.location.href).searchParams.get("surface");

  if (surface === "home" || surface === "history" || surface === "settings") {
    return surface;
  }

  return "home";
}

const activeSurface = ref<AppSurface>(resolveInitialSurface());
const health = ref<HealthStatus | null>(null);
const snapshot = ref<BootstrapSnapshot | null>(null);

const settings = ref<UserSettings | null>(null);
const settingsDraft = ref<UserSettings | null>(null);
const providerCatalog = ref(createEmptyProviderCatalog());
const historyItems = ref<InteractionSession[]>([]);
const selectedHistoryId = ref<string | null>(null);

const settingsLoading = ref(true);
const settingsSaving = ref(false);
const settingsError = ref<string | null>(null);
const providerCatalogLoading = ref(true);
const providerCatalogError = ref<string | null>(null);
const historyLoading = ref(true);
const historyError = ref<string | null>(null);

const ocrText = ref("");
const userNotes = ref("");
const analysisLoading = ref(false);
const analysisError = ref<string | null>(null);
const lastResponse = ref<AnalyzeCaptureResponse["response"] | null>(null);
const lastPrompt = ref("");
const conversationMessages = ref<ConversationMessage[]>([]);
const conversationLoading = ref(false);
const conversationError = ref<string | null>(null);
const continuePrompt = ref("");
const continueLoading = ref(false);
const continueError = ref<string | null>(null);

interface PendingCapture {
  image_path: string;
  width: number;
  height: number;
  data_url: string;
}

const pendingCapture = ref<PendingCapture | null>(null);
const captureProcessing = ref(false);

let unlistenSurfaceNavigation: (() => void) | null = null;
let unlistenCloseRequest: (() => void) | null = null;
let unlistenOcrResult: (() => void) | null = null;
let unlistenShortcutAction: (() => void) | null = null;
let unlistenStartAreaCapture: (() => void) | null = null;
let unlistenSelectionResult: (() => void) | null = null;
let unlistenSelectionCancelled: (() => void) | null = null;
let unlistenPreviewConfirm: (() => void) | null = null;
let unlistenPreviewCancel: (() => void) | null = null;

const versionText = computed(() => health.value?.version ?? snapshot.value?.metadata.version ?? "v0.1.0");

const sections = [
  {
    id: "home",
    label: "Home",
    description: "Captura, contexto e resposta",
    icon: House
  },
  {
    id: "history",
    label: "Historico",
    description: "Sessoes persistidas",
    icon: History
  },
  {
    id: "settings",
    label: "Configuracoes",
    description: "Provider, prompt e atalhos",
    icon: Settings2
  }
] as const;

const sectionSummary = computed(() => {
  if (activeSurface.value === "home") {
    return {
      title: "Captura e Analise",
      description:
        "Monte o contexto, execute a analise e acompanhe a resposta em tempo real."
    };
  }

  if (activeSurface.value === "history") {
    return {
      title: "Historico de Sessoes",
      description:
        "Revise OCR, prompt, notas e resposta em sessoes salvas no backend local."
    };
  }

  return {
    title: "Configuracoes",
    description:
      "Defina o provider padrao, modelo principal, prompt base e os atalhos do workspace."
  };
});

const activeSection = computed(
  () => sections.find((section) => section.id === activeSurface.value) ?? sections[0]
);

const hasUnsavedSettings = computed(() => {
  if (!settings.value || !settingsDraft.value) {
    return false;
  }

  return JSON.stringify(settings.value) !== JSON.stringify(settingsDraft.value);
});

const homeErrorText = computed(() => {
  if (!settings.value) {
    return settingsError.value;
  }

  return providerCatalogError.value;
});

const historyPanelErrorText = computed(() => historyError.value);
const settingsPanelErrorText = computed(() => settingsError.value);

const appStatusText = computed(() => {
  if (analysisLoading.value) {
    return "Analise em andamento";
  }

  if (settingsSaving.value) {
    return "Salvando configuracoes";
  }

  if (activeSurface.value === "history") {
    return historyLoading.value ? "Carregando historico" : `${historyItems.value.length} sessao(oes) carregadas`;
  }

  if (activeSurface.value === "settings") {
    return hasUnsavedSettings.value ? "Alteracoes locais presentes" : "Preferencias sincronizadas";
  }

  if (lastResponse.value) {
    return `Resposta pronta em ${lastResponse.value.model_key}`;
  }

  return "Janela principal pronta";
});

onMounted(async () => {
  await syncAppWindowAppearance();
  void emitSurfaceChanged(activeSurface.value);
  unlistenSurfaceNavigation = await listenForSurfaceNavigation((surface) => {
    activateSurface(surface);
  });
  unlistenCloseRequest = await listenForAppCloseToHide();
  unlistenOcrResult = await listenForOcrResult((text) => {
    // Legacy fallback path: keep populating the OCR field if some other
    // codepath still emits raw OCR results.
    ocrText.value = text;
    activateSurface("home");
  });
  unlistenShortcutAction = await listenForShortcutAction((action) => {
    if (action === "open_settings") activateSurface("settings");
    else if (action === "open_history") activateSurface("history");
  });
  unlistenStartAreaCapture = await listenForStartAreaCapture(() => {
    void handleCapture();
  });
  unlistenSelectionResult = await listenForSelectionResult((rect) => {
    void handleSelectionResult(rect);
  });
  unlistenSelectionCancelled = await listenForSelectionCancelled(() => {
    captureProcessing.value = false;
  });
  unlistenPreviewConfirm = await listenForPreviewConfirm((payload) => {
    void confirmPendingCapture(payload.user_notes);
  });
  unlistenPreviewCancel = await listenForPreviewCancel(() => {
    discardPendingCapture();
  });

  void loadSystemStatus();
  void Promise.allSettled([refreshSettings(), refreshProviderCatalog(), refreshHistory()]);
});

onBeforeUnmount(() => {
  unlistenSurfaceNavigation?.();
  unlistenCloseRequest?.();
  unlistenOcrResult?.();
  unlistenShortcutAction?.();
  unlistenStartAreaCapture?.();
  unlistenSelectionResult?.();
  unlistenSelectionCancelled?.();
  unlistenPreviewConfirm?.();
  unlistenPreviewCancel?.();
});


watch(lastResponse, (next) => {
  if (next) {
    void openResponseWindow();
    void emitToResponseWindow({
      response: next.answer,
      request_prompt: lastPrompt.value
    });
  }
});

watch(selectedHistoryId, (sessionId) => {
  if (!sessionId) {
    conversationMessages.value = [];
    conversationError.value = null;
    continuePrompt.value = "";
    return;
  }

  void refreshConversation(sessionId);
});

async function loadSystemStatus() {
  try {
    const [healthStatus, bootstrap] = await Promise.all([
      fetchHealthStatus(),
      fetchBootstrapSnapshot()
    ]);

    health.value = healthStatus;
    snapshot.value = bootstrap;
  } catch {
    health.value = {
      appName: "Apollo",
      status: "Modo web para desenvolvimento",
      version: "v0.1.0"
    };
  }
}

function pickModel(providerKind: ProviderKind, preferredModel: string): string {
  const models = providerCatalog.value[providerKind] ?? [];

  if (models.some((model) => model.model_key === preferredModel)) {
    return preferredModel;
  }

  return models.find((model) => model.is_default)?.model_key ?? models[0]?.model_key ?? preferredModel;
}

function normalizeSettings(nextSettings: UserSettings): UserSettings {
  const normalizedSettings = cloneSettings(nextSettings);
  normalizedSettings.preferred_model = pickModel(
    normalizedSettings.preferred_provider,
    normalizedSettings.preferred_model
  );

  return normalizedSettings;
}

function updateSettingsDraft(nextSettings: UserSettings) {
  settingsDraft.value = normalizeSettings(nextSettings);
}

function activateSurface(surface: AppSurface) {
  activeSurface.value = surface;
  void emitSurfaceChanged(surface);
}

async function refreshSettings() {
  settingsLoading.value = true;
  settingsError.value = null;

  try {
    const loadedSettings = await loadSettings();
    settings.value = normalizeSettings(loadedSettings);
    settingsDraft.value = cloneSettings(settings.value);
    void applyGlobalShortcuts(settings.value.shortcuts).catch(() => {});
  } catch (error) {
    settingsError.value = commandErrorMessage(
      error,
      "Nao foi possivel carregar as configuracoes persistidas."
    );
  } finally {
    settingsLoading.value = false;
  }
}

async function refreshProviderCatalog() {
  providerCatalogLoading.value = true;
  providerCatalogError.value = null;

  try {
    const catalogEntries = await Promise.allSettled(
      PROVIDER_OPTIONS.map(async ({ kind }) => [kind, await listProviderModelsFor(kind)] as const)
    );
    const nextCatalog = createEmptyProviderCatalog();
    let rejectedCount = 0;

    for (const entry of catalogEntries) {
      if (entry.status === "fulfilled") {
        const [providerKind, models] = entry.value;
        nextCatalog[providerKind] = models;
      } else {
        rejectedCount += 1;
      }
    }

    providerCatalog.value = nextCatalog;

    if (rejectedCount > 0) {
      providerCatalogError.value =
        rejectedCount === PROVIDER_OPTIONS.length
          ? "Nao foi possivel carregar o catalogo de modelos do Apollo."
          : "Parte do catalogo de modelos nao ficou disponivel neste bootstrap.";
    }

    if (settings.value) {
      settings.value = normalizeSettings(settings.value);
      settingsDraft.value = cloneSettings(settings.value);
    }
  } catch (error) {
    providerCatalogError.value = commandErrorMessage(
      error,
      "Nao foi possivel carregar o catalogo de modelos."
    );
  } finally {
    providerCatalogLoading.value = false;
  }
}

async function refreshHistory() {
  historyLoading.value = true;
  historyError.value = null;

  try {
    const sessions = await listHistory();
    historyItems.value = sessions;
    selectedHistoryId.value = sessions[0]?.id ?? null;
  } catch (error) {
    historyError.value = commandErrorMessage(error, "Nao foi possivel carregar o historico.");
  } finally {
    historyLoading.value = false;
  }
}

async function refreshConversation(sessionId: string) {
  conversationLoading.value = true;
  conversationError.value = null;

  try {
    conversationMessages.value = await loadConversationMessages(sessionId);
  } catch (error) {
    conversationError.value = commandErrorMessage(
      error,
      "Nao foi possivel carregar a conversa persistida desta sessao."
    );
  } finally {
    conversationLoading.value = false;
  }
}

function showSurface(surface: AppSurface) {
  activateSurface(surface);
}

async function handleAnalyze() {
  if (!settingsDraft.value || !ocrText.value.trim()) {
    return;
  }

  analysisLoading.value = true;
  analysisError.value = null;

  try {
    const outputLang = settingsDraft.value.output_language?.trim();
    const effectiveBasePrompt = outputLang
      ? `${settingsDraft.value.base_prompt}\n\nSempre responda em ${outputLang}.`
      : settingsDraft.value.base_prompt;

    const result = await analyzeCapture({
      provider_kind: settingsDraft.value.preferred_provider,
      model_key: settingsDraft.value.preferred_model,
      base_prompt: effectiveBasePrompt,
      ocr_text: ocrText.value.trim(),
      user_notes: userNotes.value.trim() ? userNotes.value.trim() : null,
      conversation_context: []
    });

    lastPrompt.value = result.prompt;
    lastResponse.value = result.response;
    historyItems.value = [
      result.session,
      ...historyItems.value.filter((session) => session.id !== result.session.id)
    ];
    selectedHistoryId.value = result.session.id;
    continuePrompt.value = "";
    await refreshConversation(result.session.id);
  } catch (error) {
    analysisError.value = commandErrorMessage(
      error,
      "Nao foi possivel executar a analise deste contexto."
    );
  } finally {
    analysisLoading.value = false;
  }
}

async function handleContinueConversation() {
  const selectedSession = historyItems.value.find(
    (session) => session.id === selectedHistoryId.value
  );

  if (!selectedSession || !continuePrompt.value.trim()) {
    return;
  }

  continueLoading.value = true;
  continueError.value = null;

  try {
    const result = await continueConversation({
      session_id: selectedSession.id,
      provider_kind: selectedSession.provider_kind,
      model_key: selectedSession.model_key,
      prompt: continuePrompt.value.trim(),
      existing_messages: conversationMessages.value
    });

    conversationMessages.value = [
      ...conversationMessages.value,
      ...result.appended_messages
    ];
    continuePrompt.value = "";
    lastResponse.value = result.response;

    historyItems.value = historyItems.value.map((session) =>
      session.id === selectedSession.id
        ? {
            ...session,
            source_kind: "ManualText",
            user_notes: result.appended_messages[0]?.content ?? session.user_notes,
            response_text: result.response.answer
          }
        : session
    );
  } catch (error) {
    continueError.value = commandErrorMessage(
      error,
      "Nao foi possivel continuar a conversa desta sessao."
    );
  } finally {
    continueLoading.value = false;
  }
}

async function handleSaveSettings() {
  if (!settingsDraft.value) {
    return;
  }

  settingsSaving.value = true;
  settingsError.value = null;

  try {
    await saveSettings(settingsDraft.value);
    settings.value = cloneSettings(settingsDraft.value);
    settingsDraft.value = cloneSettings(settings.value);
    void applyGlobalShortcuts(settings.value.shortcuts).catch(() => {});
  } catch (error) {
    settingsError.value = commandErrorMessage(
      error,
      "Nao foi possivel persistir as configuracoes atuais."
    );
  } finally {
    settingsSaving.value = false;
  }
}

async function handleCapture() {
  if (captureProcessing.value) {
    return;
  }

  analysisError.value = null;
  captureProcessing.value = true;

  try {
    await openSelectionWindow();
  } catch (error) {
    captureProcessing.value = false;
    analysisError.value = commandErrorMessage(
      error,
      "Nao foi possivel abrir a area de selecao."
    );
  }
}

async function handleSelectionResult(rect: {
  x: number;
  y: number;
  width: number;
  height: number;
}) {
  try {
    const result = await captureScreenRegion(rect);
    pendingCapture.value = {
      image_path: result.image_path,
      width: result.width,
      height: result.height,
      data_url: result.data_url
    };

    await openPreviewWindow();
    await emitToPreviewWindow({
      image_data_url: result.data_url,
      image_width: result.width,
      image_height: result.height,
      has_capture: true
    });
  } catch (error) {
    analysisError.value = commandErrorMessage(
      error,
      "Nao foi possivel capturar a area selecionada."
    );
    captureProcessing.value = false;
  }
}

async function confirmPendingCapture(notes: string) {
  const capture = pendingCapture.value;
  if (!capture) {
    captureProcessing.value = false;
    return;
  }

  analysisLoading.value = true;
  analysisError.value = null;
  userNotes.value = notes;

  try {
    // Phase 1: OCR
    await emitPreviewAnalysisStatus({ status: "ocr", message: "Extraindo texto da imagem..." });
    const text = await runOcrOnImage(capture.image_path, settingsDraft.value?.ocr_language ?? 'por');
    ocrText.value = text;

    // Phase 2: Analyze
    await emitPreviewAnalysisStatus({ status: "analyzing", message: "Analisando com o provider..." });
    pendingCapture.value = null;
    await handleAnalyze();

    // Phase 3: Done — tell preview to hide, response window is opened by the watch(lastResponse)
    await emitPreviewAnalysisStatus({ status: "done", message: "" });
  } catch (error) {
    const msg = commandErrorMessage(error, "Nao foi possivel concluir a analise.");
    analysisError.value = msg;
    await emitPreviewAnalysisStatus({ status: "error", message: msg });
  } finally {
    analysisLoading.value = false;
    captureProcessing.value = false;
  }
}

function discardPendingCapture() {
  pendingCapture.value = null;
  captureProcessing.value = false;
}


</script>

<template>
  <div class="apollo-main-shell flex h-screen overflow-hidden bg-apollo-app-shell text-slate-50">
    <aside class="flex w-60 shrink-0 flex-col border-r border-apollo-app-border bg-apollo-app-sidebar">
      <button
        class="flex items-center px-5 py-5 text-apollo-app-muted transition hover:text-white"
        type="button"
        @click="activateSurface('home')"
      >
        <ArrowLeft class="h-5 w-5" />
      </button>

      <nav class="flex-1 space-y-1 px-3">
        <button
          v-for="section in sections"
          :key="section.id"
          class="flex w-full items-center gap-3 rounded-xl px-4 py-3 text-left text-sm transition"
          :class="
            activeSurface === section.id
              ? 'bg-apollo-app-selected font-medium text-white'
              : 'text-apollo-app-muted hover:bg-apollo-app-hover hover:text-white'
          "
          type="button"
          @click="activateSurface(section.id)"
        >
          <component
            :is="section.icon"
            class="h-5 w-5 shrink-0"
          />
          {{ section.label }}
        </button>
      </nav>

      <div class="border-t border-apollo-app-border px-5 py-4 text-xs text-apollo-app-muted">
        {{ versionText }}
      </div>
    </aside>

    <main class="min-w-0 flex-1 overflow-y-auto bg-apollo-app-panel">
      <div class="mx-auto max-w-4xl px-10 py-10">
        <h1 class="text-2xl font-semibold text-white">{{ sectionSummary.title }}</h1>
        <p class="mt-3 max-w-2xl text-sm leading-6 text-apollo-app-muted">{{ sectionSummary.description }}</p>

        <div class="mt-8">
          <Transition name="surface-float" mode="out-in">
            <HomeSurface
              v-if="activeSurface === 'home'"
              :loading="settingsLoading || providerCatalogLoading"
              :error-text="homeErrorText"
              :settings="settingsDraft"
              :is-analyzing="analysisLoading"
              :analysis-error-text="analysisError"
              @capture="handleCapture"
              @open-settings="showSurface('settings')"
              @open-history="showSurface('history')"
            />

            <HistorySurface
              v-else-if="activeSurface === 'history'"
              :loading="historyLoading"
              :error-text="historyPanelErrorText"
              :sessions="historyItems"
              :selected-session-id="selectedHistoryId"
              :conversation-messages="conversationMessages"
              :conversation-loading="conversationLoading"
              :conversation-error-text="conversationError"
              :continue-prompt="continuePrompt"
              :continue-loading="continueLoading"
              :continue-error-text="continueError"
              @select-session="selectedHistoryId = $event"
              @update:continue-prompt="continuePrompt = $event"
              @continue-conversation="handleContinueConversation"
              @open-home="showSurface('home')"
            />

            <SettingsSurface
              v-else
              :loading="settingsLoading || providerCatalogLoading"
              :saving="settingsSaving"
              :error-text="settingsPanelErrorText ?? providerCatalogError"
              :settings="settingsDraft"
              :models-by-provider="providerCatalog"
              :has-unsaved-changes="hasUnsavedSettings"
              @update:settings="updateSettingsDraft($event)"
              @save="handleSaveSettings"
            />
          </Transition>
        </div>
      </div>
    </main>
  </div>
</template>
