use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::domain::{
    entities::configured_provider::ProviderKind,
    value_objects::{identifiers::SessionId, model_key::ModelKey},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisSourceKind {
    ScreenCapture,
    ManualText,
    FileImport,
}

impl AnalysisSourceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ScreenCapture => "screen_capture",
            Self::ManualText => "manual_text",
            Self::FileImport => "file_import",
        }
    }
}

impl FromStr for AnalysisSourceKind {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "screen_capture" => Ok(Self::ScreenCapture),
            "manual_text" => Ok(Self::ManualText),
            "file_import" => Ok(Self::FileImport),
            _ => Err("unknown analysis source kind"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionSession {
    pub id: SessionId,
    pub provider_kind: ProviderKind,
    pub model_key: ModelKey,
    pub source_kind: AnalysisSourceKind,
    pub ocr_text: Option<String>,
    pub user_notes: Option<String>,
    pub request_prompt: Option<String>,
    pub response_text: Option<String>,
}
