use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use rusqlite::{Connection, OptionalExtension, params};

use crate::{
    application::{
        errors::{ApplicationError, ApplicationErrorKind},
        ports::repositories::{
            ConversationRepository, HistoryRepository, SettingsRepository, ShortcutRepository,
        },
    },
    domain::{
        entities::{
            conversation_message::ConversationMessage, interaction_session::InteractionSession,
            shortcut_binding::ShortcutBinding, user_settings::UserSettings,
        },
        value_objects::{
            identifiers::{MessageId, SessionId},
            model_key::ModelKey,
            reasoning_effort::ReasoningEffort,
            shortcut::{ShortcutAccelerator, ShortcutAction},
        },
    },
    infrastructure::migrations::run_migrations,
};

pub struct SqliteAppRepository {
    database_path: PathBuf,
    cleanup_on_drop: bool,
}

impl SqliteAppRepository {
    pub fn new(database_path: PathBuf) -> Self {
        let repository = Self {
            database_path,
            cleanup_on_drop: false,
        };
        repository
            .initialize()
            .expect("sqlite repository should initialize");
        repository
    }

    pub fn in_memory() -> Result<Self, ApplicationError> {
        let database_path = std::env::temp_dir().join(format!(
            "apollo-temp-{}.db",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        let repository = Self {
            database_path,
            cleanup_on_drop: true,
        };
        repository.initialize()?;
        Ok(repository)
    }

    fn initialize(&self) -> Result<(), ApplicationError> {
        let connection = self.open_connection()?;
        run_migrations(&connection).map_err(map_sqlite_error)?;
        Ok(())
    }

    fn open_connection(&self) -> Result<Connection, ApplicationError> {
        let connection = Connection::open(&self.database_path).map_err(map_sqlite_error)?;
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(map_sqlite_error)?;
        Ok(connection)
    }
}

impl Drop for SqliteAppRepository {
    fn drop(&mut self) {
        if self.cleanup_on_drop {
            let _ = fs::remove_file(&self.database_path);
        }
    }
}

#[async_trait]
impl SettingsRepository for SqliteAppRepository {
    async fn load(&self) -> Result<UserSettings, ApplicationError> {
        let connection = self.open_connection()?;
        let row = connection
            .query_row(
                "SELECT preferred_provider_id, preferred_model_id, base_prompt, ocr_language, output_language, reasoning_effort FROM user_settings WHERE id = 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, Option<String>>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                    ))
                },
            )
            .optional()
            .map_err(map_sqlite_error)?;

        let shortcuts = load_shortcuts_from_connection(&connection)?;
        let defaults = UserSettings::default();

        match row {
            Some((
                preferred_provider,
                preferred_model,
                base_prompt,
                ocr_language,
                output_language,
                reasoning_effort,
            )) => Ok(UserSettings {
                preferred_provider: preferred_provider
                    .as_deref()
                    .unwrap_or(defaults.preferred_provider.as_str())
                    .parse()
                    .map_err(invalid_configuration)?,
                preferred_model: ModelKey::new(
                    preferred_model
                        .unwrap_or_else(|| defaults.preferred_model.as_str().to_string()),
                )?,
                reasoning_effort: reasoning_effort
                    .as_deref()
                    .unwrap_or(defaults.reasoning_effort.as_str())
                    .parse::<ReasoningEffort>()
                    .map_err(invalid_configuration)?,
                base_prompt,
                ocr_language: ocr_language.unwrap_or(defaults.ocr_language),
                output_language: output_language.unwrap_or(defaults.output_language),
                shortcuts: if shortcuts.is_empty() {
                    defaults.shortcuts
                } else {
                    shortcuts
                },
            }),
            None => Ok(defaults),
        }
    }

    async fn save(&self, settings: &UserSettings) -> Result<(), ApplicationError> {
        let mut connection = self.open_connection()?;
        let transaction = connection.transaction().map_err(map_sqlite_error)?;

        transaction
            .execute(
                "INSERT INTO user_settings (
                    id,
                    base_prompt,
                    preferred_provider_id,
                    preferred_model_id,
                    ocr_language,
                    output_language,
                    reasoning_effort,
                    updated_at
                ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
                ON CONFLICT(id) DO UPDATE SET
                    base_prompt = excluded.base_prompt,
                    preferred_provider_id = excluded.preferred_provider_id,
                    preferred_model_id = excluded.preferred_model_id,
                    ocr_language = excluded.ocr_language,
                    output_language = excluded.output_language,
                    reasoning_effort = excluded.reasoning_effort,
                    updated_at = CURRENT_TIMESTAMP",
                params![
                    settings.base_prompt.as_str(),
                    settings.preferred_provider.as_str(),
                    settings.preferred_model.as_str(),
                    settings.ocr_language.as_str(),
                    settings.output_language.as_str(),
                    settings.reasoning_effort.as_str(),
                ],
            )
            .map_err(map_sqlite_error)?;

        replace_shortcuts_in_transaction(&transaction, &settings.shortcuts)?;
        transaction.commit().map_err(map_sqlite_error)?;

        Ok(())
    }
}

#[async_trait]
impl ShortcutRepository for SqliteAppRepository {
    async fn list(&self) -> Result<Vec<ShortcutBinding>, ApplicationError> {
        let connection = self.open_connection()?;
        load_shortcuts_from_connection(&connection)
    }

    async fn replace_all(&self, shortcuts: &[ShortcutBinding]) -> Result<(), ApplicationError> {
        let mut connection = self.open_connection()?;
        let transaction = connection.transaction().map_err(map_sqlite_error)?;

        replace_shortcuts_in_transaction(&transaction, shortcuts)?;
        transaction.commit().map_err(map_sqlite_error)?;

        Ok(())
    }
}

#[async_trait]
impl HistoryRepository for SqliteAppRepository {
    async fn save_session(&self, session: &InteractionSession) -> Result<(), ApplicationError> {
        let connection = self.open_connection()?;

        connection
            .execute(
                "INSERT INTO interaction_sessions (
                    id,
                    provider_id,
                    model_id,
                    source_kind,
                    ocr_text,
                    user_notes,
                    request_prompt,
                    response_text
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                ON CONFLICT(id) DO UPDATE SET
                    provider_id = excluded.provider_id,
                    model_id = excluded.model_id,
                    source_kind = excluded.source_kind,
                    ocr_text = excluded.ocr_text,
                    user_notes = excluded.user_notes,
                    request_prompt = excluded.request_prompt,
                    response_text = excluded.response_text",
                params![
                    session.id.as_str(),
                    session.provider_kind.as_str(),
                    session.model_key.as_str(),
                    session.source_kind.as_str(),
                    session.ocr_text.as_deref(),
                    session.user_notes.as_deref(),
                    session.request_prompt.as_deref(),
                    session.response_text.as_deref(),
                ],
            )
            .map_err(map_sqlite_error)?;

        Ok(())
    }

    async fn list_sessions(&self) -> Result<Vec<InteractionSession>, ApplicationError> {
        let connection = self.open_connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, provider_id, model_id, source_kind, ocr_text, user_notes, request_prompt, response_text
                FROM interaction_sessions
                ORDER BY created_at DESC, rowid DESC",
            )
            .map_err(map_sqlite_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                ))
            })
            .map_err(map_sqlite_error)?;

        rows.into_iter()
            .map(|row| {
                let (
                    id,
                    provider_id,
                    model_id,
                    source_kind,
                    ocr_text,
                    user_notes,
                    request_prompt,
                    response_text,
                ) = row.map_err(map_sqlite_error)?;

                Ok(InteractionSession {
                    id: SessionId::new(id)?,
                    provider_kind: provider_id.parse().map_err(invalid_configuration)?,
                    model_key: ModelKey::new(model_id)?,
                    source_kind: source_kind.parse().map_err(invalid_configuration)?,
                    ocr_text,
                    user_notes,
                    request_prompt,
                    response_text,
                })
            })
            .collect()
    }

    async fn delete_session(&self, session_id: &SessionId) -> Result<(), ApplicationError> {
        let mut connection = self.open_connection()?;
        let transaction = connection.transaction().map_err(map_sqlite_error)?;

        transaction
            .execute(
                "DELETE FROM conversation_messages WHERE session_id = ?1",
                params![session_id.as_str()],
            )
            .map_err(map_sqlite_error)?;
        transaction
            .execute(
                "DELETE FROM interaction_sessions WHERE id = ?1",
                params![session_id.as_str()],
            )
            .map_err(map_sqlite_error)?;
        transaction.commit().map_err(map_sqlite_error)?;

        Ok(())
    }

    async fn clear_sessions(&self) -> Result<(), ApplicationError> {
        let mut connection = self.open_connection()?;
        let transaction = connection.transaction().map_err(map_sqlite_error)?;

        transaction
            .execute("DELETE FROM conversation_messages", [])
            .map_err(map_sqlite_error)?;
        transaction
            .execute("DELETE FROM interaction_sessions", [])
            .map_err(map_sqlite_error)?;
        transaction.commit().map_err(map_sqlite_error)?;

        Ok(())
    }
}

#[async_trait]
impl ConversationRepository for SqliteAppRepository {
    async fn append_message(&self, message: &ConversationMessage) -> Result<(), ApplicationError> {
        let connection = self.open_connection()?;

        connection
            .execute(
                "INSERT INTO conversation_messages (id, session_id, role, content)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(id) DO UPDATE SET
                    session_id = excluded.session_id,
                    role = excluded.role,
                    content = excluded.content",
                params![
                    message.id.as_str(),
                    message.session_id.as_str(),
                    message.role.as_str(),
                    message.content.as_str(),
                ],
            )
            .map_err(map_sqlite_error)?;

        Ok(())
    }

    async fn load_by_session(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<ConversationMessage>, ApplicationError> {
        let connection = self.open_connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, role, content
                FROM conversation_messages
                WHERE session_id = ?1
                ORDER BY created_at ASC, rowid ASC",
            )
            .map_err(map_sqlite_error)?;
        let rows = statement
            .query_map([session_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(map_sqlite_error)?;

        rows.into_iter()
            .map(|row| {
                let (id, role, content) = row.map_err(map_sqlite_error)?;

                Ok(ConversationMessage::new(
                    MessageId::new(id)?,
                    session_id.clone(),
                    role.parse().map_err(invalid_configuration)?,
                    content,
                )?)
            })
            .collect()
    }
}

fn load_shortcuts_from_connection(
    connection: &Connection,
) -> Result<Vec<ShortcutBinding>, ApplicationError> {
    let mut statement = connection
        .prepare(
            "SELECT action_key, accelerator, is_enabled
            FROM shortcut_bindings
            ORDER BY created_at ASC, rowid ASC",
        )
        .map_err(map_sqlite_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(map_sqlite_error)?;

    rows.into_iter()
        .map(|row| {
            let (action_key, accelerator, is_enabled) = row.map_err(map_sqlite_error)?;

            Ok(ShortcutBinding {
                action: ShortcutAction::new(action_key)?,
                accelerator: ShortcutAccelerator::new(accelerator)?,
                enabled: is_enabled != 0,
            })
        })
        .collect()
}

fn replace_shortcuts_in_transaction(
    transaction: &rusqlite::Transaction<'_>,
    shortcuts: &[ShortcutBinding],
) -> Result<(), ApplicationError> {
    transaction
        .execute("DELETE FROM shortcut_bindings", [])
        .map_err(map_sqlite_error)?;

    for shortcut in shortcuts {
        transaction
            .execute(
                "INSERT INTO shortcut_bindings (
                    id,
                    action_key,
                    accelerator,
                    is_enabled,
                    updated_at
                ) VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)",
                params![
                    shortcut.action.as_str(),
                    shortcut.action.as_str(),
                    shortcut.accelerator.as_str(),
                    if shortcut.enabled { 1_i64 } else { 0_i64 },
                ],
            )
            .map_err(map_sqlite_error)?;
    }

    Ok(())
}

fn map_sqlite_error(error: impl ToString) -> ApplicationError {
    ApplicationError::new(ApplicationErrorKind::Unavailable, error.to_string())
}

fn invalid_configuration(error: impl ToString) -> ApplicationError {
    ApplicationError::new(
        ApplicationErrorKind::InvalidConfiguration,
        error.to_string(),
    )
}

#[allow(dead_code)]
fn database_exists(database_path: &Path) -> bool {
    database_path.exists()
}
