import { mount } from "@vue/test-utils";
import { nextTick } from "vue";

import HomeSurface from "@/components/surfaces/HomeSurface.vue";
import type { UserSettings } from "@/composables/useApolloDesktop";
import { apolloStoreKey, createApolloStore } from "@/store/apollo";

const settings: UserSettings = {
  preferred_provider: "OpenAi",
  preferred_model: "gpt-4.1-mini",
  reasoning_effort: "medium",
  base_prompt: "Explain meaning, grammar and usage.",
  ocr_language: "por",
  output_language: "Português",
  shortcuts: [
    {
      action: "capture_screen",
      accelerator: "CmdOrCtrl+Shift+A",
      enabled: true
    },
    {
      action: "open_settings",
      accelerator: "CmdOrCtrl+,",
      enabled: true
    },
    {
      action: "open_history",
      accelerator: "CmdOrCtrl+Shift+H",
      enabled: false
    }
  ]
};

function mountHomeSurface() {
  const store = createApolloStore();
  const wrapper = mount(HomeSurface, {
    global: {
      plugins: [[store, apolloStoreKey]]
    }
  });

  return { store, wrapper };
}

describe("HomeSurface", () => {
  it("renders a loading state while the workspace is bootstrapping", () => {
    const { wrapper } = mountHomeSurface();

    expect(wrapper.find('[data-testid="home-loading"]').exists()).toBe(true);
  });

  it("renders the clean ready state with provider, model and shortcut summaries", async () => {
    const { store, wrapper } = mountHomeSurface();

    store.commit("patchSettingsState", {
      loading: false,
      providerCatalogLoading: false,
      error: null,
      providerCatalogError: null,
      saved: settings,
      draft: settings
    });
    await nextTick();

    expect(wrapper.find('[data-testid="home-ready"]').exists()).toBe(true);
    expect(wrapper.text()).toContain("Provider ativo");
    expect(wrapper.text()).toContain("Modelo selecionado");

    const shortcutsBlock = wrapper.find('[data-testid="home-shortcuts"]');
    expect(shortcutsBlock.exists()).toBe(true);
    expect(shortcutsBlock.text()).toContain("Capturar tela");
    expect(shortcutsBlock.text()).not.toContain("Abrir historico");
    expect(shortcutsBlock.text()).toMatch(/Shift \+ A/);
  });

  it("emits a capture intent from the main action", async () => {
    const { store, wrapper } = mountHomeSurface();

    store.commit("patchSettingsState", {
      loading: false,
      providerCatalogLoading: false,
      saved: settings,
      draft: settings
    });
    await nextTick();

    await wrapper.find('[data-testid="capture-button"]').trigger("click");

    expect(wrapper.emitted("capture")).toHaveLength(1);
  });
});
