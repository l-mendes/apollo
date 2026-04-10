use std::env;
use std::fs;

use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use screenshots::Screen;
use uuid::Uuid;

use crate::{
    application::{
        errors::{ApplicationError, ApplicationErrorKind},
        ports::capture::ScreenCapturePort,
    },
    domain::{
        entities::capture_record::{CaptureRecord, OcrStatus},
        value_objects::identifiers::CaptureId,
    },
};

#[derive(Debug, Default)]
pub struct ScreenshotCapturePort;

#[async_trait]
impl ScreenCapturePort for ScreenshotCapturePort {
    async fn capture_area(&self) -> Result<CaptureRecord, ApplicationError> {
        let (path, width, height) = tauri::async_runtime::spawn_blocking(capture_primary_screen)
            .await
            .map_err(|e| ApplicationError::new(
                ApplicationErrorKind::Unavailable,
                format!("capture task panicked: {e}"),
            ))??;

        let id = CaptureId::new(Uuid::new_v4().to_string())
            .expect("uuid is always a valid capture id");

        Ok(CaptureRecord {
            id,
            session_id: None,
            image_path: path,
            width,
            height,
            ocr_status: OcrStatus::Pending,
        })
    }
}

/// Synchronous screen capture — safe to call from `spawn_blocking` or shortcut handler threads.
pub fn capture_primary_screen() -> Result<(String, u32, u32), ApplicationError> {
    let screens = Screen::all().map_err(|e| ApplicationError::new(
        ApplicationErrorKind::Unavailable,
        format!("screen enumeration failed: {e}"),
    ))?;

    let screen = screens
        .into_iter()
        .find(|s| s.display_info.is_primary)
        .ok_or_else(|| ApplicationError::new(
            ApplicationErrorKind::Unavailable,
            "no primary screen found",
        ))?;

    let width = screen.display_info.width;
    let height = screen.display_info.height;

    let image = screen.capture().map_err(|e| ApplicationError::new(
        ApplicationErrorKind::Unavailable,
        format!("screen capture failed: {e}"),
    ))?;

    let filename = format!("apollo_{}.png", Uuid::new_v4());
    let path = env::temp_dir().join(&filename);

    image.save(&path).map_err(|e| ApplicationError::new(
        ApplicationErrorKind::Unavailable,
        format!("failed to save capture: {e}"),
    ))?;

    Ok((path.to_string_lossy().into_owned(), width, height))
}

/// Capture a rectangular region of the screen, given coordinates expressed in
/// the virtual desktop space (logical pixels). Returns the on-disk PNG path,
/// the captured width/height, and a base64-encoded PNG ready to render in a
/// webview as `data:image/png;base64,...`.
pub fn capture_screen_region_sync(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(String, u32, u32, String), ApplicationError> {
    if width == 0 || height == 0 {
        return Err(ApplicationError::new(
            ApplicationErrorKind::Validation,
            "selection area must have a positive size",
        ));
    }

    // Pick the screen that contains the upper-left corner of the selection.
    // Falls back to the primary screen if `from_point` does not match.
    let screen = match Screen::from_point(x, y) {
        Ok(screen) => screen,
        Err(_) => Screen::all()
            .map_err(|e| ApplicationError::new(
                ApplicationErrorKind::Unavailable,
                format!("could not enumerate screens: {e}"),
            ))?
            .into_iter()
            .find(|s| s.display_info.is_primary)
            .ok_or_else(|| ApplicationError::new(
                ApplicationErrorKind::Unavailable,
                "no primary screen found",
            ))?,
    };

    // `capture_area` expects coordinates relative to the chosen screen origin.
    let local_x = x - screen.display_info.x;
    let local_y = y - screen.display_info.y;

    let image = screen
        .capture_area(local_x, local_y, width, height)
        .map_err(|e| ApplicationError::new(
            ApplicationErrorKind::Unavailable,
            format!("region capture failed: {e}"),
        ))?;

    let captured_width = image.width();
    let captured_height = image.height();

    let filename = format!("apollo_region_{}.png", Uuid::new_v4());
    let path = env::temp_dir().join(&filename);

    image.save(&path).map_err(|e| ApplicationError::new(
        ApplicationErrorKind::Unavailable,
        format!("failed to save region capture: {e}"),
    ))?;

    let bytes = fs::read(&path).map_err(|e| ApplicationError::new(
        ApplicationErrorKind::Unavailable,
        format!("failed to read region capture: {e}"),
    ))?;
    let base64 = general_purpose::STANDARD.encode(&bytes);
    let data_url = format!("data:image/png;base64,{base64}");

    Ok((
        path.to_string_lossy().into_owned(),
        captured_width,
        captured_height,
        data_url,
    ))
}

/// Synchronous OCR extraction — safe to call from shortcut handler threads.
pub fn extract_text_sync(image_path: &str, lang: &str) -> Result<String, ApplicationError> {
    let output = std::process::Command::new("tesseract")
        .arg(image_path)
        .arg("stdout")
        .arg("-l")
        .arg(lang)
        .output()
        .map_err(|e| ApplicationError::new(
            ApplicationErrorKind::Unavailable,
            format!("tesseract execution failed: {e}"),
        ))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(ApplicationError::new(
            ApplicationErrorKind::Unavailable,
            if stderr.is_empty() { "OCR failed".to_string() } else { stderr },
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
