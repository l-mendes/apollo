import type {
  ConversationMessage,
  InteractionSession
} from "@/composables/useApolloDesktop";

import type {
  ChatMessage,
  ChatMessageRole
} from "@/components/chat/chat.types";

function mapRole(role: ConversationMessage["role"]): ChatMessageRole {
  if (role === "Assistant") {
    return "assistant";
  }

  if (role === "System") {
    return "system";
  }

  return "user";
}

export function buildChatMessages(
  session: InteractionSession | null,
  conversationMessages: ConversationMessage[]
): ChatMessage[] {
  if (conversationMessages.length > 0) {
    return conversationMessages.map((message, index) => {
      const isInitialCaptureTurn =
        index === 0 &&
        message.role === "User" &&
        Boolean(session?.ocr_text?.trim());

      return {
        id: message.id,
        role: mapRole(message.role),
        content: isInitialCaptureTurn
          ? (session?.ocr_text?.trim() ?? message.content)
          : message.content
      };
    });
  }

  const fallbackMessages: ChatMessage[] = [];

  if (session?.ocr_text?.trim()) {
    fallbackMessages.push({
      id: `${session.id}-ocr`,
      role: "user",
      content: session.ocr_text.trim()
    });
  }

  if (session?.response_text?.trim()) {
    fallbackMessages.push({
      id: `${session.id}-response`,
      role: "assistant",
      content: session.response_text.trim()
    });
  }

  return fallbackMessages;
}
