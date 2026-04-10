use async_trait::async_trait;

use crate::application::{
    dto::analysis::{ContinueConversationRequest, ContinueConversationResponse},
    errors::ApplicationError,
};

#[async_trait]
pub trait ContinueConversation: Send + Sync {
    async fn execute(
        &self,
        request: ContinueConversationRequest,
    ) -> Result<ContinueConversationResponse, ApplicationError>;
}
