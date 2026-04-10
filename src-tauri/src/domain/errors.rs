use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("field `{field}` cannot be empty")]
    EmptyValue { field: &'static str },
    #[error("invalid model key `{value}`")]
    InvalidModelKey { value: String },
    #[error("invalid shortcut accelerator `{value}`")]
    InvalidShortcutAccelerator { value: String },
    #[error("conversation message content cannot be empty")]
    EmptyConversationMessage,
}
