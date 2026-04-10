mod support;

use support::{ProviderKind, contract_harness, sample_settings};

#[test]
fn settings_round_trip_preserves_provider_model_prompt_and_shortcuts() {
    let subject = contract_harness();
    let expected = sample_settings();

    subject
        .save_settings(&expected)
        .expect("settings should persist");

    let actual = subject.load_settings().expect("settings should load");

    assert_eq!(actual, expected);
}

#[test]
fn settings_allow_switching_between_http_and_cli_providers() {
    let subject = contract_harness();
    let mut settings = sample_settings();
    settings.preferred_provider = ProviderKind::CodexCli;
    settings.preferred_model_id = "codex-latest".to_string();

    subject
        .save_settings(&settings)
        .expect("provider selection should persist");

    let loaded = subject.load_settings().expect("settings should load");

    assert_eq!(loaded.preferred_provider, ProviderKind::CodexCli);
    assert_eq!(loaded.preferred_model_id, "codex-latest");
}
