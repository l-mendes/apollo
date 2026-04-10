use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppMetadata {
    pub name: String,
    pub version: String,
    pub database_file_name: String,
}

impl AppMetadata {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            name: "Apollo".to_string(),
            version: version.into(),
            database_file_name: "apollo.db".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppMetadata;

    #[test]
    fn builds_metadata_with_expected_defaults() {
        let metadata = AppMetadata::new("0.1.0");

        assert_eq!(metadata.name, "Apollo");
        assert_eq!(metadata.version, "0.1.0");
        assert_eq!(metadata.database_file_name, "apollo.db");
    }
}
