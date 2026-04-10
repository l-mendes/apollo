use std::{sync::Arc, time::Duration};

use async_trait::async_trait;

use crate::{
    application::{
        dto::analysis::{AnalyzeCaptureRequest, NormalizedResponse},
        errors::{ApplicationError, ApplicationErrorKind},
        ports::provider::CliProviderExecutor,
    },
    domain::{
        entities::configured_provider::ProviderKind,
        value_objects::reasoning_effort::ReasoningEffort,
    },
    infrastructure::providers::cli::command_runner::{CommandExecutionRequest, CommandRunner},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptMode {
    Stdin,
    Argument,
    ArgumentWithFlag(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReasoningEffortArgument {
    Flag(String),
    CodexConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliCommandProfile {
    pub provider_kind: ProviderKind,
    pub binary: String,
    pub args: Vec<String>,
    pub availability_args: Vec<String>,
    pub model_flag: Option<String>,
    pub reasoning_effort_argument: Option<ReasoningEffortArgument>,
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
        let dynamic_args = build_dynamic_args(&self.profile, request);
        let (args, stdin) = match &self.profile.prompt_mode {
            PromptMode::Stdin => {
                let mut args = self.profile.args.clone();
                args.extend(dynamic_args);
                (args, Some(request.base_prompt.clone()))
            }
            PromptMode::Argument => {
                let mut args = self.profile.args.clone();
                args.extend(dynamic_args);
                args.push(request.base_prompt.clone());
                (args, None)
            }
            PromptMode::ArgumentWithFlag(prompt_flag) => {
                let mut args = self.profile.args.clone();
                args.extend(dynamic_args);
                args.push(prompt_flag.clone());
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

fn build_dynamic_args(profile: &CliCommandProfile, request: &AnalyzeCaptureRequest) -> Vec<String> {
    let mut args = Vec::new();

    if let Some(model_flag) = &profile.model_flag {
        if should_pass_model_key(request.model_key.as_str()) {
            args.push(model_flag.clone());
            args.push(request.model_key.as_str().to_string());
        }
    }

    if let Some(reasoning_effort_argument) = &profile.reasoning_effort_argument {
        match reasoning_effort_argument {
            ReasoningEffortArgument::Flag(flag) => {
                args.push(flag.clone());
                args.push(reasoning_effort_value(
                    profile.provider_kind,
                    request.reasoning_effort,
                ));
            }
            ReasoningEffortArgument::CodexConfig => {
                args.push("-c".to_string());
                args.push(format!(
                    "model_reasoning_effort=\"{}\"",
                    request.reasoning_effort.as_str()
                ));
            }
        }
    }

    args
}

fn should_pass_model_key(model_key: &str) -> bool {
    !matches!(
        model_key,
        "claude-cli-default" | "codex-latest" | "copilot-chat"
    )
}

fn reasoning_effort_value(
    provider_kind: ProviderKind,
    reasoning_effort: ReasoningEffort,
) -> String {
    match provider_kind {
        ProviderKind::ClaudeCli => reasoning_effort.as_claude_effort().to_string(),
        _ => reasoning_effort.as_str().to_string(),
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
