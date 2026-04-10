use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    application::{
        dto::analysis::{AnalyzeCaptureRequest, NormalizedResponse},
        errors::ApplicationError,
    },
    domain::{
        entities::{configured_provider::ProviderKind, provider_model::ProviderModel},
        value_objects::model_key::ModelKey,
    },
};

#[async_trait]
pub trait AiProvider: Send + Sync {
    fn kind(&self) -> ProviderKind;

    async fn list_models(&self) -> Result<Vec<ProviderModel>, ApplicationError>;

    async fn analyze(
        &self,
        request: &AnalyzeCaptureRequest,
    ) -> Result<NormalizedResponse, ApplicationError>;
}

#[async_trait]
pub trait CliProviderExecutor: Send + Sync {
    fn kind(&self) -> ProviderKind;

    async fn probe_availability(&self) -> Result<(), ApplicationError>;

    async fn execute(
        &self,
        request: &AnalyzeCaptureRequest,
    ) -> Result<NormalizedResponse, ApplicationError>;
}

pub trait ProviderRegistry: Send + Sync {
    fn resolve_ai(
        &self,
        provider_kind: ProviderKind,
    ) -> Result<Arc<dyn AiProvider>, ApplicationError>;

    fn resolve_cli(
        &self,
        provider_kind: ProviderKind,
    ) -> Result<Arc<dyn CliProviderExecutor>, ApplicationError>;

    fn resolve_model(
        &self,
        provider_kind: ProviderKind,
        model_key: &ModelKey,
    ) -> Result<ProviderModel, ApplicationError>;
}
