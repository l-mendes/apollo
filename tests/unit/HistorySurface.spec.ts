import { mount } from "@vue/test-utils";
import { nextTick } from "vue";

import HistorySurface from "@/components/surfaces/HistorySurface.vue";
import { apolloStoreKey, createApolloStore } from "@/store/apollo";

function mountHistorySurface() {
  const store = createApolloStore();
  const wrapper = mount(HistorySurface, {
    global: {
      plugins: [[store, apolloStoreKey]]
    }
  });

  return { store, wrapper };
}

function session(id: string, responseText = "Answer") {
  return {
    id,
    provider_kind: "OpenAi",
    model_key: "gpt-4.1-mini",
    source_kind: "ScreenCapture",
    ocr_text: "A sentence",
    user_notes: "Notes",
    request_prompt: "Prompt",
    response_text: responseText
  };
}

describe("HistorySurface", () => {
  it("renders an empty state when there are no sessions yet", async () => {
    const { store, wrapper } = mountHistorySurface();

    store.commit("patchHistoryState", {
      loading: false,
      error: null,
      items: []
    });
    await nextTick();

    expect(wrapper.find('[data-testid="history-empty"]').exists()).toBe(true);
    expect(wrapper.text()).toContain("Nenhuma sessao foi registrada ainda");
  });

  it("renders only the persisted session list and updates selection", async () => {
    const { store, wrapper } = mountHistorySurface();

    store.commit("patchHistoryState", {
      loading: false,
      error: null,
      items: [
        session("session-1", "Answer 1"),
        session("session-2", "Answer 2")
      ],
      selectedHistoryId: "session-1"
    });
    await nextTick();

    await wrapper
      .findAll('[data-testid="history-session"]')[1]
      .trigger("click");

    expect(wrapper.find('[data-testid="history-ready"]').exists()).toBe(true);
    expect(wrapper.findAll('[data-testid="history-session"]')).toHaveLength(2);
    expect(wrapper.text()).toContain("Sessoes salvas");
    expect(wrapper.text()).toContain("Answer 2");
    expect(wrapper.text()).not.toContain("Prompt registrado");
    expect(wrapper.text()).not.toContain("Conversa continua");
    expect(store.state.history.selectedHistoryId).toBe("session-2");
  });

  it("emits clear history from the toolbar action", async () => {
    const { store, wrapper } = mountHistorySurface();

    store.commit("patchHistoryState", {
      loading: false,
      error: null,
      items: [session("session-1")],
      selectedHistoryId: "session-1"
    });
    await nextTick();

    await wrapper.find('[data-testid="clear-history-button"]').trigger("click");

    expect(wrapper.emitted("clear-history")).toHaveLength(1);
  });

  it("emits delete for a single session without opening the chat", async () => {
    const { store, wrapper } = mountHistorySurface();

    store.commit("patchHistoryState", {
      loading: false,
      error: null,
      items: [session("session-1")],
      selectedHistoryId: "session-1"
    });
    await nextTick();

    await wrapper
      .find('[data-testid="delete-session-button"]')
      .trigger("click");

    expect(wrapper.find('[data-testid="delete-session-button"]').text()).toBe(
      ""
    );
    expect(
      wrapper
        .find('[data-testid="delete-session-button"]')
        .attributes("aria-label")
    ).toBe("Excluir sessao session-1");
    expect(wrapper.emitted("delete-session")).toEqual([["session-1"]]);
    expect(wrapper.emitted("open-session-chat")).toBeUndefined();
  });

  it("opens the response chat window flow on double click", async () => {
    const { store, wrapper } = mountHistorySurface();

    store.commit("patchHistoryState", {
      loading: false,
      error: null,
      items: [session("session-1")],
      selectedHistoryId: null
    });
    await nextTick();

    await wrapper.find('[data-testid="history-session"]').trigger("dblclick");

    expect(store.state.history.selectedHistoryId).toBe("session-1");
    expect(wrapper.emitted("open-session-chat")).toEqual([["session-1"]]);
  });
});
