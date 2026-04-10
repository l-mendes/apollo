import { InjectionKey } from "vue";
import { createStore, useStore as baseUseStore, type Store } from "vuex";

import {
  createEmptyProviderCatalog,
  type AnalyzeCaptureResponse,
  type ConversationMessage,
  type InteractionSession,
  type ProviderCatalog,
  type UserSettings
} from "@/composables/useApolloDesktop";
import type {
  BootstrapSnapshot,
  HealthStatus
} from "@/composables/useDesktopCapabilities";
import type { AppSurface } from "@/composables/useWindowShell";

export interface PendingCapture {
  image_path: string;
  width: number;
  height: number;
  data_url: string;
}

export interface ApolloShellState {
  activeSurface: AppSurface;
  health: HealthStatus | null;
  snapshot: BootstrapSnapshot | null;
}

export interface ApolloSettingsState {
  saved: UserSettings | null;
  draft: UserSettings | null;
  providerCatalog: ProviderCatalog;
  loading: boolean;
  saving: boolean;
  error: string | null;
  providerCatalogLoading: boolean;
  providerCatalogError: string | null;
}

export interface ApolloHistoryState {
  items: InteractionSession[];
  selectedHistoryId: string | null;
  loading: boolean;
  error: string | null;
  conversationMessages: ConversationMessage[];
  conversationLoading: boolean;
  conversationError: string | null;
  continuePrompt: string;
  continueLoading: boolean;
  continueError: string | null;
}

export interface ApolloAnalysisState {
  ocrText: string;
  userNotes: string;
  loading: boolean;
  error: string | null;
  lastResponse: AnalyzeCaptureResponse["response"] | null;
  lastPrompt: string;
  pendingCapture: PendingCapture | null;
  captureProcessing: boolean;
}

export interface ApolloStoreState {
  shell: ApolloShellState;
  settings: ApolloSettingsState;
  history: ApolloHistoryState;
  analysis: ApolloAnalysisState;
}

function createInitialState(): ApolloStoreState {
  return {
    shell: {
      activeSurface: "home",
      health: null,
      snapshot: null
    },
    settings: {
      saved: null,
      draft: null,
      providerCatalog: createEmptyProviderCatalog(),
      loading: true,
      saving: false,
      error: null,
      providerCatalogLoading: true,
      providerCatalogError: null
    },
    history: {
      items: [],
      selectedHistoryId: null,
      loading: true,
      error: null,
      conversationMessages: [],
      conversationLoading: false,
      conversationError: null,
      continuePrompt: "",
      continueLoading: false,
      continueError: null
    },
    analysis: {
      ocrText: "",
      userNotes: "",
      loading: false,
      error: null,
      lastResponse: null,
      lastPrompt: "",
      pendingCapture: null,
      captureProcessing: false
    }
  };
}

function hasUnsavedSettings(state: ApolloStoreState): boolean {
  if (!state.settings.saved || !state.settings.draft) {
    return false;
  }

  return JSON.stringify(state.settings.saved) !== JSON.stringify(state.settings.draft);
}

function selectedSession(state: ApolloStoreState): InteractionSession | null {
  return (
    state.history.items.find(
      (session) => session.id === state.history.selectedHistoryId
    ) ??
    state.history.items[0] ??
    null
  );
}

export function createApolloStore() {
  return createStore<ApolloStoreState>({
    state: createInitialState,
    getters: {
      versionText(state: ApolloStoreState): string {
        return (
          state.shell.health?.version ??
          state.shell.snapshot?.metadata.version ??
          "v0.1.0"
        );
      },
      hasUnsavedSettings(state: ApolloStoreState): boolean {
        return hasUnsavedSettings(state);
      },
      homeErrorText(state: ApolloStoreState): string | null {
        if (!state.settings.saved) {
          return state.settings.error;
        }

        return state.settings.providerCatalogError;
      },
      historyPanelErrorText(state: ApolloStoreState): string | null {
        return state.history.error;
      },
      settingsPanelErrorText(state: ApolloStoreState): string | null {
        return state.settings.error;
      },
      selectedSession(state: ApolloStoreState): InteractionSession | null {
        return selectedSession(state);
      },
      appStatusText(state: ApolloStoreState): string {
        if (state.analysis.loading) {
          return "Analise em andamento";
        }

        if (state.settings.saving) {
          return "Salvando configuracoes";
        }

        if (state.shell.activeSurface === "history") {
          return state.history.loading
            ? "Carregando historico"
            : `${state.history.items.length} sessao(oes) carregadas`;
        }

        if (state.shell.activeSurface === "settings") {
          return hasUnsavedSettings(state)
            ? "Alteracoes locais presentes"
            : "Preferencias sincronizadas";
        }

        if (state.analysis.lastResponse) {
          return `Resposta pronta em ${state.analysis.lastResponse.model_key}`;
        }

        return "Janela principal pronta";
      }
    },
    mutations: {
      resetState(state: ApolloStoreState) {
        Object.assign(state, createInitialState());
      },
      setActiveSurface(state: ApolloStoreState, surface: AppSurface) {
        state.shell.activeSurface = surface;
      },
      patchShellState(
        state: ApolloStoreState,
        payload: Partial<ApolloShellState>
      ) {
        state.shell = { ...state.shell, ...payload };
      },
      patchSettingsState(
        state: ApolloStoreState,
        payload: Partial<ApolloSettingsState>
      ) {
        state.settings = { ...state.settings, ...payload };
      },
      patchHistoryState(
        state: ApolloStoreState,
        payload: Partial<ApolloHistoryState>
      ) {
        state.history = { ...state.history, ...payload };
      },
      patchAnalysisState(
        state: ApolloStoreState,
        payload: Partial<ApolloAnalysisState>
      ) {
        state.analysis = { ...state.analysis, ...payload };
      }
    }
  });
}

export type ApolloStore = Store<ApolloStoreState>;

export const apolloStoreKey: InjectionKey<ApolloStore> = Symbol("apollo-store");

export const apolloStore = createApolloStore();

export function useApolloStore() {
  return baseUseStore(apolloStoreKey);
}
