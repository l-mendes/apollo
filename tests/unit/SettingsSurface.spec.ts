import { mount } from "@vue/test-utils";
import { nextTick } from "vue";

import SettingsSurface from "@/components/surfaces/SettingsSurface.vue";
import { createEmptyProviderCatalog } from "@/composables/useApolloDesktop";
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

describe("SettingsSurface", () => {
  it("renders the loading state while preferences are fetched", () => {
    const { wrapper } = mountSettingsSurface();

    expect(wrapper.find('[data-testid="settings-loading"]').exists()).toBe(
      true
    );
  });

  it("updates the settings draft in the store and emits save from the productive form", async () => {
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

    const { store, wrapper } = mountSettingsSurface();

    store.commit("patchSettingsState", {
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
        shortcuts: [
          {
            action: "capture_screen",
            accelerator: "CmdOrCtrl+Shift+A",
            enabled: true
          }
        ]
      },
      draft: {
        preferred_provider: "OpenAi",
        preferred_model: "gpt-4.1-mini",
        reasoning_effort: "medium",
        base_prompt: "Explain meaning and usage.",
        ocr_language: "por",
        output_language: "Português",
        shortcuts: [
          {
            action: "capture_screen",
            accelerator: "CmdOrCtrl+Shift+A",
            enabled: true
          }
        ]
      }
    });
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
});
