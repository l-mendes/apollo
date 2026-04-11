<script setup lang="ts">
import { History, House, Settings2 } from "lucide-vue-next";
import { computed, onBeforeUnmount, onMounted, watch } from "vue";

import { buildChatMessages } from "@/components/chat/chatMessages";
import HistorySurface from "@/components/surfaces/HistorySurface.vue";
import HomeSurface from "@/components/surfaces/HomeSurface.vue";
import SettingsSurface from "@/components/surfaces/SettingsSurface.vue";
import {
  analyzeCapture,
  applyGlobalShortcuts,
  captureScreenRegion,
  clearHistory,
  cloneSettings,
  commandErrorMessage,
  createEmptyProviderCatalog,
  deleteHistorySession,
  type AnalyzeCaptureResponse,
  type CaptureRegionRequest,
  type InteractionSession,
  listHistory,
  listProviderModelsFor,
  loadConversationMessages,
  loadSettings,
  type ProviderModel,
  runOcrOnImage,
  saveSettings,
  PROVIDER_OPTIONS,
  type ProviderKind,
  type UserSettings
} from "@/composables/useApolloDesktop";
import {
  emitPreviewAnalysisStatus,
  emitSurfaceChanged,
  emitToPreviewWindow,
  emitToResponseWindow,
  hideResponseWindow,
  listenForAppCloseToHide,
  listenForOcrResult,
  listenForPreviewCancel,
  listenForPreviewConfirm,
  listenForResponseConversationSync,
  listenForSelectionCancelled,
  listenForSelectionResult,
  listenForShortcutAction,
  listenForStartAreaCapture,
  listenForSurfaceNavigation,
  openPreviewWindow,
  openResponseWindow,
  openSelectionWindow,
  revealCurrentWindow,
  syncAppWindowAppearance,
  type AppSurface,
  type ResponseUpdatePayload
} from "@/composables/useWindowShell";
import {
  fetchBootstrapSnapshot,
  fetchHealthStatus,
  requestQuit,
  type HealthStatus
} from "@/composables/useDesktopCapabilities";
import { useApolloStore } from "@/store/apollo";

function resolveInitialSurface(): AppSurface {
  const surface = new URL(window.location.href).searchParams.get("surface");

  if (surface === "home" || surface === "history" || surface === "settings") {
    return surface;
  }

  return "home";
}

const store = useApolloStore();
store.commit("setActiveSurface", resolveInitialSurface());

let unlistenSurfaceNavigation: (() => void) | null = null;
let unlistenCloseRequest: (() => void) | null = null;
let unlistenOcrResult: (() => void) | null = null;
let unlistenShortcutAction: (() => void) | null = null;
let unlistenStartAreaCapture: (() => void) | null = null;
let unlistenSelectionResult: (() => void) | null = null;
let unlistenSelectionCancelled: (() => void) | null = null;
let unlistenPreviewConfirm: (() => void) | null = null;
let unlistenPreviewCancel: (() => void) | null = null;
let unlistenResponseConversationSync: (() => void) | null = null;
let shortcutsSuppressedForRecording = false;
let autoSaveSettingsTimer: ReturnType<typeof setTimeout> | null = null;

const activeSurface = computed(() => store.state.shell.activeSurface);
const versionText = computed(() => store.getters.versionText as string);

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
  if (store.state.shell.activeSurface === "home") {
    return {
      title: "Captura e Analise",
      description:
        "Monte o contexto, execute a analise e acompanhe a resposta em tempo real."
    };
  }

  if (store.state.shell.activeSurface === "history") {
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

onMounted(async () => {
  await syncAppWindowAppearance();
  void emitSurfaceChanged(store.state.shell.activeSurface);
  unlistenSurfaceNavigation = await listenForSurfaceNavigation((surface) => {
    activateSurface(surface);
  });
  unlistenCloseRequest = await listenForAppCloseToHide();
  unlistenOcrResult = await listenForOcrResult((text) => {
    store.commit("patchAnalysisState", {
      ocrText: text
    });
    activateSurface("home");
  });
  unlistenShortcutAction = await listenForShortcutAction((action) => {
    const targetSurface = shortcutSurfaceTarget(action);

    if (targetSurface) {
      activateSurface(targetSurface);
      void revealCurrentWindow();
    }
  });
  unlistenStartAreaCapture = await listenForStartAreaCapture(() => {
    void handleCapture();
  });
  unlistenSelectionResult = await listenForSelectionResult((rect) => {
    void handleSelectionResult(rect);
  });
  unlistenSelectionCancelled = await listenForSelectionCancelled(() => {
    store.commit("patchAnalysisState", {
      captureProcessing: false
    });
  });
  unlistenPreviewConfirm = await listenForPreviewConfirm((payload) => {
    void confirmPendingCapture(payload.user_notes);
  });
  unlistenPreviewCancel = await listenForPreviewCancel(() => {
    discardPendingCapture();
  });
  unlistenResponseConversationSync = await listenForResponseConversationSync(
    (payload) => {
      syncResponseConversation(
        payload.session_id,
        payload.prompt,
        payload.response,
        payload.appended_messages
      );
    }
  );

  void loadSystemStatus();
  void Promise.allSettled([
    refreshSettings(),
    refreshProviderCatalog(),
    refreshHistory()
  ]);
});

onBeforeUnmount(() => {
  if (autoSaveSettingsTimer) {
    clearTimeout(autoSaveSettingsTimer);
    autoSaveSettingsTimer = null;
  }

  unlistenSurfaceNavigation?.();
  unlistenCloseRequest?.();
  unlistenOcrResult?.();
  unlistenShortcutAction?.();
  unlistenStartAreaCapture?.();
  unlistenSelectionResult?.();
  unlistenSelectionCancelled?.();
  unlistenPreviewConfirm?.();
  unlistenPreviewCancel?.();
  unlistenResponseConversationSync?.();
});

watch(
  () => store.state.history.selectedHistoryId,
  (sessionId) => {
    if (!sessionId) {
      store.commit("patchHistoryState", {
        conversationMessages: [],
        conversationError: null,
        continuePrompt: "",
        pendingFollowUp: null
      });
      return;
    }

    void refreshConversation(sessionId);
  }
);

watch(activeSurface, (surface, previousSurface) => {
  if (surface !== previousSurface) {
    void emitSurfaceChanged(surface);
  }
});

watch(
  () => store.state.settings.draft,
  () => {
    scheduleAutoSaveSettings();
  },
  { deep: true }
);

async function loadSystemStatus() {
  try {
    const [healthStatus, bootstrap] = await Promise.all([
      fetchHealthStatus(),
      fetchBootstrapSnapshot()
    ]);

    store.commit("patchShellState", {
      health: healthStatus,
      snapshot: bootstrap
    });
  } catch {
    const health: HealthStatus = {
      appName: "Apollo",
      status: "Modo web para desenvolvimento",
      version: "v0.1.0"
    };

    store.commit("patchShellState", { health });
  }
}

function pickModel(providerKind: ProviderKind, preferredModel: string): string {
  const models = (store.state.settings.providerCatalog[providerKind] ??
    []) as ProviderModel[];

  if (
    models.some((model: ProviderModel) => model.model_key === preferredModel)
  ) {
    return preferredModel;
  }

  return (
    models.find((model: ProviderModel) => model.is_default)?.model_key ??
    models[0]?.model_key ??
    preferredModel
  );
}

function normalizeSettings(nextSettings: UserSettings): UserSettings {
  const normalizedSettings = cloneSettings(nextSettings);
  normalizedSettings.preferred_model = pickModel(
    normalizedSettings.preferred_provider,
    normalizedSettings.preferred_model
  );

  return normalizedSettings;
}

function activateSurface(surface: AppSurface) {
  store.commit("setActiveSurface", surface);
}

function shortcutSurfaceTarget(action: string): AppSurface | null {
  const actionLower = action.toLowerCase();

  if (actionLower.includes("history") || actionLower.includes("histor")) {
    return "history";
  }

  if (actionLower.includes("settings") || actionLower.includes("config")) {
    return "settings";
  }

  return null;
}

async function refreshSettings() {
  store.commit("patchSettingsState", {
    loading: true,
    error: null
  });

  try {
    const loadedSettings = await loadSettings();
    const normalizedSettings = normalizeSettings(loadedSettings);

    store.commit("patchSettingsState", {
      saved: normalizedSettings,
      draft: cloneSettings(normalizedSettings)
    });

    void applyGlobalShortcuts(normalizedSettings.shortcuts).catch(() => {});
  } catch (error) {
    store.commit("patchSettingsState", {
      error: commandErrorMessage(
        error,
        "Nao foi possivel carregar as configuracoes persistidas."
      )
    });
  } finally {
    store.commit("patchSettingsState", {
      loading: false
    });
  }
}

async function refreshProviderCatalog() {
  store.commit("patchSettingsState", {
    providerCatalogLoading: true,
    providerCatalogError: null
  });

  try {
    const catalogEntries = await Promise.allSettled(
      PROVIDER_OPTIONS.map(
        async ({ kind }) => [kind, await listProviderModelsFor(kind)] as const
      )
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

    store.commit("patchSettingsState", {
      providerCatalog: nextCatalog
    });

    if (rejectedCount > 0) {
      store.commit("patchSettingsState", {
        providerCatalogError:
          rejectedCount === PROVIDER_OPTIONS.length
            ? "Nao foi possivel carregar o catalogo de modelos do Apollo."
            : "Parte do catalogo de modelos nao ficou disponivel neste bootstrap."
      });
    }

    if (store.state.settings.saved) {
      const normalizedSettings = normalizeSettings(store.state.settings.saved);
      store.commit("patchSettingsState", {
        saved: normalizedSettings,
        draft: cloneSettings(normalizedSettings)
      });
    }
  } catch (error) {
    store.commit("patchSettingsState", {
      providerCatalogError: commandErrorMessage(
        error,
        "Nao foi possivel carregar o catalogo de modelos."
      )
    });
  } finally {
    store.commit("patchSettingsState", {
      providerCatalogLoading: false
    });
  }
}

async function refreshHistory() {
  store.commit("patchHistoryState", {
    loading: true,
    error: null
  });

  try {
    const sessions = await listHistory();
    store.commit("patchHistoryState", {
      items: sessions,
      selectedHistoryId: sessions[0]?.id ?? null
    });
  } catch (error) {
    store.commit("patchHistoryState", {
      error: commandErrorMessage(
        error,
        "Nao foi possivel carregar o historico."
      )
    });
  } finally {
    store.commit("patchHistoryState", {
      loading: false
    });
  }
}

async function refreshConversation(sessionId: string) {
  store.commit("patchHistoryState", {
    conversationLoading: true,
    conversationError: null
  });

  try {
    const conversationMessages = await loadConversationMessages(sessionId);

    store.commit("patchHistoryState", {
      conversationMessages
    });

    return conversationMessages;
  } catch (error) {
    store.commit("patchHistoryState", {
      conversationError: commandErrorMessage(
        error,
        "Nao foi possivel carregar a conversa persistida desta sessao."
      )
    });
  } finally {
    store.commit("patchHistoryState", {
      conversationLoading: false
    });
  }

  return [];
}

function mergeConversationMessages(
  currentMessages: typeof store.state.history.conversationMessages,
  appendedMessages: typeof store.state.history.conversationMessages
) {
  const mergedMessages = [...currentMessages];
  const knownMessageIds = new Set(currentMessages.map((message) => message.id));

  for (const message of appendedMessages) {
    if (!knownMessageIds.has(message.id)) {
      mergedMessages.push(message);
      knownMessageIds.add(message.id);
    }
  }

  return mergedMessages;
}

function buildResponsePayload(
  session: InteractionSession,
  conversationMessages: typeof store.state.history.conversationMessages
): ResponseUpdatePayload {
  return {
    session_id: session.id,
    provider_kind: session.provider_kind,
    model_key: session.model_key,
    reasoning_effort:
      store.state.settings.draft?.reasoning_effort ??
      store.state.settings.saved?.reasoning_effort ??
      "medium",
    display_messages: buildChatMessages(session, conversationMessages),
    conversation_messages: conversationMessages
  };
}

async function syncResponseWindow(
  session: InteractionSession,
  conversationMessages: typeof store.state.history.conversationMessages
) {
  await openResponseWindow();
  await emitToResponseWindow(
    buildResponsePayload(session, conversationMessages)
  );
}

function syncResponseConversation(
  sessionId: string,
  prompt: string,
  response: AnalyzeCaptureResponse["response"],
  appendedMessages: typeof store.state.history.conversationMessages
) {
  store.commit("patchHistoryState", {
    conversationMessages:
      store.state.history.selectedHistoryId === sessionId
        ? mergeConversationMessages(
            store.state.history.conversationMessages,
            appendedMessages
          )
        : store.state.history.conversationMessages,
    continuePrompt:
      store.state.history.selectedHistoryId === sessionId
        ? ""
        : store.state.history.continuePrompt,
    pendingFollowUp:
      store.state.history.selectedHistoryId === sessionId
        ? null
        : store.state.history.pendingFollowUp
  });
  store.commit("patchHistoryState", {
    items: store.state.history.items.map((session: InteractionSession) =>
      session.id === sessionId
        ? {
            ...session,
            source_kind: "ManualText",
            user_notes: prompt,
            response_text: response.answer
          }
        : session
    )
  });
  store.commit("patchAnalysisState", {
    lastResponse: response
  });
}

function resetHistoryConversationState() {
  store.commit("patchHistoryState", {
    selectedHistoryId: null,
    conversationMessages: [],
    conversationError: null,
    continuePrompt: "",
    pendingFollowUp: null,
    continueLoading: false,
    continueError: null
  });
}

async function openHistorySessionChat(sessionId: string) {
  const session = store.state.history.items.find(
    (item: InteractionSession) => item.id === sessionId
  );

  if (!session) {
    return;
  }

  store.commit("patchHistoryState", {
    selectedHistoryId: sessionId,
    conversationError: null
  });

  const conversationMessages = await refreshConversation(sessionId);

  await syncResponseWindow(session, conversationMessages);
}

async function handleDeleteHistorySession(sessionId: string) {
  store.commit("patchHistoryState", {
    error: null
  });

  try {
    await deleteHistorySession(sessionId);

    const wasSelectedSession =
      store.state.history.selectedHistoryId === sessionId;
    const remainingSessions = store.state.history.items.filter(
      (session: InteractionSession) => session.id !== sessionId
    );

    store.commit("patchHistoryState", {
      items: remainingSessions,
      selectedHistoryId: wasSelectedSession
        ? (remainingSessions[0]?.id ?? null)
        : store.state.history.selectedHistoryId,
      conversationMessages: wasSelectedSession
        ? []
        : store.state.history.conversationMessages,
      continuePrompt: wasSelectedSession
        ? ""
        : store.state.history.continuePrompt,
      pendingFollowUp: wasSelectedSession
        ? null
        : store.state.history.pendingFollowUp,
      continueError: wasSelectedSession
        ? null
        : store.state.history.continueError
    });

    if (wasSelectedSession) {
      await hideResponseWindow();
    }
  } catch (error) {
    store.commit("patchHistoryState", {
      error: commandErrorMessage(
        error,
        "Nao foi possivel excluir esta sessao do historico."
      )
    });
  }
}

async function handleClearHistory() {
  store.commit("patchHistoryState", {
    error: null
  });

  try {
    await clearHistory();
    store.commit("patchHistoryState", {
      items: []
    });
    resetHistoryConversationState();
    await hideResponseWindow();
  } catch (error) {
    store.commit("patchHistoryState", {
      error: commandErrorMessage(error, "Nao foi possivel limpar o historico.")
    });
  }
}

async function handleAnalyze() {
  const settingsDraft = store.state.settings.draft;
  const ocrText = store.state.analysis.ocrText.trim();

  if (!settingsDraft || !ocrText) {
    return;
  }

  store.commit("patchAnalysisState", {
    loading: true,
    error: null
  });

  try {
    const outputLang = settingsDraft.output_language?.trim();
    const effectiveBasePrompt = outputLang
      ? `${settingsDraft.base_prompt}\n\nSempre responda em ${outputLang}.`
      : settingsDraft.base_prompt;

    const result = await analyzeCapture({
      provider_kind: settingsDraft.preferred_provider,
      model_key: settingsDraft.preferred_model,
      reasoning_effort: settingsDraft.reasoning_effort,
      base_prompt: effectiveBasePrompt,
      ocr_text: ocrText,
      user_notes: store.state.analysis.userNotes.trim()
        ? store.state.analysis.userNotes.trim()
        : null,
      conversation_context: []
    });

    store.commit("patchAnalysisState", {
      lastPrompt: result.prompt,
      lastResponse: result.response
    });
    store.commit("patchHistoryState", {
      items: [
        result.session,
        ...store.state.history.items.filter(
          (session: InteractionSession) => session.id !== result.session.id
        )
      ],
      selectedHistoryId: result.session.id,
      continuePrompt: "",
      pendingFollowUp: null
    });
    const conversationMessages = await refreshConversation(result.session.id);

    await syncResponseWindow(result.session, conversationMessages);
  } catch (error) {
    store.commit("patchAnalysisState", {
      error: commandErrorMessage(
        error,
        "Nao foi possivel executar a analise deste contexto."
      )
    });
  } finally {
    store.commit("patchAnalysisState", {
      loading: false
    });
  }
}

async function handleSaveSettings() {
  if (!store.state.settings.draft) {
    return;
  }

  const draftSnapshot = cloneSettings(store.state.settings.draft);

  store.commit("patchSettingsState", {
    saving: true,
    error: null
  });

  try {
    await saveSettings(draftSnapshot);

    const currentDraft = store.state.settings.draft;
    const draftChangedDuringSave =
      currentDraft &&
      JSON.stringify(currentDraft) !== JSON.stringify(draftSnapshot);
    store.commit("patchSettingsState", {
      saved: draftSnapshot,
      draft: draftChangedDuringSave
        ? currentDraft
        : cloneSettings(draftSnapshot)
    });

    if (!shortcutsSuppressedForRecording) {
      void applyGlobalShortcuts(draftSnapshot.shortcuts).catch(() => {});
    }
  } catch (error) {
    store.commit("patchSettingsState", {
      error: commandErrorMessage(
        error,
        "Nao foi possivel persistir as configuracoes atuais."
      )
    });
  } finally {
    store.commit("patchSettingsState", {
      saving: false
    });

    if (
      store.state.settings.saved &&
      store.state.settings.draft &&
      JSON.stringify(store.state.settings.saved) !==
        JSON.stringify(store.state.settings.draft)
    ) {
      scheduleAutoSaveSettings();
    }
  }
}

function hasDuplicatedShortcutAccelerators(settings: UserSettings): boolean {
  const accelerators = new Set<string>();

  for (const shortcut of settings.shortcuts) {
    const accelerator = canonicalShortcutAccelerator(shortcut.accelerator);

    if (!accelerator) {
      continue;
    }

    if (accelerators.has(accelerator)) {
      return true;
    }

    accelerators.add(accelerator);
  }

  return false;
}

function canonicalShortcutAccelerator(accelerator: string): string | null {
  const isMac =
    typeof navigator !== "undefined" &&
    /Mac|iPhone|iPad|iPod/i.test(navigator.platform);
  const modifierAliases: Record<string, string> = {
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
  const modifierOrder = ["Cmd", "Ctrl", "Shift", "Alt"];
  const modifiers = new Set<string>();
  let mainKey = "";

  for (const part of accelerator.split("+")) {
    const trimmed = part.trim();
    const alias = modifierAliases[trimmed.toLowerCase()];

    if (alias) {
      modifiers.add(alias);
    } else if (trimmed) {
      mainKey = trimmed.toUpperCase();
    }
  }

  if (!mainKey) {
    return null;
  }

  return [
    ...modifierOrder.filter((modifier) => modifiers.has(modifier)),
    mainKey
  ].join("+");
}

function scheduleAutoSaveSettings() {
  if (autoSaveSettingsTimer) {
    clearTimeout(autoSaveSettingsTimer);
    autoSaveSettingsTimer = null;
  }

  const draft = store.state.settings.draft;
  const saved = store.state.settings.saved;

  if (
    !draft ||
    !saved ||
    store.state.settings.loading ||
    store.state.settings.saving ||
    hasDuplicatedShortcutAccelerators(draft) ||
    JSON.stringify(draft) === JSON.stringify(saved)
  ) {
    return;
  }

  autoSaveSettingsTimer = setTimeout(() => {
    autoSaveSettingsTimer = null;
    void handleSaveSettings();
  }, 600);
}

function activeShortcutBindings() {
  return (
    store.state.settings.saved?.shortcuts ??
    store.state.settings.draft?.shortcuts ??
    []
  );
}

function handleShortcutRecordingChange(recording: boolean) {
  shortcutsSuppressedForRecording = recording;

  void applyGlobalShortcuts(recording ? [] : activeShortcutBindings()).catch(
    () => {}
  );
}

function handleQuit() {
  void requestQuit().catch(() => {});
}

async function handleCapture() {
  if (store.state.analysis.captureProcessing) {
    return;
  }

  store.commit("patchAnalysisState", {
    error: null,
    captureProcessing: true
  });

  try {
    await openSelectionWindow();
  } catch (error) {
    store.commit("patchAnalysisState", {
      captureProcessing: false,
      error: commandErrorMessage(
        error,
        "Nao foi possivel abrir a area de selecao."
      )
    });
  }
}

async function handleSelectionResult(rect: CaptureRegionRequest) {
  try {
    const result = await captureScreenRegion(rect);
    store.commit("patchAnalysisState", {
      pendingCapture: {
        image_path: result.image_path,
        width: result.width,
        height: result.height,
        data_url: result.data_url
      }
    });

    await openPreviewWindow();
    await emitToPreviewWindow({
      image_data_url: result.data_url,
      image_width: result.width,
      image_height: result.height,
      has_capture: true
    });
  } catch (error) {
    store.commit("patchAnalysisState", {
      error: commandErrorMessage(
        error,
        "Nao foi possivel capturar a area selecionada."
      ),
      captureProcessing: false
    });
  }
}

async function confirmPendingCapture(notes: string) {
  const capture = store.state.analysis.pendingCapture;
  if (!capture) {
    store.commit("patchAnalysisState", {
      captureProcessing: false
    });
    return;
  }

  store.commit("patchAnalysisState", {
    loading: true,
    error: null,
    userNotes: notes
  });

  try {
    await emitPreviewAnalysisStatus({
      status: "ocr",
      message: "Extraindo texto da imagem..."
    });
    const text = await runOcrOnImage(
      capture.image_path,
      store.state.settings.draft?.ocr_language ?? "por"
    );
    store.commit("patchAnalysisState", {
      ocrText: text
    });

    await emitPreviewAnalysisStatus({
      status: "analyzing",
      message: "Analisando com o provider..."
    });
    store.commit("patchAnalysisState", {
      pendingCapture: null
    });
    await handleAnalyze();

    await emitPreviewAnalysisStatus({ status: "done", message: "" });
  } catch (error) {
    const message = commandErrorMessage(
      error,
      "Nao foi possivel concluir a analise."
    );
    store.commit("patchAnalysisState", {
      error: message
    });
    await emitPreviewAnalysisStatus({ status: "error", message });
  } finally {
    store.commit("patchAnalysisState", {
      loading: false,
      captureProcessing: false
    });
  }
}

function discardPendingCapture() {
  store.commit("patchAnalysisState", {
    pendingCapture: null,
    captureProcessing: false
  });
}
</script>

<template>
  <div
    class="apollo-main-shell flex h-screen flex-col overflow-hidden bg-apollo-app-shell text-slate-50"
  >
    <header
      class="flex h-14 shrink-0 items-center border-b border-apollo-app-border bg-apollo-app-sidebar px-5"
    >
      <button
        v-if="activeSurface === 'settings'"
        class="inline-flex h-9 w-9 items-center justify-center rounded-lg text-apollo-app-muted transition hover:bg-apollo-app-hover hover:text-white"
        data-testid="settings-back-button"
        type="button"
        aria-label="Voltar para home"
        @click="activateSurface('home')"
      >
        <svg
          class="h-5 w-5"
          aria-hidden="true"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
        >
          <path d="M15 18l-6-6 6-6" />
        </svg>
      </button>

      <template v-else>
        <div class="min-w-0">
          <p class="text-sm font-semibold text-white">Apollo</p>
          <p class="mt-1 text-xs text-apollo-app-muted">{{ versionText }}</p>
        </div>

        <nav
          class="ml-auto flex items-center gap-2 rounded-xl border border-apollo-app-border bg-apollo-app-shell p-1"
          data-testid="main-navigation"
          aria-label="Navegacao principal"
        >
          <button
            v-for="section in sections"
            :key="section.id"
            class="flex items-center gap-2 rounded-lg px-4 py-2 text-sm transition"
            :class="
              activeSurface === section.id
                ? 'bg-apollo-app-selected font-medium text-white'
                : 'text-apollo-app-muted hover:bg-apollo-app-hover hover:text-white'
            "
            type="button"
            @click="activateSurface(section.id)"
          >
            <component :is="section.icon" class="h-5 w-5 shrink-0" />
            {{ section.label }}
          </button>
        </nav>

        <button
          class="inline-flex h-9 items-center gap-2 rounded-lg border border-apollo-app-border bg-apollo-app-shell px-3 text-sm font-medium text-apollo-app-muted transition hover:border-apollo-app-selectedBorder hover:text-white ml-3"
          data-testid="quit-button"
          type="button"
          aria-label="Sair do Apollo"
          @click="handleQuit"
        >
          <svg
            class="h-4 w-4"
            aria-hidden="true"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
          >
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
            <path d="M16 17l5-5-5-5" />
            <path d="M21 12H9" />
          </svg>
          <span>Sair</span>
        </button>
      </template>
    </header>

    <main class="min-w-0 flex-1 overflow-hidden bg-apollo-app-panel">
      <Transition name="surface-float" mode="out-in">
        <div
          v-if="activeSurface === 'home'"
          key="home"
          class="h-full overflow-y-auto"
        >
          <div class="mx-auto max-w-6xl px-8 py-8">
            <h1 class="text-2xl font-semibold text-white">
              {{ sectionSummary.title }}
            </h1>
            <p class="mt-3 max-w-2xl text-sm leading-6 text-apollo-app-muted">
              {{ sectionSummary.description }}
            </p>

            <div class="mt-8">
              <HomeSurface @capture="handleCapture" />
            </div>
          </div>
        </div>

        <div
          v-else-if="activeSurface === 'history'"
          key="history"
          class="h-full overflow-y-auto"
        >
          <div class="mx-auto max-w-6xl px-8 py-8">
            <h1 class="text-2xl font-semibold text-white">
              {{ sectionSummary.title }}
            </h1>
            <p class="mt-3 max-w-2xl text-sm leading-6 text-apollo-app-muted">
              {{ sectionSummary.description }}
            </p>

            <div class="mt-8">
              <HistorySurface
                @clear-history="handleClearHistory"
                @delete-session="handleDeleteHistorySession"
                @open-session-chat="openHistorySessionChat"
              />
            </div>
          </div>
        </div>

        <div v-else key="settings" class="h-full overflow-hidden">
          <SettingsSurface
            @shortcut-recording-change="handleShortcutRecordingChange"
          />
        </div>
      </Transition>
    </main>
  </div>
</template>
