use std::process::Command;

use async_trait::async_trait;

use crate::{
    application::{
        dto::ocr::OcrTextExtraction,
        errors::{ApplicationError, ApplicationErrorKind},
        ports::capture::OcrEngine,
    },
    domain::entities::capture_record::CaptureRecord,
};

pub struct TesseractOcrEngine {
    binary_name: String,
}

impl Default for TesseractOcrEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TesseractOcrEngine {
    pub fn new() -> Self {
        Self {
            binary_name: "tesseract".to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_binary(binary_name: impl Into<String>) -> Self {
        Self {
            binary_name: binary_name.into(),
        }
    }
}

#[async_trait]
impl OcrEngine for TesseractOcrEngine {
    async fn extract_text(
        &self,
        capture: &CaptureRecord,
    ) -> Result<OcrTextExtraction, ApplicationError> {
        let output = Command::new(&self.binary_name)
            .arg(&capture.image_path)
            .arg("stdout")
            .output()
            .map_err(|error| {
                ApplicationError::new(
                    ApplicationErrorKind::Unavailable,
                    format!("failed to execute Tesseract: {error}"),
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

            return Err(ApplicationError::new(
                ApplicationErrorKind::Unavailable,
                if stderr.is_empty() {
                    "Tesseract OCR failed".to_string()
                } else {
                    stderr
                },
            ));
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if text.is_empty() {
            return Err(ApplicationError::new(
                ApplicationErrorKind::Validation,
                "Tesseract did not detect any text in the capture",
            ));
        }

        Ok(OcrTextExtraction { text })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        application::{errors::ApplicationErrorKind, ports::capture::OcrEngine},
        domain::{
            entities::capture_record::{CaptureRecord, OcrStatus},
            value_objects::identifiers::CaptureId,
        },
    };

    use super::TesseractOcrEngine;

    #[test]
    fn reports_unavailable_engine_when_binary_is_missing() {
        tauri::async_runtime::block_on(async {
            let engine = TesseractOcrEngine::with_binary("apollo-missing-tesseract");
            let capture = CaptureRecord {
                id: CaptureId::new("capture-1").expect("capture id should be valid"),
                session_id: None,
                image_path: "/tmp/does-not-matter.png".to_string(),
                width: 100,
                height: 100,
                ocr_status: OcrStatus::Pending,
            };

            let error = engine
                .extract_text(&capture)
                .await
                .expect_err("missing binary should be reported");

            assert_eq!(error.kind, ApplicationErrorKind::Unavailable);
        });
    }
}
