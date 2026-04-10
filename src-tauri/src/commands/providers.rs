use tauri::State;

use crate::{
    application::{
        dto::analysis::{ListProviderModelsRequest, ListProviderModelsResponse},
        errors::ApplicationError,
        state::AppState,
        use_cases::list_provider_models::ListProviderModels,
    },
    domain::entities::provider_model::ProviderModel,
};

#[tauri::command]
pub async fn list_provider_models(
    state: State<'_, AppState>,
    request: ListProviderModelsRequest,
) -> Result<ListProviderModelsResponse, ApplicationError> {
    state.list_provider_models().execute(request).await
}

#[tauri::command]
pub async fn list_provider_models_for(
    state: State<'_, AppState>,
    provider_kind: crate::domain::entities::configured_provider::ProviderKind,
) -> Result<Vec<ProviderModel>, ApplicationError> {
    crate::application::ports::repositories::ProviderModelCatalog::list_by_provider(
        state.model_catalog().as_ref(),
        provider_kind,
    )
    .await
}
