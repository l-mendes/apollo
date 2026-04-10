use async_trait::async_trait;

use crate::application::{
    dto::analysis::{ListProviderModelsRequest, ListProviderModelsResponse},
    errors::ApplicationError,
};

#[async_trait]
pub trait ListProviderModels: Send + Sync {
    async fn execute(
        &self,
        request: ListProviderModelsRequest,
    ) -> Result<ListProviderModelsResponse, ApplicationError>;
}
