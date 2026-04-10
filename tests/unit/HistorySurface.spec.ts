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

  it("renders persisted sessions and updates the selected session in the store", async () => {
    const { store, wrapper } = mountHistorySurface();

    store.commit("patchHistoryState", {
      loading: false,
      error: null,
      items: [
        {
          id: "session-1",
          provider_kind: "OpenAi",
          model_key: "gpt-4.1-mini",
          source_kind: "ScreenCapture",
          ocr_text: "A sentence",
          user_notes: "Notes",
          request_prompt: "Prompt",
          response_text: "Answer"
        },
        {
          id: "session-2",
          provider_kind: "OpenAi",
          model_key: "gpt-4.1-mini",
          source_kind: "ManualText",
          ocr_text: "Another sentence",
          user_notes: "More notes",
          request_prompt: "Prompt 2",
          response_text: "Answer 2"
        }
      ],
      selectedHistoryId: "session-1"
    });
    await nextTick();

    await wrapper.findAll("aside button")[1].trigger("click");

    expect(wrapper.find('[data-testid="history-ready"]').exists()).toBe(true);
    expect(wrapper.text()).toContain("Sessao selecionada");
    expect(store.state.history.selectedHistoryId).toBe("session-2");
  });

  it("stores follow-up edits and emits continuation from the action button", async () => {
    const { store, wrapper } = mountHistorySurface();

    store.commit("patchHistoryState", {
      loading: false,
      error: null,
      items: [
        {
          id: "session-1",
          provider_kind: "OpenAi",
          model_key: "gpt-4.1-mini",
          source_kind: "ScreenCapture",
          ocr_text: "A sentence",
          user_notes: "Notes",
          request_prompt: "Prompt",
          response_text: "Answer"
        }
      ],
      selectedHistoryId: "session-1",
      conversationMessages: [
        {
          id: "message-1",
          session_id: "session-1",
          role: "User",
          content: "Original question"
        }
      ],
      continuePrompt: "Give me two more examples",
      continueLoading: false,
      continueError: null
    });
    await nextTick();

    await wrapper
      .find('[data-testid="chat-composer"]')
      .setValue("Another follow-up");
    await wrapper.find("form").trigger("submit");

    expect(store.state.history.continuePrompt).toBe("");
    expect(wrapper.emitted("continue-conversation")).toEqual([
      ["Another follow-up"]
    ]);
  });

  it("submits with Enter and keeps Shift+Enter available for line breaks", async () => {
    const { store, wrapper } = mountHistorySurface();

    store.commit("patchHistoryState", {
      loading: false,
      error: null,
      items: [
        {
          id: "session-1",
          provider_kind: "OpenAi",
          model_key: "gpt-4.1-mini",
          source_kind: "ScreenCapture",
          ocr_text: "A sentence",
          user_notes: "Notes",
          request_prompt: "Prompt",
          response_text: "Answer"
        }
      ],
      selectedHistoryId: "session-1",
      conversationMessages: [
        {
          id: "message-1",
          session_id: "session-1",
          role: "User",
          content: "Original question"
        }
      ],
      continuePrompt: "",
      continueLoading: false,
      continueError: null
    });
    await nextTick();

    const composer = wrapper.find('[data-testid="chat-composer"]');

    await composer.setValue("Line one");
    await composer.trigger("keydown", { key: "Enter", shiftKey: true });
    expect(wrapper.emitted("continue-conversation")).toBeUndefined();

    await composer.trigger("keydown", { key: "Enter" });
    expect(wrapper.emitted("continue-conversation")).toEqual([["Line one"]]);
  });

  it("shows the pending follow-up and thinking state while continuation is loading", async () => {
    const { store, wrapper } = mountHistorySurface();

    store.commit("patchHistoryState", {
      loading: false,
      error: null,
      items: [
        {
          id: "session-1",
          provider_kind: "OpenAi",
          model_key: "gpt-4.1-mini",
          source_kind: "ScreenCapture",
          ocr_text: "A sentence",
          user_notes: "Notes",
          request_prompt: "Prompt",
          response_text: "Answer"
        }
      ],
      selectedHistoryId: "session-1",
      conversationMessages: [
        {
          id: "message-1",
          session_id: "session-1",
          role: "User",
          content: "Original question"
        }
      ],
      pendingFollowUp: "Another follow-up",
      continuePrompt: "",
      continueLoading: true,
      continueError: null
    });
    await nextTick();

    const composer = wrapper.find<HTMLTextAreaElement>(
      '[data-testid="chat-composer"]'
    );

    expect(composer.element.value).toBe("");
    expect(composer.element.disabled).toBe(true);
    expect(wrapper.find('[data-testid="chat-submit"]').text()).toBe(
      "Continuar conversa"
    );
    expect(wrapper.text()).toContain("Another follow-up");
    expect(wrapper.text()).toContain("Pensando...");
    expect(wrapper.text()).not.toContain(
      "Sua mensagem foi enviada. Aguarde a resposta do Apollo."
    );
  });
});
