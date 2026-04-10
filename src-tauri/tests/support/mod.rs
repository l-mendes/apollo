#![allow(dead_code)]

use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use apollo::{
    application::{
        dto::{
            analysis::{
                AnalyzeCaptureRequest, ContinueConversationRequest, ListProviderModelsRequest,
                NormalizedResponse,
            },
            ocr::OcrTextExtraction,
        },
        errors::{ApplicationError, ApplicationErrorKind},
        ports::{
            capture::OcrEngine,
            provider::{AiProvider, CliProviderExecutor, ProviderRegistry},
            repositories::{ConversationRepository, HistoryRepository, SettingsRepository},
        },
        services::prompt_builder::PromptBuilder,
        use_cases::{
            analyze_capture::AnalyzeCapture, analyze_capture_service::AnalyzeCaptureService,
            continue_conversation::ContinueConversation,
            continue_conversation_service::ContinueConversationService,
            list_provider_models::ListProviderModels,
            list_provider_models_service::ListProviderModelsService,
        },
    },
    domain::{
        entities::{
            capture_record::{CaptureRecord, OcrStatus},
            configured_provider as domain_provider,
            conversation_message::{ConversationMessage, MessageRole as DomainMessageRole},
            interaction_session::{AnalysisSourceKind, InteractionSession},
            shortcut_binding::ShortcutBinding,
            user_settings::UserSettings,
        },
        value_objects::{
            identifiers::{CaptureId, MessageId, SessionId},
            model_key::ModelKey,
            reasoning_effort::ReasoningEffort,
            shortcut::{ShortcutAccelerator, ShortcutAction},
        },
    },
    infrastructure::{
        persistence::SqliteAppRepository, provider_catalog::ManualProviderModelCatalog,
        provider_registry::InMemoryProviderRegistry,
    },
};

static IDENTIFIER_SEQUENCE: AtomicU64 = AtomicU64::new(1);

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
    pub reasoning_effort: ReasoningEffortSpec,
    pub base_prompt: String,
    pub shortcuts: Vec<ShortcutBindingSpec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasoningEffortSpec {
    Low,
    Medium,
    High,
    XHigh,
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

pub trait BackendContractHarness {
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

    fn continue_conversation(
        &self,
        session_id: &str,
        provider_kind: ProviderKind,
        model_id: &str,
        prompt: &str,
    ) -> Result<Vec<MessageSpec>, ProviderFailureSpec>;

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

struct ContractAiProvider {
    kind: domain_provider::ProviderKind,
}

#[async_trait::async_trait]
impl AiProvider for ContractAiProvider {
    fn kind(&self) -> domain_provider::ProviderKind {
        self.kind
    }

    async fn list_models(
        &self,
    ) -> Result<Vec<apollo::domain::entities::provider_model::ProviderModel>, ApplicationError>
    {
        Ok(Vec::new())
    }

    async fn analyze(
        &self,
        request: &AnalyzeCaptureRequest,
    ) -> Result<NormalizedResponse, ApplicationError> {
        match self.kind {
            domain_provider::ProviderKind::OpenAi => Ok(success_response(
                request.provider_kind,
                request.model_key.as_str(),
                "OpenAI normalized answer",
                "{\"provider\":\"openai\",\"answer\":\"OpenAI normalized answer\"}",
            )),
            domain_provider::ProviderKind::Anthropic => Err(ApplicationError::new(
                ApplicationErrorKind::Timeout,
                "Anthropic request timed out",
            )),
            domain_provider::ProviderKind::OllamaCloud => Err(ApplicationError::new(
                ApplicationErrorKind::Unavailable,
                "Ollama Cloud is unavailable",
            )),
            domain_provider::ProviderKind::OllamaLocal => Ok(success_response(
                request.provider_kind,
                request.model_key.as_str(),
                "Ollama local normalized answer",
                "{\"provider\":\"ollama_local\",\"answer\":\"Ollama local normalized answer\"}",
            )),
            _ => Err(ApplicationError::new(
                ApplicationErrorKind::InvalidConfiguration,
                "HTTP provider kind expected",
            )),
        }
    }
}

struct ContractCliProvider {
    kind: domain_provider::ProviderKind,
}

#[async_trait::async_trait]
impl CliProviderExecutor for ContractCliProvider {
    fn kind(&self) -> domain_provider::ProviderKind {
        self.kind
    }

    async fn probe_availability(&self) -> Result<(), ApplicationError> {
        match self.kind {
            domain_provider::ProviderKind::CopilotCli => Err(ApplicationError::new(
                ApplicationErrorKind::MissingCli,
                "Copilot CLI is not installed",
            )),
            _ => Ok(()),
        }
    }

    async fn execute(
        &self,
        request: &AnalyzeCaptureRequest,
    ) -> Result<NormalizedResponse, ApplicationError> {
        match self.kind {
            domain_provider::ProviderKind::CodexCli => Ok(success_response(
                request.provider_kind,
                request.model_key.as_str(),
                "Codex normalized answer",
                "Codex normalized answer",
            )),
            domain_provider::ProviderKind::ClaudeCli => Err(ApplicationError::new(
                ApplicationErrorKind::Authentication,
                "Claude CLI authentication required",
            )),
            domain_provider::ProviderKind::CopilotCli => Err(ApplicationError::new(
                ApplicationErrorKind::MissingCli,
                "Copilot CLI is not installed",
            )),
            _ => Err(ApplicationError::new(
                ApplicationErrorKind::InvalidConfiguration,
                "CLI provider kind expected",
            )),
        }
    }
}

struct ScenarioOcrEngine;

#[async_trait::async_trait]
impl OcrEngine for ScenarioOcrEngine {
    async fn extract_text(
        &self,
        capture: &CaptureRecord,
    ) -> Result<OcrTextExtraction, ApplicationError> {
        if capture.image_path.contains("missing-engine") {
            return Err(ApplicationError::new(
                ApplicationErrorKind::Unavailable,
                "OCR engine is unavailable",
            ));
        }

        if capture.image_path.contains("empty") {
            return Err(ApplicationError::new(
                ApplicationErrorKind::Validation,
                "No text detected in capture",
            ));
        }

        if capture.image_path.contains("unreadable") {
            return Err(ApplicationError::new(
                ApplicationErrorKind::NotFound,
                "Capture image could not be read",
            ));
        }

        Ok(OcrTextExtraction {
            text: "Extracted text from capture".to_string(),
        })
    }
}

struct ContractBackendHarness {
    repository: Arc<SqliteAppRepository>,
    catalog: Arc<ManualProviderModelCatalog>,
    registry: Arc<InMemoryProviderRegistry>,
    ocr_engine: Arc<dyn OcrEngine>,
    prompt_builder: PromptBuilder,
}

impl ContractBackendHarness {
    fn new() -> Self {
        let repository = Arc::new(
            SqliteAppRepository::in_memory().expect("in-memory repository should initialize"),
        );
        let catalog = Arc::new(ManualProviderModelCatalog::new());
        let registry = Arc::new(
            InMemoryProviderRegistry::new(catalog.clone())
                .with_ai_provider(Arc::new(ContractAiProvider {
                    kind: domain_provider::ProviderKind::OpenAi,
                }))
                .with_ai_provider(Arc::new(ContractAiProvider {
                    kind: domain_provider::ProviderKind::Anthropic,
                }))
                .with_ai_provider(Arc::new(ContractAiProvider {
                    kind: domain_provider::ProviderKind::OllamaCloud,
                }))
                .with_ai_provider(Arc::new(ContractAiProvider {
                    kind: domain_provider::ProviderKind::OllamaLocal,
                }))
                .with_cli_provider(Arc::new(ContractCliProvider {
                    kind: domain_provider::ProviderKind::ClaudeCli,
                }))
                .with_cli_provider(Arc::new(ContractCliProvider {
                    kind: domain_provider::ProviderKind::CodexCli,
                }))
                .with_cli_provider(Arc::new(ContractCliProvider {
                    kind: domain_provider::ProviderKind::CopilotCli,
                })),
        );

        Self {
            repository,
            catalog,
            registry,
            ocr_engine: Arc::new(ScenarioOcrEngine),
            prompt_builder: PromptBuilder::new(),
        }
    }

    fn analyze_service(&self) -> AnalyzeCaptureService {
        AnalyzeCaptureService::new(
            self.registry.clone(),
            self.repository.clone(),
            self.repository.clone(),
        )
    }

    fn continue_service(&self) -> ContinueConversationService {
        ContinueConversationService::new(
            self.registry.clone(),
            self.repository.clone(),
            self.repository.clone(),
        )
    }

    fn list_models_service(&self) -> ListProviderModelsService {
        ListProviderModelsService::new(self.catalog.clone())
    }
}

impl BackendContractHarness for ContractBackendHarness {
    fn list_models(
        &self,
        provider_kind: ProviderKind,
    ) -> Result<Vec<ProviderModelSpec>, ProviderFailureSpec> {
        tauri::async_runtime::block_on(async {
            self.list_models_service()
                .execute(ListProviderModelsRequest {
                    provider_kind: provider_kind.into(),
                })
                .await
                .map(|response| response.models.into_iter().map(Into::into).collect())
                .map_err(map_provider_error)
        })
    }

    fn analyze(
        &self,
        request: &AnalyzeRequestSpec,
    ) -> Result<NormalizedResponseSpec, ProviderFailureSpec> {
        tauri::async_runtime::block_on(async {
            self.analyze_service()
                .execute(to_analyze_request(request))
                .await
                .map(|response| response.response.into())
                .map_err(map_provider_error)
        })
    }

    fn extract_text(&self, capture: &CaptureSpec) -> Result<OcrSuccessSpec, OcrFailureSpec> {
        tauri::async_runtime::block_on(async {
            self.ocr_engine
                .extract_text(&to_capture_record(capture))
                .await
                .map(|result| OcrSuccessSpec { text: result.text })
                .map_err(map_ocr_error)
        })
    }

    fn save_settings(&self, settings: &SettingsSpec) -> Result<(), PersistenceFailureSpec> {
        tauri::async_runtime::block_on(async {
            SettingsRepository::save(self.repository.as_ref(), &to_user_settings(settings))
                .await
                .map_err(map_persistence_error)
        })
    }

    fn load_settings(&self) -> Result<SettingsSpec, PersistenceFailureSpec> {
        tauri::async_runtime::block_on(async {
            SettingsRepository::load(self.repository.as_ref())
                .await
                .map(Into::into)
                .map_err(map_persistence_error)
        })
    }

    fn save_session(&self, session: &HistorySessionSpec) -> Result<(), PersistenceFailureSpec> {
        tauri::async_runtime::block_on(async {
            HistoryRepository::save_session(
                self.repository.as_ref(),
                &to_interaction_session(session),
            )
            .await
            .map_err(map_persistence_error)
        })
    }

    fn append_message(
        &self,
        session_id: &str,
        message: &MessageSpec,
    ) -> Result<(), PersistenceFailureSpec> {
        tauri::async_runtime::block_on(async {
            ConversationRepository::append_message(
                self.repository.as_ref(),
                &to_conversation_message(session_id, message),
            )
            .await
            .map_err(map_persistence_error)
        })
    }

    fn list_sessions(&self) -> Result<Vec<HistorySessionSpec>, PersistenceFailureSpec> {
        tauri::async_runtime::block_on(async {
            HistoryRepository::list_sessions(self.repository.as_ref())
                .await
                .map(|sessions| sessions.into_iter().map(Into::into).collect())
                .map_err(map_persistence_error)
        })
    }

    fn load_conversation(
        &self,
        session_id: &str,
    ) -> Result<Vec<MessageSpec>, PersistenceFailureSpec> {
        tauri::async_runtime::block_on(async {
            ConversationRepository::load_by_session(
                self.repository.as_ref(),
                &SessionId::new(session_id).expect("session id should be valid"),
            )
            .await
            .map(|messages| messages.into_iter().map(Into::into).collect())
            .map_err(map_persistence_error)
        })
    }

    fn continue_conversation(
        &self,
        session_id: &str,
        provider_kind: ProviderKind,
        model_id: &str,
        prompt: &str,
    ) -> Result<Vec<MessageSpec>, ProviderFailureSpec> {
        tauri::async_runtime::block_on(async {
            let session_id = SessionId::new(session_id).expect("session id should be valid");
            let existing_messages =
                ConversationRepository::load_by_session(self.repository.as_ref(), &session_id)
                    .await
                    .map_err(map_provider_error)?;

            self.continue_service()
                .execute(ContinueConversationRequest {
                    session_id: session_id.clone(),
                    provider_kind: provider_kind.into(),
                    model_key: ModelKey::new(model_id).expect("model key should be valid"),
                    reasoning_effort: ReasoningEffort::Medium,
                    prompt: prompt.to_string(),
                    existing_messages,
                })
                .await
                .map(|response| {
                    response
                        .appended_messages
                        .into_iter()
                        .map(Into::into)
                        .collect()
                })
                .map_err(map_provider_error)
        })
    }

    fn compose_prompt(&self, request: &AnalyzeRequestSpec) -> String {
        self.prompt_builder
            .compose_analysis_prompt(&to_analyze_request(request))
    }

    fn execute_cli_analysis(
        &self,
        _provider_kind: ProviderKind,
        request: &AnalyzeRequestSpec,
    ) -> Result<NormalizedResponseSpec, ProviderFailureSpec> {
        tauri::async_runtime::block_on(async {
            self.analyze_service()
                .execute(to_analyze_request(request))
                .await
                .map(|response| response.response.into())
                .map_err(map_provider_error)
        })
    }

    fn probe_cli_availability(
        &self,
        provider_kind: ProviderKind,
    ) -> Result<(), ProviderFailureSpec> {
        tauri::async_runtime::block_on(async {
            self.registry
                .resolve_cli(provider_kind.into())
                .map_err(map_provider_error)?
                .probe_availability()
                .await
                .map_err(map_provider_error)
        })
    }

    fn resolve_active_model(
        &self,
        preferred_provider: ProviderKind,
        preferred_model_id: &str,
    ) -> Result<ProviderModelSpec, ProviderFailureSpec> {
        self.registry
            .resolve_model(
                preferred_provider.into(),
                &ModelKey::new(preferred_model_id).expect("model key should be valid"),
            )
            .map(Into::into)
            .map_err(map_provider_error)
    }
}

pub fn contract_harness() -> Box<dyn BackendContractHarness> {
    Box::new(ContractBackendHarness::new())
}

fn success_response(
    provider_kind: domain_provider::ProviderKind,
    model_key: &str,
    answer: &str,
    raw_output: &str,
) -> NormalizedResponse {
    NormalizedResponse {
        provider_kind,
        model_key: ModelKey::new(model_key).expect("model key should be valid"),
        answer: answer.to_string(),
        raw_output: raw_output.to_string(),
    }
}

fn to_analyze_request(spec: &AnalyzeRequestSpec) -> AnalyzeCaptureRequest {
    let session_id = SessionId::new("session-contract").expect("session id should be valid");

    AnalyzeCaptureRequest {
        provider_kind: spec.provider_kind.into(),
        model_key: ModelKey::new(&spec.model_id).expect("model key should be valid"),
        reasoning_effort: ReasoningEffort::Medium,
        base_prompt: spec.base_prompt.clone(),
        ocr_text: spec.ocr_text.clone(),
        user_notes: Some(spec.user_notes.clone()),
        conversation_context: spec
            .conversation_context
            .iter()
            .enumerate()
            .map(|(index, message)| {
                ConversationMessage::new(
                    MessageId::new(format!("message-context-{index}"))
                        .expect("message id should be valid"),
                    session_id.clone(),
                    message.role.into(),
                    message.content.clone(),
                )
                .expect("message should be valid")
            })
            .collect(),
    }
}

fn to_capture_record(capture: &CaptureSpec) -> CaptureRecord {
    CaptureRecord {
        id: CaptureId::new(next_identifier("capture")).expect("capture id should be valid"),
        session_id: None,
        image_path: capture.image_path.clone(),
        width: capture.width,
        height: capture.height,
        ocr_status: OcrStatus::Pending,
    }
}

fn to_user_settings(settings: &SettingsSpec) -> UserSettings {
    let defaults = UserSettings::default();

    UserSettings {
        preferred_provider: settings.preferred_provider.into(),
        preferred_model: ModelKey::new(&settings.preferred_model_id)
            .expect("model key should be valid"),
        reasoning_effort: settings.reasoning_effort.into(),
        base_prompt: settings.base_prompt.clone(),
        ocr_language: defaults.ocr_language,
        output_language: defaults.output_language,
        shortcuts: settings
            .shortcuts
            .iter()
            .map(|shortcut| ShortcutBinding {
                action: ShortcutAction::new(&shortcut.action_key)
                    .expect("shortcut action should be valid"),
                accelerator: ShortcutAccelerator::new(&shortcut.accelerator)
                    .expect("shortcut accelerator should be valid"),
                enabled: true,
            })
            .collect(),
    }
}

fn to_interaction_session(session: &HistorySessionSpec) -> InteractionSession {
    InteractionSession {
        id: SessionId::new(&session.session_id).expect("session id should be valid"),
        provider_kind: session.provider_kind.into(),
        model_key: ModelKey::new(&session.model_id).expect("model key should be valid"),
        source_kind: AnalysisSourceKind::ScreenCapture,
        ocr_text: Some(session.ocr_text.clone()),
        user_notes: Some(session.user_notes.clone()),
        request_prompt: Some(session.request_prompt.clone()),
        response_text: Some(session.response_text.clone()),
    }
}

fn to_conversation_message(session_id: &str, message: &MessageSpec) -> ConversationMessage {
    ConversationMessage::new(
        MessageId::new(next_identifier("message")).expect("message id should be valid"),
        SessionId::new(session_id).expect("session id should be valid"),
        message.role.into(),
        message.content.clone(),
    )
    .expect("message should be valid")
}

fn next_identifier(prefix: &str) -> String {
    let sequence = IDENTIFIER_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();

    format!("{prefix}-{nanos}-{sequence}")
}

fn map_provider_error(error: ApplicationError) -> ProviderFailureSpec {
    let kind = match error.kind {
        ApplicationErrorKind::Authentication => ProviderFailureKind::Authentication,
        ApplicationErrorKind::Timeout => ProviderFailureKind::Timeout,
        ApplicationErrorKind::MissingCli => ProviderFailureKind::MissingCli,
        ApplicationErrorKind::Validation | ApplicationErrorKind::InvalidConfiguration => {
            ProviderFailureKind::InvalidConfiguration
        }
        ApplicationErrorKind::Unavailable
        | ApplicationErrorKind::NotFound
        | ApplicationErrorKind::Conflict
        | ApplicationErrorKind::Unknown => ProviderFailureKind::Unavailable,
    };

    ProviderFailureSpec {
        kind,
        message: error.message,
    }
}

fn map_persistence_error(error: ApplicationError) -> PersistenceFailureSpec {
    let kind = match error.kind {
        ApplicationErrorKind::NotFound => PersistenceFailureKind::NotFound,
        ApplicationErrorKind::Conflict => PersistenceFailureKind::Conflict,
        _ => PersistenceFailureKind::Unavailable,
    };

    PersistenceFailureSpec {
        kind,
        message: error.message,
    }
}

fn map_ocr_error(error: ApplicationError) -> OcrFailureSpec {
    let kind = match error.kind {
        ApplicationErrorKind::Unavailable => OcrFailureKind::EngineUnavailable,
        ApplicationErrorKind::Validation => OcrFailureKind::NoTextDetected,
        _ => OcrFailureKind::ImageUnreadable,
    };

    OcrFailureSpec {
        kind,
        message: error.message,
    }
}

impl From<ProviderKind> for domain_provider::ProviderKind {
    fn from(value: ProviderKind) -> Self {
        match value {
            ProviderKind::OpenAi => Self::OpenAi,
            ProviderKind::Anthropic => Self::Anthropic,
            ProviderKind::OllamaCloud => Self::OllamaCloud,
            ProviderKind::OllamaLocal => Self::OllamaLocal,
            ProviderKind::ClaudeCli => Self::ClaudeCli,
            ProviderKind::CodexCli => Self::CodexCli,
            ProviderKind::CopilotCli => Self::CopilotCli,
        }
    }
}

impl From<domain_provider::ProviderKind> for ProviderKind {
    fn from(value: domain_provider::ProviderKind) -> Self {
        match value {
            domain_provider::ProviderKind::OpenAi => Self::OpenAi,
            domain_provider::ProviderKind::Anthropic => Self::Anthropic,
            domain_provider::ProviderKind::OllamaCloud => Self::OllamaCloud,
            domain_provider::ProviderKind::OllamaLocal => Self::OllamaLocal,
            domain_provider::ProviderKind::ClaudeCli => Self::ClaudeCli,
            domain_provider::ProviderKind::CodexCli => Self::CodexCli,
            domain_provider::ProviderKind::CopilotCli => Self::CopilotCli,
        }
    }
}

impl From<domain_provider::ProviderChannel> for ProviderChannel {
    fn from(value: domain_provider::ProviderChannel) -> Self {
        match value {
            domain_provider::ProviderChannel::Http => Self::Http,
            domain_provider::ProviderChannel::Cli => Self::Cli,
        }
    }
}

impl From<ReasoningEffortSpec> for ReasoningEffort {
    fn from(value: ReasoningEffortSpec) -> Self {
        match value {
            ReasoningEffortSpec::Low => Self::Low,
            ReasoningEffortSpec::Medium => Self::Medium,
            ReasoningEffortSpec::High => Self::High,
            ReasoningEffortSpec::XHigh => Self::XHigh,
        }
    }
}

impl From<ReasoningEffort> for ReasoningEffortSpec {
    fn from(value: ReasoningEffort) -> Self {
        match value {
            ReasoningEffort::Low => Self::Low,
            ReasoningEffort::Medium => Self::Medium,
            ReasoningEffort::High => Self::High,
            ReasoningEffort::XHigh => Self::XHigh,
        }
    }
}

impl From<MessageRole> for DomainMessageRole {
    fn from(value: MessageRole) -> Self {
        match value {
            MessageRole::System => Self::System,
            MessageRole::User => Self::User,
            MessageRole::Assistant => Self::Assistant,
        }
    }
}

impl From<ConversationMessage> for MessageSpec {
    fn from(value: ConversationMessage) -> Self {
        Self {
            role: value.role.into(),
            content: value.content,
        }
    }
}

impl From<DomainMessageRole> for MessageRole {
    fn from(value: DomainMessageRole) -> Self {
        match value {
            DomainMessageRole::System => Self::System,
            DomainMessageRole::User => Self::User,
            DomainMessageRole::Assistant => Self::Assistant,
        }
    }
}

impl From<apollo::domain::entities::provider_model::ProviderModel> for ProviderModelSpec {
    fn from(value: apollo::domain::entities::provider_model::ProviderModel) -> Self {
        Self {
            provider_kind: value.provider_kind.into(),
            channel: value.channel.into(),
            model_id: value.model_key.as_str().to_string(),
            display_name: value.display_name,
            manually_managed: value.manually_managed,
        }
    }
}

impl From<NormalizedResponse> for NormalizedResponseSpec {
    fn from(value: NormalizedResponse) -> Self {
        Self {
            provider_kind: value.provider_kind.into(),
            model_id: value.model_key.as_str().to_string(),
            answer: value.answer,
            raw_output: value.raw_output,
        }
    }
}

impl From<UserSettings> for SettingsSpec {
    fn from(value: UserSettings) -> Self {
        Self {
            preferred_provider: value.preferred_provider.into(),
            preferred_model_id: value.preferred_model.as_str().to_string(),
            reasoning_effort: value.reasoning_effort.into(),
            base_prompt: value.base_prompt,
            shortcuts: value
                .shortcuts
                .into_iter()
                .map(|shortcut| ShortcutBindingSpec {
                    action_key: shortcut.action.as_str().to_string(),
                    accelerator: shortcut.accelerator.as_str().to_string(),
                })
                .collect(),
        }
    }
}

impl From<InteractionSession> for HistorySessionSpec {
    fn from(value: InteractionSession) -> Self {
        Self {
            session_id: value.id.as_str().to_string(),
            provider_kind: value.provider_kind.into(),
            model_id: value.model_key.as_str().to_string(),
            ocr_text: value.ocr_text.unwrap_or_default(),
            user_notes: value.user_notes.unwrap_or_default(),
            request_prompt: value.request_prompt.unwrap_or_default(),
            response_text: value.response_text.unwrap_or_default(),
        }
    }
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
        image_path: "/tmp/apollo-capture-valid.png".to_string(),
        width: 1280,
        height: 720,
    }
}

pub fn sample_empty_capture() -> CaptureSpec {
    CaptureSpec {
        image_path: "/tmp/apollo-capture-empty.png".to_string(),
        width: 1280,
        height: 720,
    }
}

pub fn sample_missing_engine_capture() -> CaptureSpec {
    CaptureSpec {
        image_path: "/tmp/apollo-capture-missing-engine.png".to_string(),
        width: 1280,
        height: 720,
    }
}

pub fn sample_settings() -> SettingsSpec {
    SettingsSpec {
        preferred_provider: ProviderKind::OpenAi,
        preferred_model_id: "gpt-4.1-mini".to_string(),
        reasoning_effort: ReasoningEffortSpec::Medium,
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
