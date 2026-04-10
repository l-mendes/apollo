mod support;

use support::{phase1_harness, sample_follow_up_message, sample_history_session};

#[test]
#[ignore = "Phase 2 will provide the history repository contract"]
fn history_persists_core_session_fields_for_each_analysis() {
    let subject = phase1_harness();
    let session = sample_history_session();

    subject
        .save_session(&session)
        .expect("history session should persist");

    let sessions = subject
        .list_sessions()
        .expect("history should list sessions");

    assert!(sessions.contains(&session));
}

#[test]
#[ignore = "Phase 2 will provide the history repository contract"]
fn history_appends_follow_up_messages_to_an_existing_session() {
    let subject = phase1_harness();
    let session = sample_history_session();
    let message = sample_follow_up_message();

    subject
        .save_session(&session)
        .expect("history session should persist");
    subject
        .append_message(&session.session_id, &message)
        .expect("follow-up messages should persist");

    let conversation = subject
        .load_conversation(&session.session_id)
        .expect("conversation should load");

    assert!(conversation.contains(&message));
}
