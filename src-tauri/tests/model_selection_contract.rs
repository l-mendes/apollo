mod support;

use support::{ProviderChannel, ProviderKind, phase1_harness};

#[test]
#[ignore = "Phase 2 will provide the provider registry and model selection contract"]
fn active_model_resolution_returns_the_exact_requested_manual_catalog_entry() {
    let subject = phase1_harness();

    let model = subject
        .resolve_active_model(ProviderKind::Anthropic, "claude-3-7-sonnet")
        .expect("preferred provider and model should resolve");

    assert_eq!(model.provider_kind, ProviderKind::Anthropic);
    assert_eq!(model.model_id, "claude-3-7-sonnet");
    assert!(model.manually_managed);
}

#[test]
#[ignore = "Phase 2 will provide the provider registry and model selection contract"]
fn cli_model_resolution_returns_cli_catalog_metadata() {
    let subject = phase1_harness();

    let model = subject
        .resolve_active_model(ProviderKind::ClaudeCli, "claude-cli-default")
        .expect("cli providers should resolve manually managed models");

    assert_eq!(model.provider_kind, ProviderKind::ClaudeCli);
    assert_eq!(model.channel, ProviderChannel::Cli);
}
