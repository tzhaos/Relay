mod migrations;
mod settings_repository;
mod task_repository;

use std::path::{Path, PathBuf};

use rusqlite::Connection;

pub use settings_repository::SettingsRepository;
pub use task_repository::TaskRepository;

pub type PersistenceResult<T> = Result<T, PersistenceError>;

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("sqlite error")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json error")]
    Json(#[from] serde_json::Error),
    #[error("time formatting error")]
    TimeFormat(#[from] time::error::Format),
    #[error("task replay error")]
    Task(#[from] relay_core::TaskError),
    #[error("migration changed at step {step}: {name}")]
    MigrationChanged { step: i64, name: String },
}

pub struct RelayDatabase {
    connection: Connection,
    path: Option<PathBuf>,
}

impl RelayDatabase {
    pub fn open(path: impl AsRef<Path>) -> PersistenceResult<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
        }

        let mut connection = Connection::open(path)?;
        migrations::run(&mut connection)?;
        Ok(Self {
            connection,
            path: Some(path.to_path_buf()),
        })
    }

    pub fn in_memory() -> PersistenceResult<Self> {
        let mut connection = Connection::open_in_memory()?;
        migrations::run(&mut connection)?;
        Ok(Self {
            connection,
            path: None,
        })
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn task_repository(&mut self) -> TaskRepository<'_> {
        TaskRepository::new(&mut self.connection)
    }

    pub fn settings_repository(&mut self) -> SettingsRepository<'_> {
        SettingsRepository::new(&mut self.connection)
    }
}
