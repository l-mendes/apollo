use std::{env, time::Duration};

use crate::{
    application::errors::{ApplicationError, ApplicationErrorKind},
    domain::entities::configured_provider::ProviderKind,
    infrastructure::providers::cli::executor::{CliCommandProfile, PromptMode},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpProviderProfile {
    pub kind: ProviderKind,
    pub endpoint: String,
    pub api_key: Option<String>,
    pub credentials_required: bool,
    pub auth_header: Option<String>,
    pub auth_prefix: Option<String>,
    pub extra_headers: Vec<(String, String)>,
    pub timeout: Duration,
}

impl HttpProviderProfile {
    pub fn openai() -> Self {
        Self {
            kind: ProviderKind::OpenAi,
            endpoint: env::var("APOLLO_OPENAI_ENDPOINT")
                .unwrap_or_else(|_| "https://api.openai.com/v1/responses".to_string()),
            api_key: env::var("OPENAI_API_KEY").ok(),
            credentials_required: true,
            auth_header: Some("Authorization".to_string()),
            auth_prefix: Some("Bearer ".to_string()),
            extra_headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            timeout: Duration::from_secs(45),
        }
    }

    pub fn anthropic() -> Self {
        Self {
            kind: ProviderKind::Anthropic,
            endpoint: env::var("APOLLO_ANTHROPIC_ENDPOINT")
                .unwrap_or_else(|_| "https://api.anthropic.com/v1/messages".to_string()),
            api_key: env::var("ANTHROPIC_API_KEY").ok(),
            credentials_required: true,
            auth_header: Some("x-api-key".to_string()),
            auth_prefix: None,
            extra_headers: vec![
                ("anthropic-version".to_string(), "2023-06-01".to_string()),
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            timeout: Duration::from_secs(45),
        }
    }

    pub fn ollama_cloud(endpoint: String) -> Self {
        Self {
            kind: ProviderKind::OllamaCloud,
            endpoint,
            api_key: env::var("OLLAMA_API_KEY").ok(),
            credentials_required: false,
            auth_header: Some("Authorization".to_string()),
            auth_prefix: Some("Bearer ".to_string()),
            extra_headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            timeout: Duration::from_secs(45),
        }
    }

    pub fn ollama_local(endpoint: String) -> Self {
        Self {
            kind: ProviderKind::OllamaLocal,
            endpoint,
            api_key: None,
            credentials_required: false,
            auth_header: None,
            auth_prefix: None,
            extra_headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            timeout: Duration::from_secs(45),
        }
    }

    pub fn headers(&self) -> Result<Vec<(String, String)>, ApplicationError> {
        let mut headers = self.extra_headers.clone();

        match (&self.auth_header, &self.api_key) {
            (Some(_), None) if self.credentials_required => {
                return Err(ApplicationError::new(
                    ApplicationErrorKind::InvalidConfiguration,
                    format!("missing credentials for {}", self.kind.as_str()),
                ));
            }
            (Some(_), None) => {}
            (Some(header), Some(api_key)) => headers.push((
                header.clone(),
                format!(
                    "{}{}",
                    self.auth_prefix.clone().unwrap_or_default(),
                    api_key
                ),
            )),
            (None, _) => {}
        }

        Ok(headers)
    }
}

pub fn default_ollama_cloud_endpoint() -> String {
    env::var("APOLLO_OLLAMA_CLOUD_ENDPOINT")
        .unwrap_or_else(|_| "https://ollama.com/api/generate".to_string())
}

pub fn default_ollama_local_endpoint() -> String {
    env::var("APOLLO_OLLAMA_LOCAL_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:11434/api/generate".to_string())
}

pub fn default_cli_profiles() -> Vec<CliCommandProfile> {
    vec![
        CliCommandProfile {
            provider_kind: ProviderKind::ClaudeCli,
            binary: env::var("APOLLO_CLAUDE_CLI_BINARY").unwrap_or_else(|_| "claude".to_string()),
            args: vec!["-p".to_string()],
            availability_args: vec!["--version".to_string()],
            prompt_mode: PromptMode::Argument,
            timeout: Duration::from_secs(60),
        },
        CliCommandProfile {
            provider_kind: ProviderKind::CodexCli,
            binary: env::var("APOLLO_CODEX_CLI_BINARY").unwrap_or_else(|_| "codex".to_string()),
            args: vec!["exec".to_string(), "--skip-git-repo-check".to_string()],
            availability_args: vec!["--version".to_string()],
            prompt_mode: PromptMode::Stdin,
            timeout: Duration::from_secs(60),
        },
        CliCommandProfile {
            provider_kind: ProviderKind::CopilotCli,
            binary: env::var("APOLLO_COPILOT_CLI_BINARY").unwrap_or_else(|_| "copilot".to_string()),
            args: vec![
                "--allow-all-tools".to_string(),
                "--silent".to_string(),
                "-p".to_string(),
            ],
            availability_args: vec!["--version".to_string()],
            prompt_mode: PromptMode::Argument,
            timeout: Duration::from_secs(60),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::default_cli_profiles;
    use crate::{
        domain::entities::configured_provider::ProviderKind,
        infrastructure::providers::cli::executor::PromptMode,
    };

    #[test]
    fn copilot_cli_profile_uses_non_interactive_prompt_mode() {
        let copilot = default_cli_profiles()
            .into_iter()
            .find(|profile| profile.provider_kind == ProviderKind::CopilotCli)
            .expect("copilot profile should exist");

        assert_eq!(copilot.prompt_mode, PromptMode::Argument);
        assert_eq!(
            copilot.args,
            vec![
                "--allow-all-tools".to_string(),
                "--silent".to_string(),
                "-p".to_string(),
            ]
        );
    }
}
