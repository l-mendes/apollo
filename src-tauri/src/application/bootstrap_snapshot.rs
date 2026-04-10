use serde::Serialize;

use crate::domain::app_metadata::AppMetadata;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BootstrapSnapshot {
    pub metadata: AppMetadata,
    pub database_path: String,
    pub applied_migrations: Vec<String>,
}

impl BootstrapSnapshot {
    pub fn new(
        metadata: AppMetadata,
        database_path: impl Into<String>,
        applied_migrations: Vec<String>,
    ) -> Self {
        Self {
            metadata,
            database_path: database_path.into(),
            applied_migrations,
        }
    }

    #[cfg(test)]
    pub fn database_ready(&self) -> bool {
        !self.database_path.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::BootstrapSnapshot;
    use crate::domain::app_metadata::AppMetadata;

    #[test]
    fn reports_database_ready_when_path_exists() {
        let snapshot = BootstrapSnapshot::new(
            AppMetadata::new("0.1.0"),
            "/tmp/apollo.db",
            vec!["0001_initial_schema".to_string()],
        );

        assert!(snapshot.database_ready());
    }
}
