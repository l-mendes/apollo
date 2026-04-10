use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::domain::value_objects::identifiers::ProviderId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderKind {
    OpenAi,
    Anthropic,
    OllamaCloud,
    OllamaLocal,
    ClaudeCli,
    CodexCli,
    CopilotCli,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderChannel {
    Http,
    Cli,
}

impl ProviderKind {
    pub const fn channel(self) -> ProviderChannel {
        match self {
            Self::OpenAi | Self::Anthropic | Self::OllamaCloud | Self::OllamaLocal => {
                ProviderChannel::Http
            }
            Self::ClaudeCli | Self::CodexCli | Self::CopilotCli => ProviderChannel::Cli,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::OllamaCloud => "ollama_cloud",
            Self::OllamaLocal => "ollama_local",
            Self::ClaudeCli => "claude_cli",
            Self::CodexCli => "codex_cli",
            Self::CopilotCli => "copilot_cli",
        }
    }
}

impl FromStr for ProviderKind {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "openai" => Ok(Self::OpenAi),
            "anthropic" => Ok(Self::Anthropic),
            "ollama_cloud" => Ok(Self::OllamaCloud),
            "ollama_local" => Ok(Self::OllamaLocal),
            "claude_cli" => Ok(Self::ClaudeCli),
            "codex_cli" => Ok(Self::CodexCli),
            "copilot_cli" => Ok(Self::CopilotCli),
            _ => Err("unknown provider kind"),
        }
    }
}

impl ProviderChannel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Cli => "cli",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfiguredProvider {
    pub id: ProviderId,
    pub kind: ProviderKind,
    pub display_name: String,
    pub endpoint: Option<String>,
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::{ProviderChannel, ProviderKind};

    #[test]
    fn infers_provider_channel_from_kind() {
        assert_eq!(ProviderKind::OpenAi.channel(), ProviderChannel::Http);
        assert_eq!(ProviderKind::ClaudeCli.channel(), ProviderChannel::Cli);
    }
}
