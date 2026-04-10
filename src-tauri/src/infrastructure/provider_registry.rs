use std::{collections::HashMap, sync::Arc};

use crate::{
    application::{
        errors::{ApplicationError, ApplicationErrorKind},
        ports::provider::{AiProvider, CliProviderExecutor, ProviderRegistry},
    },
    domain::{
        entities::{configured_provider::ProviderKind, provider_model::ProviderModel},
        value_objects::model_key::ModelKey,
    },
    infrastructure::provider_catalog::ManualProviderModelCatalog,
};

pub struct InMemoryProviderRegistry {
    models: Vec<ProviderModel>,
    ai_providers: HashMap<ProviderKind, Arc<dyn AiProvider>>,
    cli_providers: HashMap<ProviderKind, Arc<dyn CliProviderExecutor>>,
}

impl InMemoryProviderRegistry {
    pub fn new(catalog: Arc<ManualProviderModelCatalog>) -> Self {
        Self {
            models: catalog.all_models().to_vec(),
            ai_providers: HashMap::new(),
            cli_providers: HashMap::new(),
        }
    }

    pub fn with_ai_provider(mut self, provider: Arc<dyn AiProvider>) -> Self {
        self.ai_providers.insert(provider.kind(), provider);
        self
    }

    pub fn with_cli_provider(mut self, provider: Arc<dyn CliProviderExecutor>) -> Self {
        self.cli_providers.insert(provider.kind(), provider);
        self
    }
}

impl ProviderRegistry for InMemoryProviderRegistry {
    fn resolve_ai(
        &self,
        provider_kind: ProviderKind,
    ) -> Result<Arc<dyn AiProvider>, ApplicationError> {
        self.ai_providers
            .get(&provider_kind)
            .cloned()
            .ok_or_else(|| {
                ApplicationError::new(
                    ApplicationErrorKind::Unavailable,
                    format!("no AI provider registered for {}", provider_kind.as_str()),
                )
            })
    }

    fn resolve_cli(
        &self,
        provider_kind: ProviderKind,
    ) -> Result<Arc<dyn CliProviderExecutor>, ApplicationError> {
        self.cli_providers
            .get(&provider_kind)
            .cloned()
            .ok_or_else(|| {
                ApplicationError::new(
                    ApplicationErrorKind::MissingCli,
                    format!("no CLI provider registered for {}", provider_kind.as_str()),
                )
            })
    }

    fn resolve_model(
        &self,
        provider_kind: ProviderKind,
        model_key: &ModelKey,
    ) -> Result<ProviderModel, ApplicationError> {
        self.models
            .iter()
            .find(|model| model.matches_selection(provider_kind, model_key))
            .cloned()
            .ok_or_else(|| {
                ApplicationError::new(
                    ApplicationErrorKind::NotFound,
                    format!(
                        "model {} is not available for {}",
                        model_key.as_str(),
                        provider_kind.as_str()
                    ),
                )
            })
    }
}
