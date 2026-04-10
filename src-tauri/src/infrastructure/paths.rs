use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::domain::app_metadata::AppMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApolloPaths {
    pub app_data_dir: PathBuf,
    pub captures_dir: PathBuf,
    pub database_path: PathBuf,
}

#[derive(Debug, Error)]
pub enum PathsError {
    #[error("failed to create Apollo directories: {0}")]
    Io(#[from] std::io::Error),
}

impl ApolloPaths {
    pub fn new(root: &Path, metadata: &AppMetadata) -> Self {
        let captures_dir = root.join("captures");

        Self {
            app_data_dir: root.to_path_buf(),
            captures_dir,
            database_path: root.join(&metadata.database_file_name),
        }
    }

    pub fn ensure(&self) -> Result<(), PathsError> {
        fs::create_dir_all(&self.app_data_dir)?;
        fs::create_dir_all(&self.captures_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::ApolloPaths;
    use crate::domain::app_metadata::AppMetadata;

    #[test]
    fn creates_expected_directories() {
        let temp_dir = tempdir().expect("temporary directory should exist");
        let paths = ApolloPaths::new(temp_dir.path(), &AppMetadata::new("0.1.0"));

        paths.ensure().expect("paths should be created");

        assert!(paths.app_data_dir.exists());
        assert!(paths.captures_dir.exists());
    }
}
