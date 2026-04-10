export type ChatMessageRole = "assistant" | "system" | "user";

export interface ChatMessage {
  id: string;
  role: ChatMessageRole;
  content: string;
}
