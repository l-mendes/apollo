import { mount } from "@vue/test-utils";
import { nextTick } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  continueConversationMock: vi.fn(),
  emitResponseConversationSyncMock: vi.fn(),
  hideMock: vi.fn(),
  minimizeMock: vi.fn()
}));

let responseUpdateHandler:
  | ((payload: {
      session_id: string;
      provider_kind: "OpenAi";
      model_key: string;
      reasoning_effort: "medium";
      display_messages: Array<{
        id: string;
        role: "assistant" | "system" | "user";
        content: string;
      }>;
      conversation_messages: Array<{
        id: string;
        session_id: string;
        role: "User" | "Assistant" | "System";
        content: string;
      }>;
    }) => void)
  | null = null;

vi.mock("@/composables/useApolloDesktop", () => ({
  commandErrorMessage: (error: unknown, fallback: string) =>
    error instanceof Error ? error.message : fallback,
  continueConversation: mocks.continueConversationMock,
  providerLabel: () => "OpenAI"
}));

vi.mock("@/composables/useWindowShell", () => ({
  emitResponseConversationSync: mocks.emitResponseConversationSyncMock,
  listenForResponseUpdate: vi.fn(async (handler) => {
    responseUpdateHandler = handler;
    return () => {};
  })
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    hide: mocks.hideMock,
    minimize: mocks.minimizeMock
  })
}));

import ResponseWindow from "@/ResponseWindow.vue";

function samplePayload() {
  return {
    session_id: "session-1",
    provider_kind: "OpenAi" as const,
    model_key: "gpt-4.1-mini",
    reasoning_effort: "medium" as const,
    display_messages: [
      {
        id: "message-1",
        role: "user" as const,
        content: "Texto capturado via OCR"
      },
      {
        id: "message-2",
        role: "assistant" as const,
        content: "Resposta inicial do modelo"
      }
    ],
    conversation_messages: [
      {
        id: "message-1",
        session_id: "session-1",
        role: "User" as const,
        content: "Captured text: Texto capturado via OCR"
      },
      {
        id: "message-2",
        session_id: "session-1",
        role: "Assistant" as const,
        content: "Resposta inicial do modelo"
      }
    ]
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise;
  });

  return { promise, resolve };
}

describe("ResponseWindow", () => {
  beforeEach(() => {
    mocks.continueConversationMock.mockReset();
    mocks.emitResponseConversationSyncMock
      .mockReset()
      .mockResolvedValue(undefined);
    mocks.hideMock.mockReset().mockResolvedValue(undefined);
    mocks.minimizeMock.mockReset().mockResolvedValue(undefined);
    responseUpdateHandler = null;
  });

  it("renders the conversation as chat messages without showing the sent prompt block", async () => {
    const wrapper = mount(ResponseWindow);
    await nextTick();

    responseUpdateHandler?.(samplePayload());
    await nextTick();

    expect(wrapper.text()).toContain("Chat do Apollo");
    expect(wrapper.text()).toContain("Texto capturado via OCR");
    expect(wrapper.text()).toContain("Resposta inicial do modelo");
    expect(wrapper.text()).not.toContain("Prompt enviado");
  });

  it("minimizes the response window from the header action", async () => {
    const wrapper = mount(ResponseWindow);
    await nextTick();

    await wrapper.find('[aria-label="Minimizar chat"]').trigger("click");
    await Promise.resolve();

    expect(mocks.minimizeMock).toHaveBeenCalled();
    expect(mocks.hideMock).not.toHaveBeenCalled();
  });

  it("continues the conversation from the response window and syncs the app window", async () => {
    const followUpResponse = {
      session_id: "session-1",
      response: {
        provider_kind: "OpenAi",
        model_key: "gpt-4.1-mini",
        answer: "Resposta de follow-up",
        raw_output: "Resposta de follow-up"
      },
      appended_messages: [
        {
          id: "message-3",
          session_id: "session-1",
          role: "User",
          content: "Me de mais exemplos"
        },
        {
          id: "message-4",
          session_id: "session-1",
          role: "Assistant",
          content: "Resposta de follow-up"
        }
      ]
    };
    mocks.continueConversationMock.mockResolvedValue(followUpResponse);

    const wrapper = mount(ResponseWindow);
    await nextTick();

    responseUpdateHandler?.(samplePayload());
    await nextTick();

    await wrapper
      .find('[data-testid="chat-composer"]')
      .setValue("Me de mais exemplos");
    await wrapper.find("form").trigger("submit");
    await nextTick();

    expect(mocks.continueConversationMock).toHaveBeenCalledWith({
      session_id: "session-1",
      provider_kind: "OpenAi",
      model_key: "gpt-4.1-mini",
      reasoning_effort: "medium",
      prompt: "Me de mais exemplos",
      existing_messages: samplePayload().conversation_messages
    });
    expect(mocks.emitResponseConversationSyncMock).toHaveBeenCalledWith({
      session_id: "session-1",
      prompt: "Me de mais exemplos",
      response: expect.objectContaining({
        answer: "Resposta de follow-up"
      }),
      appended_messages: expect.arrayContaining([
        expect.objectContaining({ id: "message-3" }),
        expect.objectContaining({ id: "message-4" })
      ])
    });
    expect(wrapper.text()).toContain("Me de mais exemplos");
    expect(wrapper.text()).toContain("Resposta de follow-up");
  });

  it("clears and locks the composer while showing a thinking state", async () => {
    const request = deferred<{
      session_id: string;
      response: {
        provider_kind: "OpenAi";
        model_key: string;
        answer: string;
        raw_output: string;
      };
      appended_messages: Array<{
        id: string;
        session_id: string;
        role: "User" | "Assistant";
        content: string;
      }>;
    }>();
    mocks.continueConversationMock.mockReturnValue(request.promise);

    const wrapper = mount(ResponseWindow);
    await nextTick();

    responseUpdateHandler?.(samplePayload());
    await nextTick();

    await wrapper
      .find('[data-testid="chat-composer"]')
      .setValue("Explique com mais exemplos");
    await wrapper.find("form").trigger("submit");
    await nextTick();

    const composer = wrapper.find<HTMLTextAreaElement>(
      '[data-testid="chat-composer"]'
    );

    expect(composer.element.value).toBe("");
    expect(composer.element.disabled).toBe(true);
    expect(wrapper.text()).toContain("Explique com mais exemplos");
    expect(wrapper.text()).toContain("Pensando...");
    expect(wrapper.find('[data-testid="chat-submit"]').text()).toBe(
      "Enviar mensagem"
    );

    request.resolve({
      session_id: "session-1",
      response: {
        provider_kind: "OpenAi",
        model_key: "gpt-4.1-mini",
        answer: "Resposta final",
        raw_output: "Resposta final"
      },
      appended_messages: [
        {
          id: "message-3",
          session_id: "session-1",
          role: "User",
          content: "Explique com mais exemplos"
        },
        {
          id: "message-4",
          session_id: "session-1",
          role: "Assistant",
          content: "Resposta final"
        }
      ]
    });
    await request.promise;
    await Promise.resolve();
    await nextTick();

    expect(
      wrapper.find<HTMLTextAreaElement>('[data-testid="chat-composer"]').element
        .disabled
    ).toBe(false);
    expect(wrapper.text()).not.toContain("Pensando...");
    expect(wrapper.text()).toContain("Resposta final");
  });
});
