use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::domain::value_objects::identifiers::{CaptureId, SessionId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OcrStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl OcrStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl FromStr for OcrStatus {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pending" => Ok(Self::Pending),
            "processing" => Ok(Self::Processing),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err("unknown OCR status"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureRecord {
    pub id: CaptureId,
    pub session_id: Option<SessionId>,
    pub image_path: String,
    pub width: u32,
    pub height: u32,
    pub ocr_status: OcrStatus,
}
