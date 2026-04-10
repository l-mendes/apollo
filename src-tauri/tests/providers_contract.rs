mod support;

use support::{ProviderChannel, ProviderFailureKind, ProviderKind, phase1_harness, sample_request};

#[test]
#[ignore = "Phase 2 will provide provider contracts and adapters"]
fn http_providers_expose_non_empty_manually_managed_model_catalogs() {
    let subject = phase1_harness();

    for provider_kind in [
        ProviderKind::OpenAi,
        ProviderKind::Anthropic,
        ProviderKind::OllamaCloud,
        ProviderKind::OllamaLocal,
    ] {
        let models = subject
            .list_models(provider_kind)
            .expect("provider catalog should load");

        assert!(!models.is_empty());
        assert!(models.iter().all(|model| model.manually_managed));
        assert!(
            models
                .iter()
                .all(|model| model.channel == ProviderChannel::Http)
        );
    }
}

#[test]
#[ignore = "Phase 2 will provide provider contracts and adapters"]
fn provider_analysis_is_normalized_to_a_common_response_shape() {
    let subject = phase1_harness();
    let request = sample_request(ProviderKind::OpenAi, "gpt-4.1-mini");

    let response = subject
        .analyze(&request)
        .expect("analysis should be normalized");

    assert_eq!(response.provider_kind, ProviderKind::OpenAi);
    assert_eq!(response.model_id, "gpt-4.1-mini");
    assert!(!response.answer.trim().is_empty());
    assert!(!response.raw_output.trim().is_empty());
}

#[test]
#[ignore = "Phase 2 will provide provider contracts and adapters"]
fn provider_timeouts_are_reported_with_a_specific_failure_kind() {
    let subject = phase1_harness();
    let request = sample_request(ProviderKind::Anthropic, "claude-3-7-sonnet");

    let error = subject
        .analyze(&request)
        .expect_err("timeout scenarios should surface as errors");

    assert_eq!(error.kind, ProviderFailureKind::Timeout);
    assert!(!error.message.trim().is_empty());
}

#[test]
#[ignore = "Phase 2 will provide provider contracts and adapters"]
fn provider_unavailability_is_distinguished_from_authentication_failures() {
    let subject = phase1_harness();
    let request = sample_request(ProviderKind::OllamaCloud, "llama3.2");

    let error = subject
        .analyze(&request)
        .expect_err("unavailable provider scenarios should surface as errors");

    assert!(matches!(
        error.kind,
        ProviderFailureKind::Unavailable | ProviderFailureKind::Authentication
    ));
}
