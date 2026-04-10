mod support;

use support::{ProviderFailureKind, ProviderKind, contract_harness, sample_request};

#[test]
fn cli_providers_surface_missing_binary_as_a_specific_error() {
    let subject = contract_harness();

    let error = subject
        .probe_cli_availability(ProviderKind::CopilotCli)
        .expect_err("missing CLI should not look like provider unavailability");

    assert_eq!(error.kind, ProviderFailureKind::MissingCli);
}

#[test]
fn cli_providers_normalize_stdout_to_the_common_response_shape() {
    let subject = contract_harness();
    let request = sample_request(ProviderKind::CodexCli, "codex-latest");

    let response = subject
        .execute_cli_analysis(ProviderKind::CodexCli, &request)
        .expect("cli output should normalize to the common response shape");

    assert_eq!(response.provider_kind, ProviderKind::CodexCli);
    assert_eq!(response.model_id, "codex-latest");
    assert!(!response.answer.trim().is_empty());
}

#[test]
fn cli_providers_report_timeout_and_authentication_failures_explicitly() {
    let subject = contract_harness();
    let request = sample_request(ProviderKind::ClaudeCli, "claude-cli-default");

    let error = subject
        .execute_cli_analysis(ProviderKind::ClaudeCli, &request)
        .expect_err("cli timeout and auth failures should be explicit");

    assert!(matches!(
        error.kind,
        ProviderFailureKind::Timeout | ProviderFailureKind::Authentication
    ));
}
