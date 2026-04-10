use async_trait::async_trait;

use crate::{
    application::errors::ApplicationError,
    domain::{
        entities::{
            conversation_message::ConversationMessage, interaction_session::InteractionSession,
            provider_model::ProviderModel, shortcut_binding::ShortcutBinding,
            user_settings::UserSettings,
        },
        value_objects::identifiers::SessionId,
    },
};

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn load(&self) -> Result<UserSettings, ApplicationError>;

    async fn save(&self, settings: &UserSettings) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait ShortcutRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<ShortcutBinding>, ApplicationError>;

    async fn replace_all(&self, shortcuts: &[ShortcutBinding]) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait HistoryRepository: Send + Sync {
    async fn save_session(&self, session: &InteractionSession) -> Result<(), ApplicationError>;

    async fn list_sessions(&self) -> Result<Vec<InteractionSession>, ApplicationError>;

    async fn delete_session(&self, session_id: &SessionId) -> Result<(), ApplicationError>;

    async fn clear_sessions(&self) -> Result<(), ApplicationError>;
}

#[async_trait]
pub trait ConversationRepository: Send + Sync {
    async fn append_message(&self, message: &ConversationMessage) -> Result<(), ApplicationError>;

    async fn load_by_session(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<ConversationMessage>, ApplicationError>;
}

#[async_trait]
pub trait ProviderModelCatalog: Send + Sync {
    async fn list_by_provider(
        &self,
        provider_kind: crate::domain::entities::configured_provider::ProviderKind,
    ) -> Result<Vec<ProviderModel>, ApplicationError>;
}
