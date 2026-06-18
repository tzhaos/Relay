use rusqlite::{Connection, params};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{PersistenceError, PersistenceResult};

struct Migration {
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    name: "0001_task_events_snapshots_settings",
    sql: r#"
        CREATE TABLE task_events (
            task_id TEXT NOT NULL,
            sequence INTEGER NOT NULL,
            event_type TEXT NOT NULL,
            event_json TEXT NOT NULL,
            occurred_at TEXT NOT NULL,
            PRIMARY KEY (task_id, sequence)
        );

        CREATE INDEX task_events_task_occurred_at_idx
            ON task_events(task_id, occurred_at);

        CREATE TABLE task_snapshots (
            task_id TEXT PRIMARY KEY,
            projection_json TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE settings (
            namespace TEXT NOT NULL,
            key TEXT NOT NULL,
            value_json TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            PRIMARY KEY (namespace, key)
        );
    "#,
}];

pub fn run(connection: &mut Connection) -> PersistenceResult<()> {
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "busy_timeout", 500_i64)?;
    connection.pragma_update(None, "foreign_keys", "ON")?;

    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            step INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            statement TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );
        "#,
    )?;

    let transaction = connection.transaction()?;
    for (step, migration) in MIGRATIONS.iter().enumerate() {
        let step = step as i64;
        let existing = transaction.query_row(
            "SELECT name, statement FROM schema_migrations WHERE step = ?1",
            params![step],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        );

        match existing {
            Ok((name, statement)) if name == migration.name && statement == migration.sql => {}
            Ok((name, _)) => {
                return Err(PersistenceError::MigrationChanged { step, name });
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                transaction.execute_batch(migration.sql)?;
                let applied_at = OffsetDateTime::now_utc().format(&Rfc3339)?;
                transaction.execute(
                    "INSERT INTO schema_migrations(step, name, statement, applied_at) VALUES (?1, ?2, ?3, ?4)",
                    params![step, migration.name, migration.sql, applied_at],
                )?;
            }
            Err(error) => return Err(error.into()),
        }
    }
    transaction.commit()?;

    Ok(())
}
