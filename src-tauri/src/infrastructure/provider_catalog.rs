use async_trait::async_trait;
use serde::Deserialize;

use crate::{
    application::{errors::ApplicationError, ports::repositories::ProviderModelCatalog},
    domain::{
        entities::{
            configured_provider::{ProviderChannel, ProviderKind},
            provider_model::ProviderModel,
        },
        value_objects::model_key::ModelKey,
    },
};

#[derive(Debug, Default)]
pub struct ManualProviderModelCatalog {
    models: Vec<ProviderModel>,
}

impl ManualProviderModelCatalog {
    pub fn new() -> Self {
        Self {
            models: load_models(),
        }
    }

    pub fn all_models(&self) -> &[ProviderModel] {
        &self.models
    }
}

#[async_trait]
impl ProviderModelCatalog for ManualProviderModelCatalog {
    async fn list_by_provider(
        &self,
        provider_kind: ProviderKind,
    ) -> Result<Vec<ProviderModel>, ApplicationError> {
        Ok(self
            .models
            .iter()
            .filter(|model| model.provider_kind == provider_kind)
            .cloned()
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct ProviderModelConfig {
    provider_kind: String,
    channel: String,
    model_key: String,
    display_name: String,
    manually_managed: bool,
    is_default: bool,
}

fn load_models() -> Vec<ProviderModel> {
    let configs: Vec<ProviderModelConfig> =
        serde_json::from_str(include_str!("../../resources/provider-models.json"))
            .expect("provider model catalog JSON should be valid");

    configs
        .into_iter()
        .map(|config| ProviderModel {
            provider_kind: config
                .provider_kind
                .parse::<ProviderKind>()
                .expect("provider kind should be valid"),
            channel: match config.channel.as_str() {
                "http" => ProviderChannel::Http,
                "cli" => ProviderChannel::Cli,
                _ => panic!("provider channel should be valid"),
            },
            model_key: ModelKey::new(config.model_key).expect("manual model key should be valid"),
            display_name: config.display_name,
            manually_managed: config.manually_managed,
            is_default: config.is_default,
        })
        .collect()
}
