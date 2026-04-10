use std::sync::Arc;

use async_trait::async_trait;

use crate::application::{
    dto::analysis::{ListProviderModelsRequest, ListProviderModelsResponse},
    errors::ApplicationError,
    ports::repositories::ProviderModelCatalog,
};

use super::list_provider_models::ListProviderModels;

pub struct ListProviderModelsService {
    catalog: Arc<dyn ProviderModelCatalog>,
}

impl ListProviderModelsService {
    pub fn new(catalog: Arc<dyn ProviderModelCatalog>) -> Self {
        Self { catalog }
    }
}

#[async_trait]
impl ListProviderModels for ListProviderModelsService {
    async fn execute(
        &self,
        request: ListProviderModelsRequest,
    ) -> Result<ListProviderModelsResponse, ApplicationError> {
        let models = self.catalog.list_by_provider(request.provider_kind).await?;

        Ok(ListProviderModelsResponse {
            provider_kind: request.provider_kind,
            models,
        })
    }
}
