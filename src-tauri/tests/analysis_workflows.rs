use std::sync::Arc;

use apollo_desktop::{
    application::{
        dto::analysis::{
            AnalyzeCaptureRequest, ContinueConversationRequest, ListProviderModelsRequest,
            NormalizedResponse,
        },
        errors::{ApplicationError, ApplicationErrorKind},
        ports::provider::{AiProvider, CliProviderExecutor},
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
            configured_provider::ProviderKind,
            conversation_message::{ConversationMessage, MessageRole},
            provider_model::ProviderModel,
        },
        value_objects::{
            identifiers::{MessageId, SessionId},
            model_key::ModelKey,
        },
    },
    infrastructure::{
        persistence::SqliteAppRepository, provider_catalog::ManualProviderModelCatalog,
        provider_registry::InMemoryProviderRegistry,
    },
};

struct FakeAiProvider {
    kind: ProviderKind,
}

#[async_trait::async_trait]
impl AiProvider for FakeAiProvider {
    fn kind(&self) -> ProviderKind {
        self.kind
    }

    async fn list_models(&self) -> Result<Vec<ProviderModel>, ApplicationError> {
        Ok(vec![ProviderModel {
            provider_kind: self.kind,
            channel: self.kind.channel(),
            model_key: ModelKey::new("fake-model").expect("model key should be valid"),
            display_name: "Fake Model".to_string(),
            manually_managed: true,
            is_default: true,
        }])
    }

    async fn analyze(
        &self,
        request: &AnalyzeCaptureRequest,
    ) -> Result<NormalizedResponse, ApplicationError> {
        Ok(NormalizedResponse {
            provider_kind: request.provider_kind,
            model_key: request.model_key.clone(),
            answer: format!("Explanation for: {}", request.ocr_text),
            raw_output: format!("RAW::{}", request.ocr_text),
        })
    }
}

struct MissingCliProvider {
    kind: ProviderKind,
}

#[async_trait::async_trait]
impl CliProviderExecutor for MissingCliProvider {
    fn kind(&self) -> ProviderKind {
        self.kind
    }

    async fn probe_availability(&self) -> Result<(), ApplicationError> {
        Err(ApplicationError::new(
            ApplicationErrorKind::MissingCli,
            "CLI provider is not installed",
        ))
    }

    async fn execute(
        &self,
        _request: &AnalyzeCaptureRequest,
    ) -> Result<NormalizedResponse, ApplicationError> {
        Err(ApplicationError::new(
            ApplicationErrorKind::MissingCli,
            "CLI provider is not installed",
        ))
    }
}

fn message(
    id: &str,
    session_id: &SessionId,
    role: MessageRole,
    content: &str,
) -> ConversationMessage {
    ConversationMessage::new(
        MessageId::new(id).expect("message id should be valid"),
        session_id.clone(),
        role,
        content,
    )
    .expect("message should be valid")
}

#[test]
fn manual_catalog_lists_manually_managed_models_per_provider() {
    tauri::async_runtime::block_on(async {
        let catalog = Arc::new(ManualProviderModelCatalog::new());
        let service = ListProviderModelsService::new(catalog.clone());

        let response = service
            .execute(ListProviderModelsRequest {
                provider_kind: ProviderKind::Anthropic,
            })
            .await
            .expect("model catalog should load");

        assert!(!response.models.is_empty());
        assert!(response.models.iter().all(|model| model.manually_managed));
        assert!(
            response
                .models
                .iter()
                .all(|model| model.provider_kind == ProviderKind::Anthropic)
        );
    });
}

#[test]
fn analyze_capture_service_composes_prompt_and_persists_session_history() {
    tauri::async_runtime::block_on(async {
        let repository = Arc::new(
            SqliteAppRepository::in_memory().expect("in-memory repository should initialize"),
        );
        let catalog = Arc::new(ManualProviderModelCatalog::new());
        let registry = Arc::new(
            InMemoryProviderRegistry::new(catalog.clone())
                .with_ai_provider(Arc::new(FakeAiProvider {
                    kind: ProviderKind::OpenAi,
                }))
                .with_cli_provider(Arc::new(MissingCliProvider {
                    kind: ProviderKind::ClaudeCli,
                })),
        );
        let service = AnalyzeCaptureService::new(registry, repository.clone(), repository.clone());
        let session_id = SessionId::new("session-100").expect("session id should be valid");
        let context = vec![
            message(
                "message-1",
                &session_id,
                MessageRole::User,
                "What does this sentence mean?",
            ),
            message(
                "message-2",
                &session_id,
                MessageRole::Assistant,
                "It expresses anticipation about a future event.",
            ),
        ];

        let response = service
            .execute(AnalyzeCaptureRequest {
                provider_kind: ProviderKind::OpenAi,
                model_key: ModelKey::new("gpt-4.1-mini").expect("model key should be valid"),
                base_prompt: "Act as a language tutor.".to_string(),
                ocr_text: "I have been looking forward to this trip for ages.".to_string(),
                user_notes: Some("Explain the nuance of looking forward to.".to_string()),
                conversation_context: context.clone(),
            })
            .await
            .expect("analysis should succeed");

        let conversation = apollo_desktop::application::ports::repositories::ConversationRepository::load_by_session(
            repository.as_ref(),
            &response.session.id,
        )
            .await
            .expect("conversation should load");

        assert!(response.prompt.contains("Act as a language tutor."));
        assert!(response.prompt.contains("I have been looking forward"));
        assert!(
            response
                .prompt
                .contains("Explain the nuance of looking forward to.")
        );
        assert!(
            response
                .prompt
                .contains("It expresses anticipation about a future event.")
        );
        assert_eq!(
            response.response.answer,
            "Explanation for: I have been looking forward to this trip for ages."
        );
        assert_eq!(conversation.len(), 4);
        assert!(
            conversation
                .iter()
                .any(|entry| entry.role == MessageRole::User)
        );
        assert!(
            conversation
                .iter()
                .any(|entry| entry.role == MessageRole::Assistant)
        );
    });
}

#[test]
fn continue_conversation_service_appends_follow_up_and_response() {
    tauri::async_runtime::block_on(async {
        let repository = Arc::new(
            SqliteAppRepository::in_memory().expect("in-memory repository should initialize"),
        );
        let catalog = Arc::new(ManualProviderModelCatalog::new());
        let registry = Arc::new(
            InMemoryProviderRegistry::new(catalog).with_ai_provider(Arc::new(FakeAiProvider {
                kind: ProviderKind::Anthropic,
            })),
        );
        let service =
            ContinueConversationService::new(registry, repository.clone(), repository.clone());
        let session_id = SessionId::new("session-continue").expect("session id should be valid");
        let existing_messages = vec![
            message(
                "message-a",
                &session_id,
                MessageRole::User,
                "Explain made up her mind.",
            ),
            message(
                "message-b",
                &session_id,
                MessageRole::Assistant,
                "It means she decided something.",
            ),
        ];

        apollo_desktop::application::ports::repositories::HistoryRepository::save_session(
            repository.as_ref(),
            &apollo_desktop::domain::entities::interaction_session::InteractionSession {
                id: session_id.clone(),
                provider_kind: ProviderKind::Anthropic,
                model_key: ModelKey::new("claude-3-7-sonnet")
                    .expect("model key should be valid"),
                source_kind: apollo_desktop::domain::entities::interaction_session::AnalysisSourceKind::ManualText,
                ocr_text: None,
                user_notes: Some("Explain made up her mind.".to_string()),
                request_prompt: Some("Explain made up her mind.".to_string()),
                response_text: Some("It means she decided something.".to_string()),
            },
        )
        .await
        .expect("session should persist");

        for entry in &existing_messages {
            apollo_desktop::application::ports::repositories::ConversationRepository::append_message(
                repository.as_ref(),
                entry,
            )
                .await
                .expect("existing messages should persist");
        }

        let response = service
            .execute(ContinueConversationRequest {
                session_id: session_id.clone(),
                provider_kind: ProviderKind::Anthropic,
                model_key: ModelKey::new("claude-3-7-sonnet").expect("model key should be valid"),
                prompt: "Give me two more casual examples.".to_string(),
                existing_messages: existing_messages.clone(),
            })
            .await
            .expect("conversation should continue");

        let conversation = apollo_desktop::application::ports::repositories::ConversationRepository::load_by_session(
            repository.as_ref(),
            &session_id,
        )
            .await
            .expect("conversation should load");

        assert_eq!(response.session_id, session_id);
        assert_eq!(response.appended_messages.len(), 2);
        assert!(conversation.len() >= 4);
        assert!(
            conversation
                .iter()
                .any(|entry| entry.content == "Give me two more casual examples.")
        );
        assert!(
            conversation
                .iter()
                .any(|entry| entry.role == MessageRole::Assistant)
        );
    });
}
