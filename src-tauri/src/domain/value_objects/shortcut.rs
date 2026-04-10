use serde::{Deserialize, Serialize};

use crate::domain::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShortcutAction(String);

impl ShortcutAction {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(DomainError::EmptyValue {
                field: "shortcut_action",
            });
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShortcutAccelerator(String);

impl ShortcutAccelerator {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();

        if value.trim().is_empty() || !value.contains('+') {
            return Err(DomainError::InvalidShortcutAccelerator { value });
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{ShortcutAccelerator, ShortcutAction};

    #[test]
    fn validates_shortcut_action_and_accelerator() {
        assert!(ShortcutAction::new("capture_screen").is_ok());
        assert!(ShortcutAccelerator::new("CmdOrCtrl+Shift+A").is_ok());
        assert!(ShortcutAccelerator::new("ShiftA").is_err());
    }
}
