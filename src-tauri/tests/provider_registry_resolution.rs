use std::sync::Arc;

use apollo::{
    application::{
        dto::analysis::{AnalyzeCaptureRequest, NormalizedResponse},
        errors::ApplicationError,
        ports::provider::{AiProvider, CliProviderExecutor, ProviderRegistry},
    },
    domain::{entities::configured_provider::ProviderKind, value_objects::model_key::ModelKey},
    infrastructure::{
        provider_catalog::ManualProviderModelCatalog, provider_registry::InMemoryProviderRegistry,
    },
};

struct StubAiProvider {
    kind: ProviderKind,
}

#[async_trait::async_trait]
impl AiProvider for StubAiProvider {
    fn kind(&self) -> ProviderKind {
        self.kind
    }

    async fn list_models(
        &self,
    ) -> Result<
        Vec<apollo::domain::entities::provider_model::ProviderModel>,
        ApplicationError,
    > {
        Ok(Vec::new())
    }

    async fn analyze(
        &self,
        request: &AnalyzeCaptureRequest,
    ) -> Result<NormalizedResponse, ApplicationError> {
        Ok(NormalizedResponse {
            provider_kind: request.provider_kind,
            model_key: request.model_key.clone(),
            answer: "ok".to_string(),
            raw_output: "ok".to_string(),
        })
    }
}

struct StubCliProvider {
    kind: ProviderKind,
}

#[async_trait::async_trait]
impl CliProviderExecutor for StubCliProvider {
    fn kind(&self) -> ProviderKind {
        self.kind
    }

    async fn probe_availability(&self) -> Result<(), ApplicationError> {
        Ok(())
    }

    async fn execute(
        &self,
        request: &AnalyzeCaptureRequest,
    ) -> Result<NormalizedResponse, ApplicationError> {
        Ok(NormalizedResponse {
            provider_kind: request.provider_kind,
            model_key: request.model_key.clone(),
            answer: "ok".to_string(),
            raw_output: "ok".to_string(),
        })
    }
}

#[test]
fn registry_resolves_http_and_cli_providers_and_models() {
    let catalog = Arc::new(ManualProviderModelCatalog::new());
    let registry = InMemoryProviderRegistry::new(catalog)
        .with_ai_provider(Arc::new(StubAiProvider {
            kind: ProviderKind::OpenAi,
        }))
        .with_cli_provider(Arc::new(StubCliProvider {
            kind: ProviderKind::ClaudeCli,
        }));

    let ai = registry
        .resolve_ai(ProviderKind::OpenAi)
        .expect("http provider should resolve");
    let cli = registry
        .resolve_cli(ProviderKind::ClaudeCli)
        .expect("cli provider should resolve");
    let model = registry
        .resolve_model(
            ProviderKind::ClaudeCli,
            &ModelKey::new("claude-cli-default").expect("model key should be valid"),
        )
        .expect("manual cli model should resolve");

    assert_eq!(ai.kind(), ProviderKind::OpenAi);
    assert_eq!(cli.kind(), ProviderKind::ClaudeCli);
    assert_eq!(model.provider_kind, ProviderKind::ClaudeCli);
    assert_eq!(model.model_key.as_str(), "claude-cli-default");
}
