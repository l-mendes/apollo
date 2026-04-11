import { mount } from "@vue/test-utils";
import { nextTick } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { apolloStoreKey, createApolloStore } from "@/store/apollo";

const mocks = vi.hoisted(() => ({
  applyGlobalShortcutsMock: vi.fn(),
  loadSettingsMock: vi.fn(),
  listProviderModelsForMock: vi.fn(),
  listHistoryMock: vi.fn(),
  fetchBootstrapSnapshotMock: vi.fn(),
  fetchHealthStatusMock: vi.fn(),
  revealCurrentWindowMock: vi.fn(),
  shortcutActionHandler: null as null | ((action: string) => void)
}));

const savedShortcuts = [
  {
    action: "capture_screen",
    accelerator: "CmdOrCtrl+Shift+A",
    enabled: true
  }
];

vi.mock("@/composables/useApolloDesktop", () => ({
  PROVIDER_OPTIONS: [
    { kind: "OpenAi", label: "OpenAI", channel: "Http" },
    { kind: "Anthropic", label: "Anthropic", channel: "Http" },
    { kind: "OllamaCloud", label: "Ollama Cloud", channel: "Http" },
    { kind: "OllamaLocal", label: "Ollama Local", channel: "Http" },
    { kind: "ClaudeCli", label: "Claude CLI", channel: "Cli" },
    { kind: "CodexCli", label: "Codex CLI", channel: "Cli" },
    { kind: "CopilotCli", label: "Copilot CLI", channel: "Cli" }
  ],
  analyzeCapture: vi.fn(),
  applyGlobalShortcuts: mocks.applyGlobalShortcutsMock,
  captureScreenRegion: vi.fn(),
  clearHistory: vi.fn(),
  cloneSettings: (settings: unknown) => structuredClone(settings),
  commandErrorMessage: (error: unknown, fallback: string) =>
    error instanceof Error ? error.message : fallback,
  createEmptyProviderCatalog: () => ({
    OpenAi: [],
    Anthropic: [],
    OllamaCloud: [],
    OllamaLocal: [],
    ClaudeCli: [],
    CodexCli: [],
    CopilotCli: []
  }),
  deleteHistorySession: vi.fn(),
  listHistory: mocks.listHistoryMock,
  listProviderModelsFor: mocks.listProviderModelsForMock,
  loadConversationMessages: vi.fn(),
  loadSettings: mocks.loadSettingsMock,
  providerLabel: () => "OpenAI",
  runOcrOnImage: vi.fn(),
  saveSettings: vi.fn()
}));

vi.mock("@/composables/useDesktopCapabilities", () => ({
  fetchBootstrapSnapshot: mocks.fetchBootstrapSnapshotMock,
  fetchHealthStatus: mocks.fetchHealthStatusMock
}));

vi.mock("@/composables/useWindowShell", () => ({
  emitPreviewAnalysisStatus: vi.fn(),
  emitSurfaceChanged: vi.fn(),
  emitToPreviewWindow: vi.fn(),
  emitToResponseWindow: vi.fn(),
  hideResponseWindow: vi.fn(),
  listenForAppCloseToHide: vi.fn(async () => () => {}),
  listenForOcrResult: vi.fn(async () => () => {}),
  listenForPreviewCancel: vi.fn(async () => () => {}),
  listenForPreviewConfirm: vi.fn(async () => () => {}),
  listenForResponseConversationSync: vi.fn(async () => () => {}),
  listenForSelectionCancelled: vi.fn(async () => () => {}),
  listenForSelectionResult: vi.fn(async () => () => {}),
  listenForShortcutAction: vi.fn(async (handler) => {
    mocks.shortcutActionHandler = handler;
    return () => {};
  }),
  listenForStartAreaCapture: vi.fn(async () => () => {}),
  listenForSurfaceNavigation: vi.fn(async () => () => {}),
  openPreviewWindow: vi.fn(),
  openResponseWindow: vi.fn(),
  openSelectionWindow: vi.fn(),
  revealCurrentWindow: mocks.revealCurrentWindowMock,
  syncAppWindowAppearance: vi.fn()
}));

import App from "@/App.vue";

async function flushPromises() {
  for (let index = 0; index < 8; index += 1) {
    await Promise.resolve();
    await nextTick();
  }
}

describe("App", () => {
  beforeEach(() => {
    window.history.pushState({}, "", "/?surface=settings");
    mocks.applyGlobalShortcutsMock.mockReset().mockResolvedValue(undefined);
    mocks.revealCurrentWindowMock.mockReset().mockResolvedValue(undefined);
    mocks.shortcutActionHandler = null;
    mocks.loadSettingsMock.mockReset().mockResolvedValue({
      preferred_provider: "OpenAi",
      preferred_model: "gpt-4.1-mini",
      reasoning_effort: "medium",
      base_prompt: "Explain meaning and usage.",
      ocr_language: "por",
      output_language: "Português",
      shortcuts: savedShortcuts
    });
    mocks.listProviderModelsForMock.mockReset().mockResolvedValue([
      {
        provider_kind: "OpenAi",
        channel: "Http",
        model_key: "gpt-4.1-mini",
        display_name: "GPT-4.1 Mini",
        manually_managed: true,
        is_default: true
      }
    ]);
    mocks.listHistoryMock.mockReset().mockResolvedValue([]);
    mocks.fetchBootstrapSnapshotMock.mockReset().mockResolvedValue({
      metadata: { version: "v0.1.0" }
    });
    mocks.fetchHealthStatusMock.mockReset().mockResolvedValue({
      appName: "Apollo",
      status: "Ready",
      version: "v0.1.0"
    });
  });

  it("temporarily suppresses global shortcuts while recording a settings shortcut", async () => {
    const store = createApolloStore();
    const wrapper = mount(App, {
      global: {
        plugins: [[store, apolloStoreKey]]
      }
    });
    await flushPromises();

    const shortcutRecorder = wrapper.find('[data-testid="shortcut-recorder"]');
    await shortcutRecorder.trigger("click");
    await flushPromises();

    expect(mocks.applyGlobalShortcutsMock).toHaveBeenLastCalledWith([]);

    await shortcutRecorder.trigger("keydown", { key: "Escape" });
    await flushPromises();

    expect(mocks.applyGlobalShortcutsMock).toHaveBeenLastCalledWith(
      savedShortcuts
    );
  });

  it("reveals and focuses the app window when app navigation shortcuts fire", async () => {
    const store = createApolloStore();
    mount(App, {
      global: {
        plugins: [[store, apolloStoreKey]]
      }
    });
    await flushPromises();

    mocks.shortcutActionHandler?.("abrir_historico");
    await nextTick();

    expect(store.state.shell.activeSurface).toBe("history");
    expect(mocks.revealCurrentWindowMock).toHaveBeenCalled();
  });
});
