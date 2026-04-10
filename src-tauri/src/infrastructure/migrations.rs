use rusqlite::{Connection, OptionalExtension};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct Migration {
    pub version: &'static str,
    pub sql: &'static str,
}

impl Migration {
    pub const fn new(version: &'static str, sql: &'static str) -> Self {
        Self { version, sql }
    }
}

const MIGRATIONS: &[Migration] = &[
    Migration::new(
        "0001_initial_schema",
        include_str!("../../migrations/0001_initial_schema.sql"),
    ),
    Migration::new(
        "0002_language_settings",
        include_str!("../../migrations/0002_language_settings.sql"),
    ),
    Migration::new(
        "0003_default_ocr_language_eng",
        include_str!("../../migrations/0003_default_ocr_language_eng.sql"),
    ),
    Migration::new(
        "0004_reasoning_effort_settings",
        include_str!("../../migrations/0004_reasoning_effort_settings.sql"),
    ),
];

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("sqlite migration error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

pub fn run_migrations(connection: &Connection) -> Result<Vec<String>, MigrationError> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );",
    )?;

    let mut applied_versions = Vec::new();

    for migration in MIGRATIONS {
        let already_applied = connection
            .query_row(
                "SELECT version FROM schema_migrations WHERE version = ?1",
                [migration.version],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .is_some();

        if already_applied {
            continue;
        }

        connection.execute_batch(migration.sql)?;
        connection.execute(
            "INSERT INTO schema_migrations (version) VALUES (?1)",
            [migration.version],
        )?;
        applied_versions.push(migration.version.to_string());
    }

    Ok(applied_versions)
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::run_migrations;

    #[test]
    fn applies_migrations_once() {
        let connection = Connection::open_in_memory().expect("sqlite memory db should open");

        let first_run = run_migrations(&connection).expect("migrations should apply");
        let second_run = run_migrations(&connection).expect("migrations should be idempotent");

        assert_eq!(
            first_run,
            vec![
                "0001_initial_schema".to_string(),
                "0002_language_settings".to_string(),
                "0003_default_ocr_language_eng".to_string(),
                "0004_reasoning_effort_settings".to_string(),
            ]
        );
        assert!(second_run.is_empty());
    }

    #[test]
    fn creates_core_tables() {
        let connection = Connection::open_in_memory().expect("sqlite memory db should open");

        run_migrations(&connection).expect("migrations should apply");

        let table_count = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN (
                    'user_settings',
                    'configured_providers',
                    'provider_models',
                    'interaction_sessions',
                    'conversation_messages',
                    'captures',
                    'shortcut_bindings'
                )",
                [],
                |row| row.get::<_, i64>(0),
            )
            .expect("table count query should succeed");

        assert_eq!(table_count, 7);
    }
}
