#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    OpenAi,
    Anthropic,
    OllamaCloud,
    OllamaLocal,
    ClaudeCli,
    CodexCli,
    CopilotCli,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderChannel {
    Http,
    Cli,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderFailureKind {
    Authentication,
    Timeout,
    Unavailable,
    InvalidConfiguration,
    MissingCli,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderFailureSpec {
    pub kind: ProviderFailureKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderModelSpec {
    pub provider_kind: ProviderKind,
    pub channel: ProviderChannel,
    pub model_id: String,
    pub display_name: String,
    pub manually_managed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedResponseSpec {
    pub provider_kind: ProviderKind,
    pub model_id: String,
    pub answer: String,
    pub raw_output: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSpec {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzeRequestSpec {
    pub provider_kind: ProviderKind,
    pub model_id: String,
    pub base_prompt: String,
    pub ocr_text: String,
    pub user_notes: String,
    pub conversation_context: Vec<MessageSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureSpec {
    pub image_path: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcrFailureKind {
    EngineUnavailable,
    NoTextDetected,
    ImageUnreadable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcrFailureSpec {
    pub kind: OcrFailureKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OcrSuccessSpec {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutBindingSpec {
    pub action_key: String,
    pub accelerator: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsSpec {
    pub preferred_provider: ProviderKind,
    pub preferred_model_id: String,
    pub base_prompt: String,
    pub shortcuts: Vec<ShortcutBindingSpec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistenceFailureKind {
    NotFound,
    Conflict,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistenceFailureSpec {
    pub kind: PersistenceFailureKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistorySessionSpec {
    pub session_id: String,
    pub provider_kind: ProviderKind,
    pub model_id: String,
    pub ocr_text: String,
    pub user_notes: String,
    pub request_prompt: String,
    pub response_text: String,
}

pub trait Phase1BackendHarness {
    fn list_models(
        &self,
        provider_kind: ProviderKind,
    ) -> Result<Vec<ProviderModelSpec>, ProviderFailureSpec>;

    fn analyze(
        &self,
        request: &AnalyzeRequestSpec,
    ) -> Result<NormalizedResponseSpec, ProviderFailureSpec>;

    fn extract_text(&self, capture: &CaptureSpec) -> Result<OcrSuccessSpec, OcrFailureSpec>;

    fn save_settings(&self, settings: &SettingsSpec) -> Result<(), PersistenceFailureSpec>;

    fn load_settings(&self) -> Result<SettingsSpec, PersistenceFailureSpec>;

    fn save_session(&self, session: &HistorySessionSpec) -> Result<(), PersistenceFailureSpec>;

    fn append_message(
        &self,
        session_id: &str,
        message: &MessageSpec,
    ) -> Result<(), PersistenceFailureSpec>;

    fn list_sessions(&self) -> Result<Vec<HistorySessionSpec>, PersistenceFailureSpec>;

    fn load_conversation(
        &self,
        session_id: &str,
    ) -> Result<Vec<MessageSpec>, PersistenceFailureSpec>;

    fn compose_prompt(&self, request: &AnalyzeRequestSpec) -> String;

    fn execute_cli_analysis(
        &self,
        provider_kind: ProviderKind,
        request: &AnalyzeRequestSpec,
    ) -> Result<NormalizedResponseSpec, ProviderFailureSpec>;

    fn probe_cli_availability(
        &self,
        provider_kind: ProviderKind,
    ) -> Result<(), ProviderFailureSpec>;

    fn resolve_active_model(
        &self,
        preferred_provider: ProviderKind,
        preferred_model_id: &str,
    ) -> Result<ProviderModelSpec, ProviderFailureSpec>;
}

pub fn phase1_harness() -> Box<dyn Phase1BackendHarness> {
    panic!("Phase 2 will provide the concrete backend harness for these contracts")
}

pub fn sample_request(provider_kind: ProviderKind, model_id: &str) -> AnalyzeRequestSpec {
    AnalyzeRequestSpec {
        provider_kind,
        model_id: model_id.to_string(),
        base_prompt: "Act as a language tutor and explain with context.".to_string(),
        ocr_text: "I have been looking forward to this trip for ages.".to_string(),
        user_notes: "Explain the nuance of looking forward to.".to_string(),
        conversation_context: vec![
            MessageSpec {
                role: MessageRole::User,
                content: "What does this sentence mean?".to_string(),
            },
            MessageSpec {
                role: MessageRole::Assistant,
                content: "It expresses anticipation about a future event.".to_string(),
            },
        ],
    }
}

pub fn sample_capture() -> CaptureSpec {
    CaptureSpec {
        image_path: "/tmp/apollo-capture.png".to_string(),
        width: 1280,
        height: 720,
    }
}

pub fn sample_settings() -> SettingsSpec {
    SettingsSpec {
        preferred_provider: ProviderKind::OpenAi,
        preferred_model_id: "gpt-4.1-mini".to_string(),
        base_prompt: "Always answer in a concise teaching style.".to_string(),
        shortcuts: vec![
            ShortcutBindingSpec {
                action_key: "capture_screen".to_string(),
                accelerator: "CmdOrCtrl+Shift+A".to_string(),
            },
            ShortcutBindingSpec {
                action_key: "open_settings".to_string(),
                accelerator: "CmdOrCtrl+,".to_string(),
            },
        ],
    }
}

pub fn sample_history_session() -> HistorySessionSpec {
    HistorySessionSpec {
        session_id: "session-001".to_string(),
        provider_kind: ProviderKind::Anthropic,
        model_id: "claude-3-7-sonnet".to_string(),
        ocr_text: "She made up her mind after reading the article.".to_string(),
        user_notes: "Explain the phrasal verb made up her mind.".to_string(),
        request_prompt: "Explain the phrase and give two examples.".to_string(),
        response_text: "It means she decided something firmly.".to_string(),
    }
}

pub fn sample_follow_up_message() -> MessageSpec {
    MessageSpec {
        role: MessageRole::User,
        content: "Can you give me two more examples in a casual tone?".to_string(),
    }
}
