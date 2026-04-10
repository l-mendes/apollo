use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use apollo_desktop::{
    application::{
        dto::analysis::{AnalyzeCaptureRequest, NormalizedResponse},
        errors::{ApplicationError, ApplicationErrorKind},
        ports::provider::AiProvider,
    },
    domain::{entities::configured_provider::ProviderKind, value_objects::model_key::ModelKey},
    infrastructure::providers::http::{
        adapters::{AnthropicProvider, OllamaProvider, OpenAiProvider},
        transport::{HttpRequest, HttpResponse, HttpTransport},
    },
    infrastructure::{
        provider_catalog::ManualProviderModelCatalog,
        providers::runtime_profiles::HttpProviderProfile,
    },
};
use serde_json::json;

#[derive(Clone, Default)]
struct FakeTransport {
    responses: Arc<Mutex<VecDeque<Result<HttpResponse, ApplicationError>>>>,
    requests: Arc<Mutex<Vec<HttpRequest>>>,
}

impl FakeTransport {
    fn with_response(response: Result<HttpResponse, ApplicationError>) -> Self {
        let mut queue = VecDeque::new();
        queue.push_back(response);

        Self {
            responses: Arc::new(Mutex::new(queue)),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn take_requests(&self) -> Vec<HttpRequest> {
        self.requests
            .lock()
            .expect("request mutex should lock")
            .clone()
    }
}

#[async_trait::async_trait]
impl HttpTransport for FakeTransport {
    async fn post_json(&self, request: HttpRequest) -> Result<HttpResponse, ApplicationError> {
        self.requests
            .lock()
            .expect("request mutex should lock")
            .push(request);

        self.responses
            .lock()
            .expect("response mutex should lock")
            .pop_front()
            .expect("fake response should exist")
    }
}

fn request(provider_kind: ProviderKind, model_key: &str) -> AnalyzeCaptureRequest {
    AnalyzeCaptureRequest {
        provider_kind,
        model_key: ModelKey::new(model_key).expect("model key should be valid"),
        base_prompt: "Explain this idiom with nuance.".to_string(),
        ocr_text: "She made up her mind.".to_string(),
        user_notes: Some("Use two examples.".to_string()),
        conversation_context: Vec::new(),
    }
}

#[test]
fn openai_adapter_normalizes_responses_api_output() {
    tauri::async_runtime::block_on(async {
        let transport = FakeTransport::with_response(Ok(HttpResponse {
            status: 200,
            body: json!({
                "id": "resp_123",
                "output_text": "It means she finally decided."
            }),
        }));
        let provider = OpenAiProvider::with_profile(
            Arc::new(transport.clone()),
            Arc::new(ManualProviderModelCatalog::new()),
            HttpProviderProfile {
                kind: ProviderKind::OpenAi,
                endpoint: "https://api.openai.com/v1/responses".to_string(),
                api_key: Some("test-openai-key".to_string()),
                credentials_required: true,
                auth_header: Some("Authorization".to_string()),
                auth_prefix: Some("Bearer ".to_string()),
                extra_headers: vec![("Content-Type".to_string(), "application/json".to_string())],
                timeout: std::time::Duration::from_secs(30),
            },
        );

        let response = provider
            .analyze(&request(ProviderKind::OpenAi, "gpt-4.1-mini"))
            .await
            .expect("openai response should normalize");

        assert_eq!(
            response,
            NormalizedResponse {
                provider_kind: ProviderKind::OpenAi,
                model_key: ModelKey::new("gpt-4.1-mini").expect("model key should be valid"),
                answer: "It means she finally decided.".to_string(),
                raw_output:
                    "{\"id\":\"resp_123\",\"output_text\":\"It means she finally decided.\"}"
                        .to_string(),
            }
        );

        let requests = transport.take_requests();
        assert_eq!(requests.len(), 1);
        assert!(requests[0].url.contains("/v1/responses"));
        assert_eq!(
            requests[0]
                .headers
                .iter()
                .find(|(key, _)| key == "Authorization")
                .map(|(_, value)| value.as_str()),
            Some("Bearer test-openai-key")
        );
    });
}

#[test]
fn anthropic_adapter_maps_authentication_failures() {
    tauri::async_runtime::block_on(async {
        let transport = FakeTransport::with_response(Ok(HttpResponse {
            status: 401,
            body: json!({
                "error": {
                    "message": "invalid x-api-key",
                    "type": "authentication_error"
                }
            }),
        }));
        let provider = AnthropicProvider::with_profile(
            Arc::new(transport),
            Arc::new(ManualProviderModelCatalog::new()),
            HttpProviderProfile {
                kind: ProviderKind::Anthropic,
                endpoint: "https://api.anthropic.com/v1/messages".to_string(),
                api_key: Some("test-anthropic-key".to_string()),
                credentials_required: true,
                auth_header: Some("x-api-key".to_string()),
                auth_prefix: None,
                extra_headers: vec![("anthropic-version".to_string(), "2023-06-01".to_string())],
                timeout: std::time::Duration::from_secs(30),
            },
        );

        let error = provider
            .analyze(&request(ProviderKind::Anthropic, "claude-3-7-sonnet"))
            .await
            .expect_err("anthropic authentication errors should be explicit");

        assert_eq!(error.kind, ApplicationErrorKind::Authentication);
        assert!(error.message.contains("invalid x-api-key"));
    });
}

#[test]
fn ollama_adapter_normalizes_generate_response() {
    tauri::async_runtime::block_on(async {
        let transport = FakeTransport::with_response(Ok(HttpResponse {
            status: 200,
            body: json!({
                "model": "llama3.2",
                "response": "It means she decided after thinking about it.",
                "done": true
            }),
        }));
        let provider = OllamaProvider::new_cloud(
            Arc::new(transport),
            Arc::new(ManualProviderModelCatalog::new()),
            "https://ollama.example.com/api/generate",
        );

        let response = provider
            .analyze(&request(ProviderKind::OllamaCloud, "llama3.2"))
            .await
            .expect("ollama response should normalize");

        assert_eq!(
            response.answer,
            "It means she decided after thinking about it."
        );
        assert_eq!(response.provider_kind, ProviderKind::OllamaCloud);
    });
}

#[test]
fn provider_transport_timeouts_surface_as_timeout_errors() {
    tauri::async_runtime::block_on(async {
        let transport = FakeTransport::with_response(Err(ApplicationError::new(
            ApplicationErrorKind::Timeout,
            "request timed out",
        )));
        let provider = OpenAiProvider::with_profile(
            Arc::new(transport),
            Arc::new(ManualProviderModelCatalog::new()),
            HttpProviderProfile {
                kind: ProviderKind::OpenAi,
                endpoint: "https://api.openai.com/v1/responses".to_string(),
                api_key: Some("test-openai-key".to_string()),
                credentials_required: true,
                auth_header: Some("Authorization".to_string()),
                auth_prefix: Some("Bearer ".to_string()),
                extra_headers: vec![("Content-Type".to_string(), "application/json".to_string())],
                timeout: std::time::Duration::from_secs(30),
            },
        );

        let error = provider
            .analyze(&request(ProviderKind::OpenAi, "gpt-4.1-mini"))
            .await
            .expect_err("timeouts should propagate");

        assert_eq!(error.kind, ApplicationErrorKind::Timeout);
    });
}
