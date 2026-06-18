use serde::{Serialize, de::DeserializeOwned};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use rusqlite::{Connection, OptionalExtension, params};

use crate::PersistenceResult;

pub struct SettingsRepository<'a> {
    connection: &'a mut Connection,
}

impl<'a> SettingsRepository<'a> {
    pub(crate) fn new(connection: &'a mut Connection) -> Self {
        Self { connection }
    }

    pub fn set_json<T: Serialize>(
        &mut self,
        namespace: &str,
        key: &str,
        value: &T,
    ) -> PersistenceResult<()> {
        let value_json = serde_json::to_string(value)?;
        let updated_at = OffsetDateTime::now_utc().format(&Rfc3339)?;
        self.connection.execute(
            r#"
            INSERT INTO settings(namespace, key, value_json, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(namespace, key) DO UPDATE SET
                value_json = excluded.value_json,
                updated_at = excluded.updated_at
            "#,
            params![namespace, key, value_json, updated_at],
        )?;
        Ok(())
    }

    pub fn get_json<T: DeserializeOwned>(
        &self,
        namespace: &str,
        key: &str,
    ) -> PersistenceResult<Option<T>> {
        self.connection
            .query_row(
                "SELECT value_json FROM settings WHERE namespace = ?1 AND key = ?2",
                params![namespace, key],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|json| serde_json::from_str(&json).map_err(Into::into))
            .transpose()
    }

    pub fn delete(&mut self, namespace: &str, key: &str) -> PersistenceResult<()> {
        self.connection.execute(
            "DELETE FROM settings WHERE namespace = ?1 AND key = ?2",
            params![namespace, key],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::RelayDatabase;

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct UiSettings {
        theme: String,
        compact: bool,
    }

    #[test]
    fn settings_should_round_trip_json_by_namespace() {
        let mut database = RelayDatabase::in_memory().expect("database should open");
        let mut repository = database.settings_repository();

        repository
            .set_json(
                "ui",
                "workbench",
                &UiSettings {
                    theme: "dark".to_string(),
                    compact: true,
                },
            )
            .expect("setting should save");

        let loaded: UiSettings = repository
            .get_json("ui", "workbench")
            .expect("setting should load")
            .expect("setting should exist");
        let missing: Option<UiSettings> = repository
            .get_json("agent", "workbench")
            .expect("missing setting should load as none");

        assert_eq!(
            loaded,
            UiSettings {
                theme: "dark".to_string(),
                compact: true,
            }
        );
        assert_eq!(missing, None);
    }
}
