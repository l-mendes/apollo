use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};

use apollo::{
    application::{
        dto::analysis::AnalyzeCaptureRequest,
        errors::{ApplicationError, ApplicationErrorKind},
        ports::provider::CliProviderExecutor,
    },
    domain::{entities::configured_provider::ProviderKind, value_objects::model_key::ModelKey},
    infrastructure::providers::cli::{
        command_runner::{CommandExecutionRequest, CommandExecutionResult, CommandRunner},
        executor::{CliCommandProfile, GenericCliProviderExecutor, PromptMode},
    },
};

#[derive(Clone, Default)]
struct FakeRunner {
    responses: Arc<Mutex<VecDeque<Result<CommandExecutionResult, ApplicationError>>>>,
    requests: Arc<Mutex<Vec<CommandExecutionRequest>>>,
}

impl FakeRunner {
    fn with_responses(responses: Vec<Result<CommandExecutionResult, ApplicationError>>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::from(responses))),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn requests(&self) -> Vec<CommandExecutionRequest> {
        self.requests
            .lock()
            .expect("request mutex should lock")
            .clone()
    }
}

#[async_trait::async_trait]
impl CommandRunner for FakeRunner {
    async fn run(
        &self,
        request: CommandExecutionRequest,
    ) -> Result<CommandExecutionResult, ApplicationError> {
        self.requests
            .lock()
            .expect("request mutex should lock")
            .push(request);

        self.responses
            .lock()
            .expect("response mutex should lock")
            .pop_front()
            .expect("fake response should exist")
    }
}

fn request(provider_kind: ProviderKind, model_key: &str) -> AnalyzeCaptureRequest {
    AnalyzeCaptureRequest {
        provider_kind,
        model_key: ModelKey::new(model_key).expect("model key should be valid"),
        base_prompt: "Explain this sentence.".to_string(),
        ocr_text: "I have been looking forward to this trip.".to_string(),
        user_notes: Some("Keep it concise.".to_string()),
        conversation_context: Vec::new(),
    }
}

fn profile(
    provider_kind: ProviderKind,
    binary: &str,
    prompt_mode: PromptMode,
) -> CliCommandProfile {
    CliCommandProfile {
        provider_kind,
        binary: binary.to_string(),
        args: vec!["run".to_string()],
        availability_args: vec!["--version".to_string()],
        prompt_mode,
        timeout: Duration::from_secs(10),
    }
}

#[test]
fn cli_executor_normalizes_stdout_to_common_response_shape() {
    tauri::async_runtime::block_on(async {
        let runner = FakeRunner::with_responses(vec![Ok(CommandExecutionResult {
            exit_code: Some(0),
            stdout: "Codex answer".to_string(),
            stderr: String::new(),
            timed_out: false,
        })]);
        let executor = GenericCliProviderExecutor::new(
            profile(ProviderKind::CodexCli, "codex", PromptMode::Stdin),
            Arc::new(runner.clone()),
        );

        let response = executor
            .execute(&request(ProviderKind::CodexCli, "codex-latest"))
            .await
            .expect("stdout should normalize");

        assert_eq!(response.provider_kind, ProviderKind::CodexCli);
        assert_eq!(response.answer, "Codex answer");
        assert_eq!(response.raw_output, "Codex answer");

        let requests = runner.requests();
        assert_eq!(requests[0].stdin.as_deref(), Some("Explain this sentence."));
    });
}

#[test]
fn cli_executor_appends_argument_prompt_without_using_stdin() {
    tauri::async_runtime::block_on(async {
        let runner = FakeRunner::with_responses(vec![Ok(CommandExecutionResult {
            exit_code: Some(0),
            stdout: "Copilot answer".to_string(),
            stderr: String::new(),
            timed_out: false,
        })]);
        let executor = GenericCliProviderExecutor::new(
            CliCommandProfile {
                provider_kind: ProviderKind::CopilotCli,
                binary: "copilot".to_string(),
                args: vec![
                    "--allow-all-tools".to_string(),
                    "--silent".to_string(),
                    "-p".to_string(),
                ],
                availability_args: vec!["--version".to_string()],
                prompt_mode: PromptMode::Argument,
                timeout: Duration::from_secs(10),
            },
            Arc::new(runner.clone()),
        );

        let response = executor
            .execute(&request(ProviderKind::CopilotCli, "copilot-chat"))
            .await
            .expect("argument prompt mode should execute");

        assert_eq!(response.provider_kind, ProviderKind::CopilotCli);
        assert_eq!(response.answer, "Copilot answer");

        let requests = runner.requests();
        assert_eq!(
            requests[0].args,
            vec![
                "--allow-all-tools".to_string(),
                "--silent".to_string(),
                "-p".to_string(),
                "Explain this sentence.".to_string(),
            ]
        );
        assert_eq!(requests[0].stdin, None);
    });
}

#[test]
fn cli_executor_reports_missing_binary_explicitly() {
    tauri::async_runtime::block_on(async {
        let runner = FakeRunner::with_responses(vec![Err(ApplicationError::new(
            ApplicationErrorKind::MissingCli,
            "binary not found",
        ))]);
        let executor = GenericCliProviderExecutor::new(
            profile(ProviderKind::CopilotCli, "copilot", PromptMode::Stdin),
            Arc::new(runner),
        );

        let error = executor
            .probe_availability()
            .await
            .expect_err("missing CLI should be explicit");

        assert_eq!(error.kind, ApplicationErrorKind::MissingCli);
    });
}

#[test]
fn cli_executor_maps_timeout_and_authentication_errors() {
    tauri::async_runtime::block_on(async {
        let timeout_runner = FakeRunner::with_responses(vec![Ok(CommandExecutionResult {
            exit_code: None,
            stdout: String::new(),
            stderr: "timed out".to_string(),
            timed_out: true,
        })]);
        let timeout_executor = GenericCliProviderExecutor::new(
            profile(ProviderKind::ClaudeCli, "claude", PromptMode::Argument),
            Arc::new(timeout_runner),
        );

        let timeout_error = timeout_executor
            .execute(&request(ProviderKind::ClaudeCli, "claude-cli-default"))
            .await
            .expect_err("timeouts should map explicitly");

        assert_eq!(timeout_error.kind, ApplicationErrorKind::Timeout);

        let auth_runner = FakeRunner::with_responses(vec![Ok(CommandExecutionResult {
            exit_code: Some(1),
            stdout: String::new(),
            stderr: "authentication failed: login required".to_string(),
            timed_out: false,
        })]);
        let auth_executor = GenericCliProviderExecutor::new(
            profile(ProviderKind::ClaudeCli, "claude", PromptMode::Argument),
            Arc::new(auth_runner),
        );

        let auth_error = auth_executor
            .execute(&request(ProviderKind::ClaudeCli, "claude-cli-default"))
            .await
            .expect_err("auth failures should map explicitly");

        assert_eq!(auth_error.kind, ApplicationErrorKind::Authentication);
    });
}
