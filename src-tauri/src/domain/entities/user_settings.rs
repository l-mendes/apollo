use serde::{Deserialize, Serialize};

use crate::domain::{
    entities::{configured_provider::ProviderKind, shortcut_binding::ShortcutBinding},
    value_objects::model_key::ModelKey,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSettings {
    pub preferred_provider: ProviderKind,
    pub preferred_model: ModelKey,
    pub base_prompt: String,
    /// Tesseract language code used for OCR (e.g. "por", "eng").
    pub ocr_language: String,
    /// Natural-language name passed in the AI prompt to request the response
    /// in a specific language (e.g. "Português", "English").
    pub output_language: String,
    pub shortcuts: Vec<ShortcutBinding>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            preferred_provider: ProviderKind::OpenAi,
            preferred_model: ModelKey::new("gpt-4.1-mini")
                .expect("default model key should be valid"),
            base_prompt: "You are Apollo, a concise and context-aware language tutor. Explain meaning, nuance, grammar, and practical usage with short examples.".to_string(),
            ocr_language: "eng".to_string(),
            output_language: "Português".to_string(),
            shortcuts: vec![
                ShortcutBinding {
                    action: crate::domain::value_objects::shortcut::ShortcutAction::new(
                        "capture_screen",
                    )
                    .expect("default shortcut action should be valid"),
                    accelerator: crate::domain::value_objects::shortcut::ShortcutAccelerator::new(
                        "CmdOrCtrl+Shift+A",
                    )
                    .expect("default shortcut accelerator should be valid"),
                    enabled: true,
                },
                ShortcutBinding {
                    action: crate::domain::value_objects::shortcut::ShortcutAction::new(
                        "open_settings",
                    )
                    .expect("default shortcut action should be valid"),
                    accelerator: crate::domain::value_objects::shortcut::ShortcutAccelerator::new(
                        "CmdOrCtrl+,",
                    )
                    .expect("default shortcut accelerator should be valid"),
                    enabled: true,
                },
                ShortcutBinding {
                    action: crate::domain::value_objects::shortcut::ShortcutAction::new(
                        "open_history",
                    )
                    .expect("default shortcut action should be valid"),
                    accelerator: crate::domain::value_objects::shortcut::ShortcutAccelerator::new(
                        "CmdOrCtrl+Shift+H",
                    )
                    .expect("default shortcut accelerator should be valid"),
                    enabled: true,
                },
            ],
        }
    }
}
