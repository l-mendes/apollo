mod support;

use support::{
    MessageRole, MessageSpec, ProviderKind, contract_harness, sample_follow_up_message,
    sample_request,
};

#[test]
fn composed_prompt_includes_base_prompt_ocr_notes_and_previous_context() {
    let subject = contract_harness();
    let request = sample_request(ProviderKind::OpenAi, "gpt-4.1-mini");

    let prompt = subject.compose_prompt(&request);

    assert!(prompt.contains("Act as a language tutor"));
    assert!(prompt.contains("I have been looking forward to this trip for ages."));
    assert!(prompt.contains("Explain the nuance of looking forward to."));
    assert!(prompt.contains("What does this sentence mean?"));
}

#[test]
fn continuing_a_conversation_preserves_previous_messages_before_new_turns() {
    let subject = contract_harness();
    let mut session = support::sample_history_session();
    session.provider_kind = ProviderKind::OpenAi;
    session.model_id = "gpt-4.1-mini".to_string();
    let follow_up = sample_follow_up_message();

    subject
        .save_session(&session)
        .expect("session should persist");
    subject
        .append_message(
            &session.session_id,
            &MessageSpec {
                role: MessageRole::Assistant,
                content: session.response_text.clone(),
            },
        )
        .expect("seed assistant message should append");
    let appended = subject
        .continue_conversation(
            &session.session_id,
            session.provider_kind,
            &session.model_id,
            &follow_up.content,
        )
        .expect("conversation continuation should succeed");

    let conversation = subject
        .load_conversation(&session.session_id)
        .expect("conversation should load");

    assert!(appended.contains(&follow_up));
    assert!(
        appended
            .iter()
            .any(|message| message.role == MessageRole::Assistant)
    );
    assert!(
        conversation
            .iter()
            .any(|message| message.content == session.response_text)
    );
    assert!(conversation.contains(&follow_up));
}
