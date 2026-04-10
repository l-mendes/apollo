import { mount } from "@vue/test-utils";
import { vi } from "vitest";

import FloatingTrayBar from "@/components/tray/FloatingTrayBar.vue";

const startDraggingMock = vi.fn().mockResolvedValue(undefined);

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    startDragging: startDraggingMock
  })
}));

describe("FloatingTrayBar", () => {
  it("renders the required actions with tooltips", () => {
    const wrapper = mount(FloatingTrayBar, {
      props: {
        activeSurface: "none",
        versionText: "v0.1.0"
      }
    });

    expect(wrapper.text()).toContain("v0.1.0");
    expect(wrapper.find('[aria-label="Open home"]').exists()).toBe(true);
    expect(wrapper.find('[aria-label="Drag tray"]').exists()).toBe(true);
    expect(wrapper.find('[aria-label="Open history"]').exists()).toBe(true);
    expect(wrapper.find('[aria-label="Open settings"]').exists()).toBe(true);
    expect(wrapper.find('[aria-label="Quit app"]').exists()).toBe(true);
  });

  it("emits semantic events when action buttons are clicked", async () => {
    const wrapper = mount(FloatingTrayBar, {
      props: {
        activeSurface: "none",
        versionText: "v0.1.0"
      }
    });

    await wrapper.find('[aria-label="Open home"]').trigger("click");
    await wrapper.find('[aria-label="Open history"]').trigger("click");
    await wrapper.find('[aria-label="Open settings"]').trigger("click");
    await wrapper.find('[aria-label="Quit app"]').trigger("click");

    expect(wrapper.emitted("home")).toHaveLength(1);
    expect(wrapper.emitted("history")).toHaveLength(1);
    expect(wrapper.emitted("settings")).toHaveLength(1);
    expect(wrapper.emitted("quit")).toHaveLength(1);
  });

  it("starts dragging when the drag handle is pressed", async () => {
    const wrapper = mount(FloatingTrayBar, {
      props: {
        activeSurface: "none",
        versionText: "v0.1.0"
      }
    });

    await wrapper.find('[aria-label="Drag tray"]').trigger("mousedown", {
      button: 0
    });

    expect(startDraggingMock).toHaveBeenCalledTimes(1);
  });
});
