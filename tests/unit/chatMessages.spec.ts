import { describe, expect, it } from "vitest";

import { buildChatMessages } from "@/components/chat/chatMessages";
import type {
  ConversationMessage,
  InteractionSession
} from "@/composables/useApolloDesktop";

const session: InteractionSession = {
  id: "session-1",
  provider_kind: "OpenAi",
  model_key: "gpt-4.1-mini",
  source_kind: "ScreenCapture",
  ocr_text: "Texto capturado via OCR",
  user_notes: "Notas do usuario",
  request_prompt: "Prompt composto interno",
  response_text: "Resposta inicial do modelo"
};

describe("buildChatMessages", () => {
  it("uses persisted conversation messages while keeping the initial capture readable", () => {
    const messages: ConversationMessage[] = [
      {
        id: "message-1",
        session_id: "session-1",
        role: "User",
        content:
          "Captured text: Texto capturado via OCR\nUser notes: Notas do usuario"
      },
      {
        id: "message-2",
        session_id: "session-1",
        role: "Assistant",
        content: "Resposta inicial do modelo"
      }
    ];

    expect(buildChatMessages(session, messages)).toEqual([
      {
        id: "message-1",
        role: "user",
        content: "Texto capturado via OCR"
      },
      {
        id: "message-2",
        role: "assistant",
        content: "Resposta inicial do modelo"
      }
    ]);
  });

  it("falls back to the session OCR and answer when no persisted turns are loaded", () => {
    expect(buildChatMessages(session, [])).toEqual([
      {
        id: "session-1-ocr",
        role: "user",
        content: "Texto capturado via OCR"
      },
      {
        id: "session-1-response",
        role: "assistant",
        content: "Resposta inicial do modelo"
      }
    ]);
  });
});
