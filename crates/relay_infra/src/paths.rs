use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;

const QUALIFIER: &str = "dev";
const ORGANIZATION: &str = "Relay";
const APPLICATION: &str = "Relay";

#[derive(Debug, Clone)]
pub struct RelayPaths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl RelayPaths {
    pub fn discover() -> Result<Self> {
        let dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .context("failed to resolve Relay project directories")?;
        let data_dir = dirs.data_dir().to_path_buf();
        let config_dir = dirs.config_dir().to_path_buf();
        let log_dir = data_dir.join("logs");

        Ok(Self {
            data_dir,
            config_dir,
            log_dir,
        })
    }

    pub fn ensure(&self) -> Result<()> {
        std::fs::create_dir_all(&self.data_dir)
            .with_context(|| format!("failed to create data dir {}", self.data_dir.display()))?;
        std::fs::create_dir_all(&self.config_dir).with_context(|| {
            format!("failed to create config dir {}", self.config_dir.display())
        })?;
        std::fs::create_dir_all(&self.log_dir)
            .with_context(|| format!("failed to create log dir {}", self.log_dir.display()))?;
        Ok(())
    }

    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("relay.sqlite3")
    }
}
