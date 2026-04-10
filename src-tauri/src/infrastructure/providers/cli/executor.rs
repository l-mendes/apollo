use std::{sync::Arc, time::Duration};

use async_trait::async_trait;

use crate::{
    application::{
        dto::analysis::{AnalyzeCaptureRequest, NormalizedResponse},
        errors::{ApplicationError, ApplicationErrorKind},
        ports::provider::CliProviderExecutor,
    },
    domain::entities::configured_provider::ProviderKind,
    infrastructure::providers::cli::command_runner::{CommandExecutionRequest, CommandRunner},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptMode {
    Stdin,
    Argument,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliCommandProfile {
    pub provider_kind: ProviderKind,
    pub binary: String,
    pub args: Vec<String>,
    pub availability_args: Vec<String>,
    pub prompt_mode: PromptMode,
    pub timeout: Duration,
}

pub struct GenericCliProviderExecutor {
    profile: CliCommandProfile,
    runner: Arc<dyn CommandRunner>,
}

impl GenericCliProviderExecutor {
    pub fn new(profile: CliCommandProfile, runner: Arc<dyn CommandRunner>) -> Self {
        Self { profile, runner }
    }
}

#[async_trait]
impl CliProviderExecutor for GenericCliProviderExecutor {
    fn kind(&self) -> ProviderKind {
        self.profile.provider_kind
    }

    async fn probe_availability(&self) -> Result<(), ApplicationError> {
        let result = self
            .runner
            .run(CommandExecutionRequest {
                binary: self.profile.binary.clone(),
                args: self.profile.availability_args.clone(),
                stdin: None,
                timeout: self.profile.timeout,
            })
            .await?;

        if result.timed_out {
            return Err(ApplicationError::new(
                ApplicationErrorKind::Timeout,
                "CLI availability probe timed out",
            ));
        }

        if result.exit_code == Some(0) {
            Ok(())
        } else {
            Err(map_cli_failure(
                result.stderr.as_str(),
                ApplicationErrorKind::MissingCli,
                "CLI availability probe failed",
            ))
        }
    }

    async fn execute(
        &self,
        request: &AnalyzeCaptureRequest,
    ) -> Result<NormalizedResponse, ApplicationError> {
        let (args, stdin) = match self.profile.prompt_mode {
            PromptMode::Stdin => (self.profile.args.clone(), Some(request.base_prompt.clone())),
            PromptMode::Argument => {
                let mut args = self.profile.args.clone();
                args.push(request.base_prompt.clone());
                (args, None)
            }
        };

        let result = self
            .runner
            .run(CommandExecutionRequest {
                binary: self.profile.binary.clone(),
                args,
                stdin,
                timeout: self.profile.timeout,
            })
            .await?;

        if result.timed_out {
            return Err(ApplicationError::new(
                ApplicationErrorKind::Timeout,
                format!("{} CLI timed out", self.kind().as_str()),
            ));
        }

        if result.exit_code.unwrap_or(1) != 0 {
            return Err(map_cli_failure(
                result.stderr.as_str(),
                ApplicationErrorKind::Unavailable,
                "CLI execution failed",
            ));
        }

        let answer = result.stdout.trim();
        if answer.is_empty() {
            return Err(ApplicationError::new(
                ApplicationErrorKind::Unknown,
                "CLI returned an empty stdout payload",
            ));
        }

        Ok(NormalizedResponse {
            provider_kind: self.kind(),
            model_key: request.model_key.clone(),
            answer: answer.to_string(),
            raw_output: result.stdout,
        })
    }
}

fn map_cli_failure(
    stderr: &str,
    default_kind: ApplicationErrorKind,
    fallback_message: &str,
) -> ApplicationError {
    let stderr_lower = stderr.to_lowercase();
    let kind = if stderr_lower.contains("auth")
        || stderr_lower.contains("login")
        || stderr_lower.contains("unauthorized")
        || stderr_lower.contains("forbidden")
    {
        ApplicationErrorKind::Authentication
    } else if stderr_lower.contains("timed out") || stderr_lower.contains("timeout") {
        ApplicationErrorKind::Timeout
    } else if stderr_lower.contains("not found") || stderr_lower.contains("no such file") {
        ApplicationErrorKind::MissingCli
    } else {
        default_kind
    };

    let message = if stderr.trim().is_empty() {
        fallback_message.to_string()
    } else {
        stderr.trim().to_string()
    };

    ApplicationError::new(kind, message)
}
