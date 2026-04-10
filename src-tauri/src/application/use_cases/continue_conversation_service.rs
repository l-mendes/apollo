use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    application::{
        dto::analysis::{
            AnalyzeCaptureRequest, ContinueConversationRequest, ContinueConversationResponse,
        },
        errors::ApplicationError,
        ports::{
            provider::ProviderRegistry,
            repositories::{ConversationRepository, HistoryRepository},
        },
        services::prompt_builder::PromptBuilder,
        use_cases::analyze_capture_service::next_identifier_value,
    },
    domain::{
        entities::{
            configured_provider::ProviderChannel,
            conversation_message::{ConversationMessage, MessageRole},
            interaction_session::{AnalysisSourceKind, InteractionSession},
        },
        value_objects::identifiers::MessageId,
    },
};

use super::continue_conversation::ContinueConversation;

pub struct ContinueConversationService {
    registry: Arc<dyn ProviderRegistry>,
    history_repository: Arc<dyn HistoryRepository>,
    conversation_repository: Arc<dyn ConversationRepository>,
    prompt_builder: PromptBuilder,
}

impl ContinueConversationService {
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
impl ContinueConversation for ContinueConversationService {
    async fn execute(
        &self,
        request: ContinueConversationRequest,
    ) -> Result<ContinueConversationResponse, ApplicationError> {
        self.registry
            .resolve_model(request.provider_kind, &request.model_key)?;

        let prompt = self
            .prompt_builder
            .compose_follow_up_prompt(&request.existing_messages, &request.prompt);
        let provider_request = AnalyzeCaptureRequest {
            provider_kind: request.provider_kind,
            model_key: request.model_key.clone(),
            base_prompt: prompt.clone(),
            ocr_text: String::new(),
            user_notes: Some(request.prompt.clone()),
            conversation_context: request.existing_messages.clone(),
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

        let session = InteractionSession {
            id: request.session_id.clone(),
            provider_kind: request.provider_kind,
            model_key: request.model_key.clone(),
            source_kind: AnalysisSourceKind::ManualText,
            ocr_text: None,
            user_notes: Some(request.prompt.clone()),
            request_prompt: Some(prompt),
            response_text: Some(response.answer.clone()),
        };
        self.history_repository.save_session(&session).await?;

        let user_turn = ConversationMessage::new(
            MessageId::new(format!("message-{}", next_identifier_value()))
                .expect("generated message ids should be valid"),
            request.session_id.clone(),
            MessageRole::User,
            request.prompt.clone(),
        )?;
        let assistant_turn = ConversationMessage::new(
            MessageId::new(format!("message-{}", next_identifier_value()))
                .expect("generated message ids should be valid"),
            request.session_id.clone(),
            MessageRole::Assistant,
            response.answer.clone(),
        )?;

        self.conversation_repository
            .append_message(&user_turn)
            .await?;
        self.conversation_repository
            .append_message(&assistant_turn)
            .await?;

        Ok(ContinueConversationResponse {
            session_id: request.session_id,
            response,
            appended_messages: vec![user_turn, assistant_turn],
        })
    }
}
