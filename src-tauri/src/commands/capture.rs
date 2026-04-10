use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    application::{
        errors::{ApplicationError, ApplicationErrorKind},
        ports::capture::{OcrEngine, ScreenCapturePort},
        state::AppState,
    },
    domain::entities::shortcut_binding::ShortcutBinding,
    infrastructure::capture::{
        capture_screen_region_sync, CaptureRegionCoordinates, XCapCapturePort,
    },
};

/// Capture the primary screen and run OCR, returning the extracted text.
#[tauri::command]
pub async fn trigger_screen_capture(
    state: State<'_, AppState>,
) -> Result<String, ApplicationError> {
    let port = XCapCapturePort;
    let capture = port.capture_area().await?;

    let extraction = OcrEngine::extract_text(state.ocr_engine().as_ref(), &capture).await?;
    Ok(extraction.text)
}

#[derive(Debug, Clone, Deserialize)]
pub struct CaptureRegionRequest {
    pub logical_x: i32,
    pub logical_y: i32,
    pub logical_width: u32,
    pub logical_height: u32,
    pub physical_x: i32,
    pub physical_y: i32,
    pub physical_width: u32,
    pub physical_height: u32,
    pub monitor_logical_x: i32,
    pub monitor_logical_y: i32,
    pub monitor_logical_width: u32,
    pub monitor_logical_height: u32,
    pub monitor_physical_x: i32,
    pub monitor_physical_y: i32,
    pub monitor_physical_width: u32,
    pub monitor_physical_height: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CaptureRegionResponse {
    pub image_path: String,
    pub width: u32,
    pub height: u32,
    pub data_url: String,
}

/// Capture a user-selected screen rectangle and return the saved PNG path
/// alongside an inline `data:` URL ready to display in a webview.
#[tauri::command]
pub async fn capture_screen_region(
    request: CaptureRegionRequest,
) -> Result<CaptureRegionResponse, ApplicationError> {
    let coordinates = CaptureRegionCoordinates::from(request);

    let (image_path, captured_width, captured_height, data_url) =
        tauri::async_runtime::spawn_blocking(move || {
            capture_screen_region_sync(coordinates)
        })
        .await
        .map_err(|e| ApplicationError::new(
            ApplicationErrorKind::Unavailable,
            format!("region capture task panicked: {e}"),
        ))??;

    Ok(CaptureRegionResponse {
        image_path,
        width: captured_width,
        height: captured_height,
        data_url,
    })
}

#[derive(Debug, Clone, Deserialize)]
pub struct OcrFromImageRequest {
    pub image_path: String,
    pub ocr_language: String,
}

/// Run OCR over a previously captured image saved on disk.
#[tauri::command]
pub async fn run_ocr_on_image(
    request: OcrFromImageRequest,
) -> Result<String, ApplicationError> {
    let path = request.image_path;
    let lang = request.ocr_language;
    let text = tauri::async_runtime::spawn_blocking(move || {
        crate::infrastructure::capture::extract_text_sync(&path, &lang)
    })
    .await
    .map_err(|e| ApplicationError::new(
        ApplicationErrorKind::Unavailable,
        format!("ocr task panicked: {e}"),
    ))??;

    Ok(text)
}

impl From<CaptureRegionRequest> for CaptureRegionCoordinates {
    fn from(value: CaptureRegionRequest) -> Self {
        Self {
            logical_x: value.logical_x,
            logical_y: value.logical_y,
            logical_width: value.logical_width,
            logical_height: value.logical_height,
            physical_x: value.physical_x,
            physical_y: value.physical_y,
            physical_width: value.physical_width,
            physical_height: value.physical_height,
            monitor_logical_x: value.monitor_logical_x,
            monitor_logical_y: value.monitor_logical_y,
            monitor_logical_width: value.monitor_logical_width,
            monitor_logical_height: value.monitor_logical_height,
            monitor_physical_x: value.monitor_physical_x,
            monitor_physical_y: value.monitor_physical_y,
            monitor_physical_width: value.monitor_physical_width,
            monitor_physical_height: value.monitor_physical_height,
        }
    }
}

/// Register the provided shortcut bindings as global shortcuts, replacing any previous ones.
#[tauri::command]
pub async fn apply_global_shortcuts(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    shortcuts: Vec<ShortcutBinding>,
) -> Result<(), ApplicationError> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    app.global_shortcut()
        .unregister_all()
        .map_err(|e| ApplicationError::new(
            ApplicationErrorKind::InvalidConfiguration,
            format!("failed to unregister shortcuts: {e}"),
        ))?;

    for binding in shortcuts.iter().filter(|b| b.enabled) {
        if let Err(e) = app.global_shortcut().register(binding.accelerator.as_str()) {
            tracing::warn!(
                accelerator = binding.accelerator.as_str(),
                error = %e,
                "failed to register global shortcut"
            );
        }
    }

    state.apply_shortcuts(shortcuts);
    Ok(())
}
