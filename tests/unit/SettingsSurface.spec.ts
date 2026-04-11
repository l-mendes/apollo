import { mount } from "@vue/test-utils";
import { nextTick } from "vue";

import SettingsSurface from "@/components/surfaces/SettingsSurface.vue";
import {
  createEmptyProviderCatalog,
  type ShortcutBinding
} from "@/composables/useApolloDesktop";
import { apolloStoreKey, createApolloStore } from "@/store/apollo";

function mountSettingsSurface() {
  const store = createApolloStore();
  const wrapper = mount(SettingsSurface, {
    global: {
      plugins: [[store, apolloStoreKey]]
    }
  });

  return { store, wrapper };
}

function shortcut(
  action: string,
  accelerator: string,
  enabled = true
): ShortcutBinding {
  return { action, accelerator, enabled };
}

function readySettings(
  shortcuts: ShortcutBinding[] = [
    shortcut("capture_screen", "CmdOrCtrl+Shift+A")
  ]
) {
  const catalog = createEmptyProviderCatalog();
  catalog.OpenAi = [
    {
      provider_kind: "OpenAi",
      channel: "Http",
      model_key: "gpt-4.1-mini",
      display_name: "GPT-4.1 Mini",
      manually_managed: true,
      is_default: true
    }
  ];

  return {
    loading: false,
    providerCatalogLoading: false,
    error: null,
    providerCatalogError: null,
    providerCatalog: catalog,
    saved: {
      preferred_provider: "OpenAi",
      preferred_model: "gpt-4.1-mini",
      reasoning_effort: "medium",
      base_prompt: "Explain meaning and usage.",
      ocr_language: "por",
      output_language: "Português",
      shortcuts: shortcuts.map((item) => ({ ...item }))
    },
    draft: {
      preferred_provider: "OpenAi",
      preferred_model: "gpt-4.1-mini",
      reasoning_effort: "medium",
      base_prompt: "Explain meaning and usage.",
      ocr_language: "por",
      output_language: "Português",
      shortcuts: shortcuts.map((item) => ({ ...item }))
    }
  };
}

async function openSettingsContext(
  wrapper: ReturnType<typeof mount>,
  label: string
) {
  await wrapper
    .findAll("button")
    .find((button) => button.text().includes(label))
    ?.trigger("click");
  await nextTick();
}

describe("SettingsSurface", () => {
  it("renders the loading state while preferences are fetched", () => {
    const { wrapper } = mountSettingsSurface();

    expect(wrapper.find('[data-testid="settings-loading"]').exists()).toBe(
      true
    );
  });

  it("updates the settings draft in the store from the productive form", async () => {
    const { store, wrapper } = mountSettingsSurface();

    store.commit("patchSettingsState", readySettings());
    await nextTick();

    await wrapper.find("textarea").setValue("Updated prompt");
    await wrapper
      .findAll("button")
      .find((button) => button.text().includes("High"))
      ?.trigger("click");

    expect(wrapper.find('[data-testid="settings-ready"]').exists()).toBe(true);
    expect(store.state.settings.draft?.base_prompt).toBe("Updated prompt");
    expect(store.state.settings.draft?.reasoning_effort).toBe("high");
  });

  it("captures shortcut accelerators from pressed keys", async () => {
    const { store, wrapper } = mountSettingsSurface();

    store.commit("patchSettingsState", readySettings());
    await nextTick();
    await openSettingsContext(wrapper, "Atalhos");

    const shortcutRecorder = wrapper.find('[data-testid="shortcut-recorder"]');

    await shortcutRecorder.trigger("click");
    await shortcutRecorder.trigger("keydown", {
      key: "Shift",
      shiftKey: true
    });
    expect(store.state.settings.draft?.shortcuts[0].accelerator).toBe(
      "CmdOrCtrl+Shift+A"
    );

    await shortcutRecorder.trigger("keydown", {
      key: "k",
      ctrlKey: true,
      altKey: true
    });

    expect(store.state.settings.draft?.shortcuts[0].accelerator).toBe(
      "Ctrl+Alt+K"
    );
    expect(shortcutRecorder.text()).toContain("Ctrl");
    expect(shortcutRecorder.text()).toContain("Alt");
    expect(shortcutRecorder.text()).toContain("K");
    expect(wrapper.emitted("shortcut-recording-change")).toEqual([
      [true],
      [false]
    ]);
  });

  it("accepts the shortcut combination already assigned to the current row", async () => {
    const { store, wrapper } = mountSettingsSurface();

    store.commit(
      "patchSettingsState",
      readySettings([
        shortcut("capture_screen", "CmdOrCtrl+Shift+A"),
        shortcut("open_history", "CmdOrCtrl+H")
      ])
    );
    await nextTick();
    await openSettingsContext(wrapper, "Atalhos");

    const shortcutRecorders = wrapper.findAll(
      '[data-testid="shortcut-recorder"]'
    );
    await shortcutRecorders[1].trigger("click");
    await shortcutRecorders[1].trigger("keydown", {
      key: "h",
      ctrlKey: true
    });

    expect(store.state.settings.draft?.shortcuts[1].accelerator).toBe("Ctrl+H");
    expect(wrapper.find('[data-testid="shortcut-validation-1"]').exists()).toBe(
      false
    );
  });

  it("cancels shortcut recording with escape", async () => {
    const { store, wrapper } = mountSettingsSurface();

    store.commit("patchSettingsState", readySettings());
    await nextTick();
    await openSettingsContext(wrapper, "Atalhos");

    const shortcutRecorder = wrapper.find('[data-testid="shortcut-recorder"]');

    await shortcutRecorder.trigger("click");
    expect(shortcutRecorder.text()).toContain("Gravando");

    await shortcutRecorder.trigger("keydown", { key: "Escape" });
    await shortcutRecorder.trigger("keydown", {
      key: "k",
      ctrlKey: true
    });

    expect(shortcutRecorder.text()).not.toContain("Gravando");
    expect(store.state.settings.draft?.shortcuts[0].accelerator).toBe(
      "CmdOrCtrl+Shift+A"
    );
    expect(wrapper.emitted("shortcut-recording-change")).toEqual([
      [true],
      [false]
    ]);
  });

  it("rejects shortcut combinations already assigned to another shortcut", async () => {
    const { store, wrapper } = mountSettingsSurface();

    store.commit(
      "patchSettingsState",
      readySettings([
        shortcut("capture_screen", "CmdOrCtrl+Shift+A"),
        shortcut("open_history", "CmdOrCtrl+H")
      ])
    );
    await nextTick();
    await openSettingsContext(wrapper, "Atalhos");

    const shortcutRecorders = wrapper.findAll(
      '[data-testid="shortcut-recorder"]'
    );
    await shortcutRecorders[1].trigger("click");
    await shortcutRecorders[1].trigger("keydown", {
      key: "a",
      ctrlKey: true,
      shiftKey: true
    });

    expect(store.state.settings.draft?.shortcuts[1].accelerator).toBe(
      "CmdOrCtrl+H"
    );
    expect(
      wrapper.find('[data-testid="shortcut-validation-1"]').text()
    ).toContain("Capturar Tela");
  });

  it("shows shortcut conflicts without exposing a manual save button", async () => {
    const { store, wrapper } = mountSettingsSurface();

    store.commit(
      "patchSettingsState",
      readySettings([
        shortcut("capture_screen", "CmdOrCtrl+Shift+A"),
        shortcut("open_settings", "Ctrl+Shift+A")
      ])
    );
    await nextTick();
    await openSettingsContext(wrapper, "Atalhos");

    expect(
      wrapper.find('[data-testid="shortcut-conflict-summary"]').exists()
    ).toBe(true);
    expect(wrapper.text()).not.toContain("Salvar configuracoes");
  });

  it("groups settings by context and renders friendly shortcut labels", async () => {
    const { store, wrapper } = mountSettingsSurface();

    store.commit(
      "patchSettingsState",
      readySettings([
        shortcut("capture_screen", "CmdOrCtrl+Shift+A"),
        shortcut("open_settings", "CmdOrCtrl+,"),
        shortcut("open_history", "CmdOrCtrl+H")
      ])
    );
    await nextTick();

    expect(
      wrapper.find('[data-testid="settings-context-providers"]').exists()
    ).toBe(true);
    expect(wrapper.find('[data-testid="settings-ready"]').classes()).toContain(
      "overflow-hidden"
    );
    expect(
      wrapper.find('[aria-label="Contextos de configuracao"]').classes()
    ).toContain("border-r");
    expect(
      wrapper.find('[data-testid="settings-context-providers"]').classes()
    ).toContain("overflow-y-auto");
    expect(wrapper.text()).toContain("Provedor");
    expect(wrapper.text()).not.toContain("Idioma da Aplicacao");

    await openSettingsContext(wrapper, "Idioma");
    expect(
      wrapper.find('[data-testid="settings-context-language"]').exists()
    ).toBe(true);
    expect(wrapper.text()).toContain("Idioma da Aplicacao");

    await openSettingsContext(wrapper, "Atalhos");
    expect(
      wrapper.find('[data-testid="settings-context-shortcuts"]').exists()
    ).toBe(true);
    expect(wrapper.find('[data-testid="shortcut-label-0"]').text()).toBe(
      "Capturar Tela"
    );
    expect(wrapper.find('[data-testid="shortcut-label-1"]').text()).toBe(
      "Abrir Configuracoes"
    );
    expect(wrapper.find('[data-testid="shortcut-label-2"]').text()).toBe(
      "Abrir Historico"
    );
  });
});
