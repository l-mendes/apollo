use std::sync::Arc;

use apollo::{
    application::ports::repositories::{
        ConversationRepository, HistoryRepository, SettingsRepository, ShortcutRepository,
    },
    domain::{
        entities::{
            configured_provider::ProviderKind,
            conversation_message::{ConversationMessage, MessageRole},
            interaction_session::{AnalysisSourceKind, InteractionSession},
            shortcut_binding::ShortcutBinding,
            user_settings::UserSettings,
        },
        value_objects::{
            identifiers::{MessageId, SessionId},
            model_key::ModelKey,
            shortcut::{ShortcutAccelerator, ShortcutAction},
        },
    },
    infrastructure::persistence::SqliteAppRepository,
};

fn shortcut_binding(action: &str, accelerator: &str) -> ShortcutBinding {
    ShortcutBinding {
        action: ShortcutAction::new(action).expect("shortcut action should be valid"),
        accelerator: ShortcutAccelerator::new(accelerator)
            .expect("shortcut accelerator should be valid"),
        enabled: true,
    }
}

fn bootstrap_repository() -> SqliteAppRepository {
    SqliteAppRepository::in_memory().expect("in-memory repository should initialize")
}

#[test]
fn sqlite_repository_round_trips_settings_and_shortcuts() {
    tauri::async_runtime::block_on(async {
        let repository = bootstrap_repository();
        let shortcuts = vec![
            shortcut_binding("capture_screen", "CmdOrCtrl+Shift+A"),
            shortcut_binding("open_settings", "CmdOrCtrl+,"),
        ];
        let settings = UserSettings {
            preferred_provider: ProviderKind::CodexCli,
            preferred_model: ModelKey::new("codex-latest").expect("model key should be valid"),
            base_prompt: "Answer as a concise language tutor.".to_string(),
            ocr_language: "eng".to_string(),
            output_language: "English".to_string(),
            shortcuts: shortcuts.clone(),
        };

        SettingsRepository::save(&repository, &settings)
            .await
            .expect("settings should persist");

        let loaded_settings = SettingsRepository::load(&repository)
            .await
            .expect("settings should load");
        let loaded_shortcuts = ShortcutRepository::list(&repository)
            .await
            .expect("shortcuts should be listed");

        assert_eq!(loaded_settings, settings);
        assert_eq!(loaded_shortcuts, shortcuts);
    });
}

#[test]
fn sqlite_repository_persists_sessions_and_conversation_messages() {
    tauri::async_runtime::block_on(async {
        let repository = Arc::new(bootstrap_repository());
        let session = InteractionSession {
            id: SessionId::new("session-001").expect("session id should be valid"),
            provider_kind: ProviderKind::Anthropic,
            model_key: ModelKey::new("claude-3-7-sonnet").expect("model key should be valid"),
            source_kind: AnalysisSourceKind::ScreenCapture,
            ocr_text: Some("She made up her mind after reading the article.".to_string()),
            user_notes: Some("Explain the phrasal verb.".to_string()),
            request_prompt: Some("Explain the phrase and give examples.".to_string()),
            response_text: Some("It means she decided firmly.".to_string()),
        };
        let follow_up = ConversationMessage::new(
            MessageId::new("message-002").expect("message id should be valid"),
            session.id.clone(),
            MessageRole::User,
            "Give me two more examples in a casual tone.",
        )
        .expect("message should be valid");

        HistoryRepository::save_session(repository.as_ref(), &session)
            .await
            .expect("session should persist");
        ConversationRepository::append_message(repository.as_ref(), &follow_up)
            .await
            .expect("message should persist");

        let sessions = HistoryRepository::list_sessions(repository.as_ref())
            .await
            .expect("history should list sessions");
        let conversation =
            ConversationRepository::load_by_session(repository.as_ref(), &session.id)
                .await
                .expect("conversation should load");

        assert!(sessions.contains(&session));
        assert!(conversation.contains(&follow_up));
    });
}

#[test]
fn sqlite_repository_returns_messages_in_creation_order_for_a_session() {
    tauri::async_runtime::block_on(async {
        let repository = Arc::new(bootstrap_repository());
        let session_id = SessionId::new("session-ordered").expect("session id should be valid");
        let session = InteractionSession {
            id: session_id.clone(),
            provider_kind: ProviderKind::OpenAi,
            model_key: ModelKey::new("gpt-4.1-mini").expect("model key should be valid"),
            source_kind: AnalysisSourceKind::ManualText,
            ocr_text: None,
            user_notes: Some("Ordered conversation".to_string()),
            request_prompt: Some("Ordered conversation".to_string()),
            response_text: Some("Initial response".to_string()),
        };
        let first = ConversationMessage::new(
            MessageId::new("message-001").expect("message id should be valid"),
            session_id.clone(),
            MessageRole::User,
            "First turn",
        )
        .expect("message should be valid");
        let second = ConversationMessage::new(
            MessageId::new("message-002").expect("message id should be valid"),
            session_id.clone(),
            MessageRole::Assistant,
            "Second turn",
        )
        .expect("message should be valid");

        HistoryRepository::save_session(repository.as_ref(), &session)
            .await
            .expect("session should persist");

        ConversationRepository::append_message(repository.as_ref(), &first)
            .await
            .expect("first message should persist");
        ConversationRepository::append_message(repository.as_ref(), &second)
            .await
            .expect("second message should persist");

        let messages = ConversationRepository::load_by_session(repository.as_ref(), &session_id)
            .await
            .expect("conversation should load");

        assert_eq!(messages, vec![first, second]);
    });
}
