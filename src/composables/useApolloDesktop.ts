import { invoke } from "@tauri-apps/api/core";

export const PROVIDER_OPTIONS = [
  { kind: "OpenAi", label: "OpenAI", channel: "Http" },
  { kind: "Anthropic", label: "Anthropic", channel: "Http" },
  { kind: "OllamaCloud", label: "Ollama Cloud", channel: "Http" },
  { kind: "OllamaLocal", label: "Ollama Local", channel: "Http" },
  { kind: "ClaudeCli", label: "Claude CLI", channel: "Cli" },
  { kind: "CodexCli", label: "Codex CLI", channel: "Cli" },
  { kind: "CopilotCli", label: "Copilot CLI", channel: "Cli" }
] as const;

export type ProviderKind = (typeof PROVIDER_OPTIONS)[number]["kind"];
export type ProviderChannel = "Http" | "Cli";
export type AnalysisSourceKind = "ScreenCapture" | "ManualText" | "FileImport";
export type SurfaceId = "none" | "home" | "history" | "settings";

export interface ShortcutBinding {
  action: string;
  accelerator: string;
  enabled: boolean;
}

export interface UserSettings {
  preferred_provider: ProviderKind;
  preferred_model: string;
  base_prompt: string;
  /** Tesseract language code used for OCR (e.g. "por", "eng"). */
  ocr_language: string;
  /** Natural-language name passed to the AI to request responses in a specific language. */
  output_language: string;
  shortcuts: ShortcutBinding[];
}

export interface ProviderModel {
  provider_kind: ProviderKind;
  channel: ProviderChannel;
  model_key: string;
  display_name: string;
  manually_managed: boolean;
  is_default: boolean;
}

export interface InteractionSession {
  id: string;
  provider_kind: ProviderKind;
  model_key: string;
  source_kind: AnalysisSourceKind;
  ocr_text: string | null;
  user_notes: string | null;
  request_prompt: string | null;
  response_text: string | null;
}

export interface ConversationMessage {
  id: string;
  session_id: string;
  role: "System" | "User" | "Assistant";
  content: string;
}

export interface NormalizedResponse {
  provider_kind: ProviderKind;
  model_key: string;
  answer: string;
  raw_output: string;
}

export interface AnalyzeCaptureRequest {
  provider_kind: ProviderKind;
  model_key: string;
  base_prompt: string;
  ocr_text: string;
  user_notes: string | null;
  conversation_context: ConversationMessage[];
}

export interface AnalyzeCaptureResponse {
  prompt: string;
  session: InteractionSession;
  response: NormalizedResponse;
}

export interface ContinueConversationRequest {
  session_id: string;
  provider_kind: ProviderKind;
  model_key: string;
  prompt: string;
  existing_messages: ConversationMessage[];
}

export interface ContinueConversationResponse {
  session_id: string;
  response: NormalizedResponse;
  appended_messages: ConversationMessage[];
}

export type ProviderCatalog = Record<ProviderKind, ProviderModel[]>;

export function createEmptyProviderCatalog(): ProviderCatalog {
  return {
    OpenAi: [],
    Anthropic: [],
    OllamaCloud: [],
    OllamaLocal: [],
    ClaudeCli: [],
    CodexCli: [],
    CopilotCli: []
  };
}

export function cloneSettings(settings: UserSettings): UserSettings {
  return {
    ...settings,
    shortcuts: settings.shortcuts.map((shortcut) => ({ ...shortcut }))
  };
}

export function providerLabel(providerKind: ProviderKind): string {
  return (
    PROVIDER_OPTIONS.find((provider) => provider.kind === providerKind)
      ?.label ?? providerKind
  );
}

export function commandErrorMessage(
  error: unknown,
  fallbackMessage: string
): string {
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }

  if (typeof error === "string" && error.trim()) {
    return error;
  }

  if (error && typeof error === "object") {
    const message =
      "message" in error &&
      typeof error.message === "string" &&
      error.message.trim()
        ? error.message
        : null;
    const kind =
      "kind" in error && typeof error.kind === "string" && error.kind.trim()
        ? error.kind
        : null;

    if (message && kind) {
      return `${kind}: ${message}`;
    }

    if (message) {
      return message;
    }
  }

  return fallbackMessage;
}

export async function loadSettings(): Promise<UserSettings> {
  return invoke<UserSettings>("load_settings");
}

export async function saveSettings(settings: UserSettings): Promise<void> {
  await invoke("save_settings", { settings });
}

export async function listHistory(): Promise<InteractionSession[]> {
  return invoke<InteractionSession[]>("list_history");
}

export async function loadConversationMessages(
  sessionId: string
): Promise<ConversationMessage[]> {
  return invoke<ConversationMessage[]>("load_conversation_messages", {
    sessionId
  });
}

export async function listProviderModelsFor(
  providerKind: ProviderKind
): Promise<ProviderModel[]> {
  return invoke<ProviderModel[]>("list_provider_models_for", {
    providerKind
  });
}

export async function analyzeCapture(
  request: AnalyzeCaptureRequest
): Promise<AnalyzeCaptureResponse> {
  return invoke<AnalyzeCaptureResponse>("analyze_capture", { request });
}

export async function continueConversation(
  request: ContinueConversationRequest
): Promise<ContinueConversationResponse> {
  return invoke<ContinueConversationResponse>("continue_conversation", {
    request
  });
}

export async function triggerScreenCapture(): Promise<string> {
  return invoke<string>("trigger_screen_capture");
}

export interface CaptureRegionRequest {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface CaptureRegionResponse {
  image_path: string;
  width: number;
  height: number;
  data_url: string;
}

export async function captureScreenRegion(
  request: CaptureRegionRequest
): Promise<CaptureRegionResponse> {
  return invoke<CaptureRegionResponse>("capture_screen_region", { request });
}

export async function runOcrOnImage(
  imagePath: string,
  ocrLanguage: string
): Promise<string> {
  return invoke<string>("run_ocr_on_image", {
    request: { image_path: imagePath, ocr_language: ocrLanguage }
  });
}

export async function applyGlobalShortcuts(
  shortcuts: ShortcutBinding[]
): Promise<void> {
  await invoke("apply_global_shortcuts", { shortcuts });
}
