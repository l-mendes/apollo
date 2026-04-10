import { beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn()
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock
}));

import {
  analyzeCapture,
  captureScreenRegion,
  commandErrorMessage,
  continueConversation,
  listHistory,
  loadConversationMessages,
  listProviderModelsFor,
  loadSettings,
  runOcrOnImage,
  saveSettings,
  type AnalyzeCaptureRequest,
  type CaptureRegionRequest,
  type ContinueConversationRequest,
  type UserSettings
} from "@/composables/useApolloDesktop";

describe("useApolloDesktop", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads persisted settings from the Tauri backend", async () => {
    invokeMock.mockResolvedValue({
      preferred_provider: "OpenAi",
      preferred_model: "gpt-4.1-mini",
      base_prompt: "prompt",
      shortcuts: []
    });

    const result = await loadSettings();

    expect(result.preferred_provider).toBe("OpenAi");
    expect(invokeMock).toHaveBeenCalledWith("load_settings");
  });

  it("persists settings through Tauri", async () => {
    const settings: UserSettings = {
      preferred_provider: "OpenAi",
      preferred_model: "gpt-4.1-mini",
      base_prompt: "prompt",
      ocr_language: "por",
      output_language: "Português",
      shortcuts: []
    };

    invokeMock.mockResolvedValue(undefined);

    await saveSettings(settings);

    expect(invokeMock).toHaveBeenCalledWith("save_settings", { settings });
  });

  it("lists history, conversation and provider models using command payloads compatible with Rust", async () => {
    invokeMock
      .mockResolvedValueOnce([{ id: "session-1" }])
      .mockResolvedValueOnce([{ id: "message-1", content: "hi" }])
      .mockResolvedValueOnce([
        {
          provider_kind: "OpenAi",
          channel: "Http",
          model_key: "gpt-4.1-mini",
          display_name: "GPT-4.1 Mini",
          manually_managed: true,
          is_default: true
        }
      ]);

    const history = await listHistory();
    const conversation = await loadConversationMessages("session-1");
    const models = await listProviderModelsFor("OpenAi");

    expect(history[0].id).toBe("session-1");
    expect(conversation[0].id).toBe("message-1");
    expect(models[0].display_name).toBe("GPT-4.1 Mini");
    expect(invokeMock).toHaveBeenNthCalledWith(1, "list_history");
    expect(invokeMock).toHaveBeenNthCalledWith(
      2,
      "load_conversation_messages",
      {
        sessionId: "session-1"
      }
    );
    expect(invokeMock).toHaveBeenNthCalledWith(3, "list_provider_models_for", {
      providerKind: "OpenAi"
    });
  });

  it("sends analyze requests with the normalized payload expected by the backend", async () => {
    const request: AnalyzeCaptureRequest = {
      provider_kind: "OpenAi",
      model_key: "gpt-4.1-mini",
      base_prompt: "Explain briefly.",
      ocr_text: "A sample sentence.",
      user_notes: "Focus on nuance.",
      conversation_context: []
    };

    invokeMock.mockResolvedValue({
      prompt: "prompt",
      session: { id: "session-1" },
      response: { answer: "answer" }
    });

    await analyzeCapture(request);

    expect(invokeMock).toHaveBeenCalledWith("analyze_capture", { request });
  });

  it("sends continue conversation requests with the current session context", async () => {
    const request: ContinueConversationRequest = {
      session_id: "session-1",
      provider_kind: "OpenAi",
      model_key: "gpt-4.1-mini",
      prompt: "Give me two more examples.",
      existing_messages: []
    };

    invokeMock.mockResolvedValue({
      session_id: "session-1",
      response: { answer: "answer" },
      appended_messages: []
    });

    await continueConversation(request);

    expect(invokeMock).toHaveBeenCalledWith("continue_conversation", {
      request
    });
  });

  it("sends capture region and OCR requests with the payload expected by the backend", async () => {
    const request: CaptureRegionRequest = {
      logical_x: 2020,
      logical_y: 40,
      logical_width: 320,
      logical_height: 200,
      physical_x: 4040,
      physical_y: 80,
      physical_width: 640,
      physical_height: 400,
      monitor_logical_x: 1920,
      monitor_logical_y: 0,
      monitor_logical_width: 1920,
      monitor_logical_height: 1080,
      monitor_physical_x: 3840,
      monitor_physical_y: 0,
      monitor_physical_width: 3840,
      monitor_physical_height: 2160
    };

    invokeMock.mockResolvedValueOnce({
      image_path: "/tmp/apollo-region.png",
      width: 640,
      height: 400,
      data_url: "data:image/png;base64,abc"
    });
    invokeMock.mockResolvedValueOnce("texto extraido");

    await captureScreenRegion(request);
    await runOcrOnImage("/tmp/apollo-region.png", "por");

    expect(invokeMock).toHaveBeenNthCalledWith(1, "capture_screen_region", {
      request
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, "run_ocr_on_image", {
      request: { image_path: "/tmp/apollo-region.png", ocr_language: "por" }
    });
  });

  it("normalizes backend and runtime failures into a readable string", () => {
    expect(
      commandErrorMessage(
        { kind: "Validation", message: "ocr text is required" },
        "fallback"
      )
    ).toBe("Validation: ocr text is required");
    expect(commandErrorMessage(new Error("failed"), "fallback")).toBe("failed");
    expect(commandErrorMessage(null, "fallback")).toBe("fallback");
  });
});
