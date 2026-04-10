import { mount } from "@vue/test-utils";

import HistorySurface from "@/components/surfaces/HistorySurface.vue";

describe("HistorySurface", () => {
  it("renders an empty state when there are no sessions yet", () => {
    const wrapper = mount(HistorySurface, {
      props: {
        loading: false,
        errorText: null,
        sessions: [],
        selectedSessionId: null,
        conversationMessages: [],
        conversationLoading: false,
        conversationErrorText: null,
        continuePrompt: "",
        continueLoading: false,
        continueErrorText: null
      }
    });

    expect(wrapper.find('[data-testid="history-empty"]').exists()).toBe(true);
    expect(wrapper.text()).toContain("Nenhuma sessao foi registrada ainda");
  });

  it("renders persisted sessions and emits selection changes", async () => {
    const wrapper = mount(HistorySurface, {
      props: {
        loading: false,
        errorText: null,
        sessions: [
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
        selectedSessionId: "session-1",
        conversationMessages: [],
        conversationLoading: false,
        conversationErrorText: null,
        continuePrompt: "",
        continueLoading: false,
        continueErrorText: null
      }
    });

    await wrapper.find("button").trigger("click");

    expect(wrapper.find('[data-testid="history-ready"]').exists()).toBe(true);
    expect(wrapper.text()).toContain("Sessao selecionada");
    expect(wrapper.emitted("select-session")?.[0]).toEqual(["session-1"]);
  });

  it("emits follow-up actions for conversation continuation", async () => {
    const wrapper = mount(HistorySurface, {
      props: {
        loading: false,
        errorText: null,
        sessions: [
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
        selectedSessionId: "session-1",
        conversationMessages: [
          {
            id: "message-1",
            session_id: "session-1",
            role: "User",
            content: "Original question"
          }
        ],
        conversationLoading: false,
        conversationErrorText: null,
        continuePrompt: "Give me two more examples",
        continueLoading: false,
        continueErrorText: null
      }
    });

    await wrapper
      .find('[data-testid="continue-prompt"]')
      .setValue("Another follow-up");
    await wrapper.find('[data-testid="continue-button"]').trigger("click");

    expect(wrapper.emitted("update:continue-prompt")?.[0]).toEqual([
      "Another follow-up"
    ]);
    expect(wrapper.emitted("continue-conversation")).toHaveLength(1);
  });
});
