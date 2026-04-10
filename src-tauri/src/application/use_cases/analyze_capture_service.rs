use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;

use crate::{
    application::{
        dto::analysis::{AnalyzeCaptureRequest, AnalyzeCaptureResponse},
        errors::ApplicationError,
        ports::{
            provider::ProviderRegistry,
            repositories::{ConversationRepository, HistoryRepository},
        },
        services::prompt_builder::PromptBuilder,
    },
    domain::{
        entities::{
            configured_provider::ProviderChannel,
            conversation_message::{ConversationMessage, MessageRole},
            interaction_session::{AnalysisSourceKind, InteractionSession},
        },
        value_objects::identifiers::{MessageId, SessionId},
    },
};

use super::analyze_capture::AnalyzeCapture;

static IDENTIFIER_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub struct AnalyzeCaptureService {
    registry: Arc<dyn ProviderRegistry>,
    history_repository: Arc<dyn HistoryRepository>,
    conversation_repository: Arc<dyn ConversationRepository>,
    prompt_builder: PromptBuilder,
}

impl AnalyzeCaptureService {
    pub fn new(
        registry: Arc<dyn ProviderRegistry>,
        history_repository: Arc<dyn HistoryRepository>,
        conversation_repository: Arc<dyn ConversationRepository>,
    ) -> Self {
        Self {
            registry,
            history_repository,
            conversation_repository,
            prompt_builder: PromptBuilder::new(),
        }
    }
}

#[async_trait]
impl AnalyzeCapture for AnalyzeCaptureService {
    async fn execute(
        &self,
        request: AnalyzeCaptureRequest,
    ) -> Result<AnalyzeCaptureResponse, ApplicationError> {
        self.registry
            .resolve_model(request.provider_kind, &request.model_key)?;

        let prompt = self.prompt_builder.compose_analysis_prompt(&request);
        let provider_request = AnalyzeCaptureRequest {
            base_prompt: prompt.clone(),
            ..request.clone()
        };

        let response = match request.provider_kind.channel() {
            ProviderChannel::Http => {
                self.registry
                    .resolve_ai(request.provider_kind)?
                    .analyze(&provider_request)
                    .await?
            }
            ProviderChannel::Cli => {
                let cli_provider = self.registry.resolve_cli(request.provider_kind)?;
                cli_provider.probe_availability().await?;
                cli_provider.execute(&provider_request).await?
            }
        };

        let session_id = request
            .conversation_context
            .first()
            .map(|message| message.session_id.clone())
            .unwrap_or_else(next_session_id);

        let session = InteractionSession {
            id: session_id.clone(),
            provider_kind: request.provider_kind,
            model_key: request.model_key.clone(),
            source_kind: AnalysisSourceKind::ScreenCapture,
            ocr_text: Some(request.ocr_text.clone()),
            user_notes: request.user_notes.clone(),
            request_prompt: Some(prompt.clone()),
            response_text: Some(response.answer.clone()),
        };

        self.history_repository.save_session(&session).await?;

        for message in &request.conversation_context {
            self.conversation_repository.append_message(message).await?;
        }

        let user_turn = ConversationMessage::new(
            next_message_id(),
            session_id.clone(),
            MessageRole::User,
            self.prompt_builder.compose_user_turn(&request),
        )?;
        let assistant_turn = ConversationMessage::new(
            next_message_id(),
            session_id,
            MessageRole::Assistant,
            response.answer.clone(),
        )?;

        self.conversation_repository
            .append_message(&user_turn)
            .await?;
        self.conversation_repository
            .append_message(&assistant_turn)
            .await?;

        Ok(AnalyzeCaptureResponse {
            prompt,
            session,
            response,
        })
    }
}

fn next_session_id() -> SessionId {
    SessionId::new(format!("session-{}", next_identifier_value()))
        .expect("generated session ids should be valid")
}

fn next_message_id() -> MessageId {
    MessageId::new(format!("message-{}", next_identifier_value()))
        .expect("generated message ids should be valid")
}

pub(crate) fn next_identifier_value() -> String {
    let sequence = IDENTIFIER_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_millis();

    format!("{millis}-{sequence}")
}
