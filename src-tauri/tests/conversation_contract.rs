mod support;

use support::{
    MessageRole, ProviderKind, phase1_harness, sample_follow_up_message, sample_request,
};

#[test]
#[ignore = "Phase 2 will provide the conversation workflow contract"]
fn composed_prompt_includes_base_prompt_ocr_notes_and_previous_context() {
    let subject = phase1_harness();
    let request = sample_request(ProviderKind::OpenAi, "gpt-4.1-mini");

    let prompt = subject.compose_prompt(&request);

    assert!(prompt.contains("Act as a language tutor"));
    assert!(prompt.contains("I have been looking forward to this trip for ages."));
    assert!(prompt.contains("Explain the nuance of looking forward to."));
    assert!(prompt.contains("What does this sentence mean?"));
}

#[test]
#[ignore = "Phase 2 will provide the conversation workflow contract"]
fn continuing_a_conversation_preserves_previous_messages_before_new_turns() {
    let subject = phase1_harness();
    let session_id = "session-001";
    let follow_up = sample_follow_up_message();

    subject
        .append_message(session_id, &follow_up)
        .expect("follow-up message should append");

    let conversation = subject
        .load_conversation(session_id)
        .expect("conversation should load");

    assert!(
        conversation
            .iter()
            .any(|message| message.role == MessageRole::Assistant)
    );
    assert!(conversation.contains(&follow_up));
}
