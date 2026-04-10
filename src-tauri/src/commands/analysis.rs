use tauri::State;

use crate::{
    application::{
        dto::{
            analysis::{
                AnalyzeCaptureRequest, AnalyzeCaptureResponse, ContinueConversationRequest,
                ContinueConversationResponse,
            },
            ocr::OcrTextExtraction,
        },
        errors::ApplicationError,
        state::AppState,
        use_cases::{analyze_capture::AnalyzeCapture, continue_conversation::ContinueConversation},
    },
    domain::entities::capture_record::CaptureRecord,
};

#[tauri::command]
pub async fn analyze_capture(
    state: State<'_, AppState>,
    request: AnalyzeCaptureRequest,
) -> Result<AnalyzeCaptureResponse, ApplicationError> {
    state.analyze_capture().execute(request).await
}

#[tauri::command]
pub async fn continue_conversation(
    state: State<'_, AppState>,
    request: ContinueConversationRequest,
) -> Result<ContinueConversationResponse, ApplicationError> {
    state.continue_conversation().execute(request).await
}

#[tauri::command]
pub async fn extract_text_from_capture(
    state: State<'_, AppState>,
    capture: CaptureRecord,
) -> Result<OcrTextExtraction, ApplicationError> {
    crate::application::ports::capture::OcrEngine::extract_text(
        state.ocr_engine().as_ref(),
        &capture,
    )
    .await
}
