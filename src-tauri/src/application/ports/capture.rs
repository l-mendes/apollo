use async_trait::async_trait;

use crate::{
    application::{dto::ocr::OcrTextExtraction, errors::ApplicationError},
    domain::entities::capture_record::CaptureRecord,
};

#[async_trait]
pub trait ScreenCapturePort: Send + Sync {
    async fn capture_area(&self) -> Result<CaptureRecord, ApplicationError>;
}

#[async_trait]
pub trait OcrEngine: Send + Sync {
    async fn extract_text(
        &self,
        capture: &CaptureRecord,
    ) -> Result<OcrTextExtraction, ApplicationError>;
}
