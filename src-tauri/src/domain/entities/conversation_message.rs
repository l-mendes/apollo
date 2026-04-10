use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::domain::{
    errors::DomainError,
    value_objects::identifiers::{MessageId, SessionId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl MessageRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

impl FromStr for MessageRole {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "system" => Ok(Self::System),
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            _ => Err("unknown message role"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: MessageId,
    pub session_id: SessionId,
    pub role: MessageRole,
    pub content: String,
}

impl ConversationMessage {
    pub fn new(
        id: MessageId,
        session_id: SessionId,
        role: MessageRole,
        content: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let content = content.into();

        if content.trim().is_empty() {
            return Err(DomainError::EmptyConversationMessage);
        }

        Ok(Self {
            id,
            session_id,
            role,
            content,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ConversationMessage, MessageRole};
    use crate::domain::value_objects::identifiers::{MessageId, SessionId};

    #[test]
    fn rejects_blank_conversation_messages() {
        let message = ConversationMessage::new(
            MessageId::new("message-1").expect("message id should be valid"),
            SessionId::new("session-1").expect("session id should be valid"),
            MessageRole::User,
            "   ",
        );

        assert!(message.is_err());
    }
}
