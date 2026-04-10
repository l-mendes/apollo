use tauri::State;

use crate::{
    application::{errors::ApplicationError, state::AppState},
    domain::{
        entities::{
            conversation_message::ConversationMessage, interaction_session::InteractionSession,
        },
        value_objects::identifiers::SessionId,
    },
};

#[tauri::command]
pub async fn list_history(
    state: State<'_, AppState>,
) -> Result<Vec<InteractionSession>, ApplicationError> {
    crate::application::ports::repositories::HistoryRepository::list_sessions(
        state.repository().as_ref(),
    )
    .await
}

#[tauri::command]
pub async fn load_conversation_messages(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<ConversationMessage>, ApplicationError> {
    let session_id = SessionId::new(session_id)?;

    crate::application::ports::repositories::ConversationRepository::load_by_session(
        state.repository().as_ref(),
        &session_id,
    )
    .await
}

#[tauri::command]
pub async fn delete_history_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), ApplicationError> {
    let session_id = SessionId::new(session_id)?;

    crate::application::ports::repositories::HistoryRepository::delete_session(
        state.repository().as_ref(),
        &session_id,
    )
    .await
}

#[tauri::command]
pub async fn clear_history(state: State<'_, AppState>) -> Result<(), ApplicationError> {
    crate::application::ports::repositories::HistoryRepository::clear_sessions(
        state.repository().as_ref(),
    )
    .await
}
