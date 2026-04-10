use tauri::State;

use crate::{
    application::{errors::ApplicationError, state::AppState},
    domain::entities::user_settings::UserSettings,
};

#[tauri::command]
pub async fn load_settings(state: State<'_, AppState>) -> Result<UserSettings, ApplicationError> {
    crate::application::ports::repositories::SettingsRepository::load(state.repository().as_ref())
        .await
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: UserSettings,
) -> Result<(), ApplicationError> {
    state
        .inner()
        .apply_ocr_language(settings.ocr_language.clone());
    crate::application::ports::repositories::SettingsRepository::save(
        state.repository().as_ref(),
        &settings,
    )
    .await
}
