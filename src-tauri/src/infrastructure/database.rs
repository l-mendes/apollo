use std::path::Path;

use rusqlite::Connection;
use thiserror::Error;

use crate::{
    application::bootstrap_snapshot::BootstrapSnapshot,
    domain::app_metadata::AppMetadata,
    infrastructure::{migrations::run_migrations, paths::ApolloPaths},
};

#[derive(Debug, Error)]
pub enum DatabaseBootstrapError {
    #[error(transparent)]
    Paths(#[from] crate::infrastructure::paths::PathsError),
    #[error("failed to open sqlite database: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Migration(#[from] crate::infrastructure::migrations::MigrationError),
}

pub fn bootstrap_database(
    app_data_root: &Path,
    metadata: &AppMetadata,
) -> Result<BootstrapSnapshot, DatabaseBootstrapError> {
    let paths = ApolloPaths::new(app_data_root, metadata);
    paths.ensure()?;

    let connection = Connection::open(&paths.database_path)?;
    let applied_migrations = run_migrations(&connection)?;

    Ok(BootstrapSnapshot::new(
        metadata.clone(),
        paths.database_path.to_string_lossy().to_string(),
        applied_migrations,
    ))
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::bootstrap_database;
    use crate::domain::app_metadata::AppMetadata;

    #[test]
    fn bootstraps_database_file_on_disk() {
        let temp_dir = tempdir().expect("temporary directory should exist");

        let snapshot = bootstrap_database(temp_dir.path(), &AppMetadata::new("0.1.0"))
            .expect("database should bootstrap");

        assert!(temp_dir.path().join("apollo.db").exists());
        assert!(snapshot.database_ready());
        assert_eq!(snapshot.applied_migrations, vec!["0001_initial_schema", "0002_language_settings", "0003_default_ocr_language_eng"]);
    }
}
