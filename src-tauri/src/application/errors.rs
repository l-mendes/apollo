use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::errors::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplicationErrorKind {
    Validation,
    Authentication,
    Timeout,
    Unavailable,
    InvalidConfiguration,
    MissingCli,
    NotFound,
    Conflict,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[error("{kind:?}: {message}")]
pub struct ApplicationError {
    pub kind: ApplicationErrorKind,
    pub message: String,
}

impl ApplicationError {
    pub fn new(kind: ApplicationErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl From<DomainError> for ApplicationError {
    fn from(value: DomainError) -> Self {
        Self::new(ApplicationErrorKind::Validation, value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::errors::DomainError;

    use super::{ApplicationError, ApplicationErrorKind};

    #[test]
    fn converts_domain_errors_to_validation_errors() {
        let error = ApplicationError::from(DomainError::EmptyConversationMessage);

        assert_eq!(error.kind, ApplicationErrorKind::Validation);
        assert!(!error.message.is_empty());
    }
}
