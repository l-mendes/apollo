use serde::{Deserialize, Serialize};

use crate::domain::{
    entities::{
        configured_provider::ProviderKind, conversation_message::ConversationMessage,
        interaction_session::InteractionSession, provider_model::ProviderModel,
    },
    value_objects::{identifiers::SessionId, model_key::ModelKey},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalyzeCaptureRequest {
    pub provider_kind: ProviderKind,
    pub model_key: ModelKey,
    pub base_prompt: String,
    pub ocr_text: String,
    pub user_notes: Option<String>,
    pub conversation_context: Vec<ConversationMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedResponse {
    pub provider_kind: ProviderKind,
    pub model_key: ModelKey,
    pub answer: String,
    pub raw_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalyzeCaptureResponse {
    pub prompt: String,
    pub session: InteractionSession,
    pub response: NormalizedResponse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinueConversationRequest {
    pub session_id: SessionId,
    pub provider_kind: ProviderKind,
    pub model_key: ModelKey,
    pub prompt: String,
    pub existing_messages: Vec<ConversationMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinueConversationResponse {
    pub session_id: SessionId,
    pub response: NormalizedResponse,
    pub appended_messages: Vec<ConversationMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListProviderModelsRequest {
    pub provider_kind: ProviderKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListProviderModelsResponse {
    pub provider_kind: ProviderKind,
    pub models: Vec<ProviderModel>,
}

#[cfg(test)]
mod tests {
    use crate::domain::{
        entities::{
            configured_provider::{ProviderChannel, ProviderKind},
            provider_model::ProviderModel,
        },
        value_objects::model_key::ModelKey,
    };

    use super::ListProviderModelsResponse;

    #[test]
    fn serializes_provider_model_response_shape() {
        let payload = ListProviderModelsResponse {
            provider_kind: ProviderKind::OpenAi,
            models: vec![ProviderModel {
                provider_kind: ProviderKind::OpenAi,
                channel: ProviderChannel::Http,
                model_key: ModelKey::new("gpt-4.1-mini").expect("model key should be valid"),
                display_name: "GPT-4.1 Mini".to_string(),
                manually_managed: true,
                is_default: true,
            }],
        };

        let json = serde_json::to_string(&payload).expect("payload should serialize");

        assert!(json.contains("GPT-4.1 Mini"));
        assert!(json.contains("OpenAi"));
    }
}
