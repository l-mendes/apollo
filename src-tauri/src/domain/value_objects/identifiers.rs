use serde::{Deserialize, Serialize};

use crate::domain::errors::DomainError;

macro_rules! string_identifier {
    ($name:ident, $field:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();

                if value.trim().is_empty() {
                    return Err(DomainError::EmptyValue { field: $field });
                }

                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

string_identifier!(CaptureId, "capture_id");
string_identifier!(MessageId, "message_id");
string_identifier!(ProviderId, "provider_id");
string_identifier!(SessionId, "session_id");

#[cfg(test)]
mod tests {
    use super::SessionId;

    #[test]
    fn rejects_empty_identifiers() {
        assert!(SessionId::new("   ").is_err());
    }
}
