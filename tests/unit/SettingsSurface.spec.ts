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

describe("SettingsSurface", () => {
  it("renders the loading state while preferences are fetched", () => {
    const { wrapper } = mountSettingsSurface();

    expect(wrapper.find('[data-testid="settings-loading"]').exists()).toBe(
      true
    );
  });

  it("updates the settings draft in the store and emits save from the productive form", async () => {
    const { store, wrapper } = mountSettingsSurface();

    store.commit("patchSettingsState", readySettings());
    await nextTick();

    await wrapper.find("textarea").setValue("Updated prompt");
    await wrapper
      .findAll("button")
      .find((button) => button.text().includes("High"))
      ?.trigger("click");
    await wrapper.find("button").trigger("click");

    expect(wrapper.find('[data-testid="settings-ready"]').exists()).toBe(true);
    expect(store.state.settings.draft?.base_prompt).toBe("Updated prompt");
    expect(store.state.settings.draft?.reasoning_effort).toBe("high");
    expect(wrapper.emitted("save")).toHaveLength(1);
  });

  it("captures shortcut accelerators from pressed keys", async () => {
    const { store, wrapper } = mountSettingsSurface();

    store.commit("patchSettingsState", readySettings());
    await nextTick();

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
    ).toContain("Capturar tela");
  });

  it("blocks saving when persisted shortcut rows are duplicated", async () => {
    const { store, wrapper } = mountSettingsSurface();

    store.commit(
      "patchSettingsState",
      readySettings([
        shortcut("capture_screen", "CmdOrCtrl+Shift+A"),
        shortcut("open_settings", "Ctrl+Shift+A")
      ])
    );
    await nextTick();

    expect(
      wrapper.find('[data-testid="shortcut-conflict-summary"]').exists()
    ).toBe(true);
    expect(wrapper.find("button").attributes("disabled")).toBeDefined();
  });
});
