import { mount } from "@vue/test-utils";

import SettingsSurface from "@/components/surfaces/SettingsSurface.vue";
import { createEmptyProviderCatalog } from "@/composables/useApolloDesktop";

describe("SettingsSurface", () => {
  it("renders the loading state while preferences are fetched", () => {
    const wrapper = mount(SettingsSurface, {
      props: {
        loading: true,
        saving: false,
        errorText: null,
        settings: null,
        modelsByProvider: createEmptyProviderCatalog(),
        hasUnsavedChanges: false
      }
    });

    expect(wrapper.find('[data-testid="settings-loading"]').exists()).toBe(
      true
    );
  });

  it("emits updated settings and save events from the productive form", async () => {
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

    const wrapper = mount(SettingsSurface, {
      props: {
        loading: false,
        saving: false,
        errorText: null,
        settings: {
          preferred_provider: "OpenAi",
          preferred_model: "gpt-4.1-mini",
          base_prompt: "Explain meaning and usage.",
          shortcuts: [
            {
              action: "capture_screen",
              accelerator: "CmdOrCtrl+Shift+A",
              enabled: true
            }
          ]
        },
        modelsByProvider: catalog,
        hasUnsavedChanges: true
      }
    });

    await wrapper.find("textarea").setValue("Updated prompt");
    await wrapper.find("button").trigger("click");

    expect(wrapper.find('[data-testid="settings-ready"]').exists()).toBe(true);
    expect(wrapper.emitted("update:settings")).toHaveLength(1);
    expect(wrapper.emitted("save")).toHaveLength(1);
  });
});
