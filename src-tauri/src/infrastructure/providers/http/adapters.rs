use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::{
    application::{
        dto::analysis::{AnalyzeCaptureRequest, NormalizedResponse},
        errors::{ApplicationError, ApplicationErrorKind},
        ports::provider::AiProvider,
        ports::repositories::ProviderModelCatalog,
    },
    domain::{
        entities::{configured_provider::ProviderKind, provider_model::ProviderModel},
        value_objects::model_key::ModelKey,
    },
    infrastructure::{
        provider_catalog::ManualProviderModelCatalog,
        providers::{
            http::transport::{HttpRequest, HttpResponse, HttpTransport},
            runtime_profiles::HttpProviderProfile,
        },
    },
};

pub struct OpenAiProvider {
    transport: Arc<dyn HttpTransport>,
    profile: HttpProviderProfile,
    catalog: Arc<ManualProviderModelCatalog>,
}

pub struct AnthropicProvider {
    transport: Arc<dyn HttpTransport>,
    profile: HttpProviderProfile,
    catalog: Arc<ManualProviderModelCatalog>,
}

pub struct OllamaProvider {
    transport: Arc<dyn HttpTransport>,
    profile: HttpProviderProfile,
    catalog: Arc<ManualProviderModelCatalog>,
}

impl OpenAiProvider {
    pub fn new(
        transport: Arc<dyn HttpTransport>,
        catalog: Arc<ManualProviderModelCatalog>,
    ) -> Self {
        Self::with_profile(transport, catalog, HttpProviderProfile::openai())
    }

    pub fn with_profile(
        transport: Arc<dyn HttpTransport>,
        catalog: Arc<ManualProviderModelCatalog>,
        profile: HttpProviderProfile,
    ) -> Self {
        Self {
            transport,
            profile,
            catalog,
        }
    }
}

impl AnthropicProvider {
    pub fn new(
        transport: Arc<dyn HttpTransport>,
        catalog: Arc<ManualProviderModelCatalog>,
    ) -> Self {
        Self::with_profile(transport, catalog, HttpProviderProfile::anthropic())
    }

    pub fn with_profile(
        transport: Arc<dyn HttpTransport>,
        catalog: Arc<ManualProviderModelCatalog>,
        profile: HttpProviderProfile,
    ) -> Self {
        Self {
            transport,
            profile,
            catalog,
        }
    }
}

impl OllamaProvider {
    pub fn new_cloud(
        transport: Arc<dyn HttpTransport>,
        catalog: Arc<ManualProviderModelCatalog>,
        endpoint: &str,
    ) -> Self {
        Self::with_profile(
            transport,
            catalog,
            HttpProviderProfile::ollama_cloud(endpoint.to_string()),
        )
    }

    pub fn new_local(
        transport: Arc<dyn HttpTransport>,
        catalog: Arc<ManualProviderModelCatalog>,
        endpoint: &str,
    ) -> Self {
        Self::with_profile(
            transport,
            catalog,
            HttpProviderProfile::ollama_local(endpoint.to_string()),
        )
    }

    pub fn with_profile(
        transport: Arc<dyn HttpTransport>,
        catalog: Arc<ManualProviderModelCatalog>,
        profile: HttpProviderProfile,
    ) -> Self {
        Self {
            transport,
            profile,
            catalog,
        }
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::OpenAi
    }

    async fn list_models(&self) -> Result<Vec<ProviderModel>, ApplicationError> {
        self.catalog.list_by_provider(self.kind()).await
    }

    async fn analyze(
        &self,
        request: &AnalyzeCaptureRequest,
    ) -> Result<NormalizedResponse, ApplicationError> {
        let http_request = HttpRequest {
            url: self.profile.endpoint.clone(),
            headers: self.profile.headers()?,
            body: json!({
                "model": request.model_key.as_str(),
                "reasoning": {
                    "effort": request.reasoning_effort.as_str()
                },
                "input": request.base_prompt,
            }),
            timeout: self.profile.timeout,
        };

        let response = self.transport.post_json(http_request).await?;
        let answer = response
            .body
            .get("output_text")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| extract_nested_text(&response.body, &["output", "content", "text"]))
            .ok_or_else(|| invalid_payload("OpenAI response did not include output_text"))?;

        normalize_http_response(self.kind(), request.model_key.clone(), response, answer)
    }
}

#[async_trait]
impl AiProvider for AnthropicProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Anthropic
    }

    async fn list_models(&self) -> Result<Vec<ProviderModel>, ApplicationError> {
        self.catalog.list_by_provider(self.kind()).await
    }

    async fn analyze(
        &self,
        request: &AnalyzeCaptureRequest,
    ) -> Result<NormalizedResponse, ApplicationError> {
        let http_request = HttpRequest {
            url: self.profile.endpoint.clone(),
            headers: self.profile.headers()?,
            body: json!({
                "model": request.model_key.as_str(),
                "max_tokens": 1024,
                "messages": [
                    {
                        "role": "user",
                        "content": request.base_prompt
                    }
                ]
            }),
            timeout: self.profile.timeout,
        };

        let response = self.transport.post_json(http_request).await?;
        if response.status == 401 || response.status == 403 {
            return Err(http_error(
                ApplicationErrorKind::Authentication,
                &response,
                "Anthropic authentication failed",
            ));
        }

        let answer = response
            .body
            .get("content")
            .and_then(Value::as_array)
            .and_then(|content| content.first())
            .and_then(|first| first.get("text"))
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(ToOwned::to_owned)
            .ok_or_else(|| invalid_payload("Anthropic response did not include content[0].text"))?;

        normalize_http_response(self.kind(), request.model_key.clone(), response, answer)
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    fn kind(&self) -> ProviderKind {
        self.profile.kind
    }

    async fn list_models(&self) -> Result<Vec<ProviderModel>, ApplicationError> {
        self.catalog.list_by_provider(self.kind()).await
    }

    async fn analyze(
        &self,
        request: &AnalyzeCaptureRequest,
    ) -> Result<NormalizedResponse, ApplicationError> {
        let http_request = HttpRequest {
            url: self.profile.endpoint.clone(),
            headers: self.profile.headers()?,
            body: json!({
                "model": request.model_key.as_str(),
                "prompt": request.base_prompt,
                "stream": false
            }),
            timeout: self.profile.timeout,
        };

        let response = self.transport.post_json(http_request).await?;
        if response.status >= 500 {
            return Err(http_error(
                ApplicationErrorKind::Unavailable,
                &response,
                "Ollama provider is unavailable",
            ));
        }

        let answer = response
            .body
            .get("response")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(ToOwned::to_owned)
            .ok_or_else(|| invalid_payload("Ollama response did not include response text"))?;

        normalize_http_response(self.kind(), request.model_key.clone(), response, answer)
    }
}

fn normalize_http_response(
    provider_kind: ProviderKind,
    model_key: ModelKey,
    response: HttpResponse,
    answer: String,
) -> Result<NormalizedResponse, ApplicationError> {
    if response.status == 401 || response.status == 403 {
        return Err(http_error(
            ApplicationErrorKind::Authentication,
            &response,
            "provider authentication failed",
        ));
    }

    if response.status == 408 {
        return Err(http_error(
            ApplicationErrorKind::Timeout,
            &response,
            "provider request timed out",
        ));
    }

    if response.status >= 500 {
        return Err(http_error(
            ApplicationErrorKind::Unavailable,
            &response,
            "provider is unavailable",
        ));
    }

    if response.status >= 400 {
        return Err(http_error(
            ApplicationErrorKind::InvalidConfiguration,
            &response,
            "provider request failed",
        ));
    }

    Ok(NormalizedResponse {
        provider_kind,
        model_key,
        answer,
        raw_output: serde_json::to_string(&response.body).map_err(|error| {
            ApplicationError::new(
                ApplicationErrorKind::Unknown,
                format!("failed to serialize provider response: {error}"),
            )
        })?,
    })
}

fn http_error(
    kind: ApplicationErrorKind,
    response: &HttpResponse,
    fallback_message: &str,
) -> ApplicationError {
    let message = response
        .body
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .or_else(|| response.body.get("message").and_then(Value::as_str))
        .unwrap_or(fallback_message);

    ApplicationError::new(kind, message)
}

fn invalid_payload(message: &str) -> ApplicationError {
    ApplicationError::new(ApplicationErrorKind::Unknown, message)
}

fn extract_nested_text(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;

    for key in path {
        current = if current.is_array() {
            current.as_array()?.first()?
        } else {
            current.get(*key)?
        };
    }

    current.as_str().map(ToOwned::to_owned)
}
