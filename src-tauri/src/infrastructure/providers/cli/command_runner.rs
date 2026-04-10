use std::{
    io::Write,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use async_trait::async_trait;

use crate::application::errors::{ApplicationError, ApplicationErrorKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandExecutionRequest {
    pub binary: String,
    pub args: Vec<String>,
    pub stdin: Option<String>,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandExecutionResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

#[async_trait]
pub trait CommandRunner: Send + Sync {
    async fn run(
        &self,
        request: CommandExecutionRequest,
    ) -> Result<CommandExecutionResult, ApplicationError>;
}

#[derive(Debug, Default)]
pub struct ProcessCommandRunner;

#[async_trait]
impl CommandRunner for ProcessCommandRunner {
    async fn run(
        &self,
        request: CommandExecutionRequest,
    ) -> Result<CommandExecutionResult, ApplicationError> {
        tauri::async_runtime::spawn_blocking(move || run_process(request))
            .await
            .map_err(|error| {
                ApplicationError::new(
                    ApplicationErrorKind::Unknown,
                    format!("failed to join command runner task: {error}"),
                )
            })?
    }
}

fn run_process(
    request: CommandExecutionRequest,
) -> Result<CommandExecutionResult, ApplicationError> {
    let mut child = Command::new(&request.binary)
        .args(&request.args)
        .stdin(if request.stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            let kind = if error.kind() == std::io::ErrorKind::NotFound {
                ApplicationErrorKind::MissingCli
            } else {
                ApplicationErrorKind::Unavailable
            };

            ApplicationError::new(kind, format!("failed to launch CLI command: {error}"))
        })?;

    if let Some(stdin) = request.stdin {
        let mut stdin_handle = child.stdin.take().ok_or_else(|| {
            ApplicationError::new(
                ApplicationErrorKind::Unknown,
                "CLI stdin was requested but unavailable",
            )
        })?;
        stdin_handle.write_all(stdin.as_bytes()).map_err(|error| {
            ApplicationError::new(
                ApplicationErrorKind::Unavailable,
                format!("failed to write to CLI stdin: {error}"),
            )
        })?;
    }

    let started_at = Instant::now();
    loop {
        if started_at.elapsed() >= request.timeout {
            let _ = child.kill();
            let output = child.wait_with_output().map_err(|error| {
                ApplicationError::new(
                    ApplicationErrorKind::Unavailable,
                    format!("failed to collect timed-out CLI output: {error}"),
                )
            })?;

            return Ok(CommandExecutionResult {
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                timed_out: true,
            });
        }

        match child.try_wait().map_err(|error| {
            ApplicationError::new(
                ApplicationErrorKind::Unavailable,
                format!("failed while polling CLI command: {error}"),
            )
        })? {
            Some(_) => {
                let output = child.wait_with_output().map_err(|error| {
                    ApplicationError::new(
                        ApplicationErrorKind::Unavailable,
                        format!("failed to collect CLI output: {error}"),
                    )
                })?;

                return Ok(CommandExecutionResult {
                    exit_code: output.status.code(),
                    stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                    timed_out: false,
                });
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
}
