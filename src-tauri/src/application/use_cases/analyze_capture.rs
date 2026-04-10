use async_trait::async_trait;

use crate::application::{
    dto::analysis::{AnalyzeCaptureRequest, AnalyzeCaptureResponse},
    errors::ApplicationError,
};

#[async_trait]
pub trait AnalyzeCapture: Send + Sync {
    async fn execute(
        &self,
        request: AnalyzeCaptureRequest,
    ) -> Result<AnalyzeCaptureResponse, ApplicationError>;
}
