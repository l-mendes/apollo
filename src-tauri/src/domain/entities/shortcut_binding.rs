use serde::{Deserialize, Serialize};

use crate::domain::value_objects::shortcut::{ShortcutAccelerator, ShortcutAction};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutBinding {
    pub action: ShortcutAction,
    pub accelerator: ShortcutAccelerator,
    pub enabled: bool,
}
