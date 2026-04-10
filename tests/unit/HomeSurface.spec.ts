import { mount } from "@vue/test-utils";

import HomeSurface from "@/components/surfaces/HomeSurface.vue";
import type { UserSettings } from "@/composables/useApolloDesktop";

const settings: UserSettings = {
  preferred_provider: "OpenAi",
  preferred_model: "gpt-4.1-mini",
  base_prompt: "Explain meaning, grammar and usage.",
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

describe("HomeSurface", () => {
  it("renders a loading state while the workspace is bootstrapping", () => {
    const wrapper = mount(HomeSurface, {
      props: {
        loading: true,
        errorText: null,
        settings: null,
        isAnalyzing: false,
        analysisErrorText: null
      }
    });

    expect(wrapper.find('[data-testid="home-loading"]').exists()).toBe(true);
  });

  it("renders the clean ready state with provider, model and shortcut summaries", () => {
    const wrapper = mount(HomeSurface, {
      props: {
        loading: false,
        errorText: null,
        settings,
        isAnalyzing: false,
        analysisErrorText: null
      }
    });

    expect(wrapper.find('[data-testid="home-ready"]').exists()).toBe(true);
    expect(wrapper.text()).toContain("Provider ativo");
    expect(wrapper.text()).toContain("Modelo selecionado");

    const shortcutsBlock = wrapper.find('[data-testid="home-shortcuts"]');
    expect(shortcutsBlock.exists()).toBe(true);
    expect(shortcutsBlock.text()).toContain("Capturar tela");
    // Only enabled shortcuts are listed.
    expect(shortcutsBlock.text()).not.toContain("Abrir historico");
    // The accelerator key combo is rendered alongside the action label.
    expect(shortcutsBlock.text()).toMatch(/Shift \+ A/);
  });

  it("emits a capture intent from the main action", async () => {
    const wrapper = mount(HomeSurface, {
      props: {
        loading: false,
        errorText: null,
        settings,
        isAnalyzing: false,
        analysisErrorText: null
      }
    });

    await wrapper.find('[data-testid="capture-button"]').trigger("click");

    expect(wrapper.emitted("capture")).toHaveLength(1);
  });
});
