use relay_core::{Task, TaskEvent, TaskId, TaskProjection};
use rusqlite::{Connection, OptionalExtension, params};
use time::format_description::well_known::Rfc3339;

use crate::PersistenceResult;

pub struct TaskRepository<'a> {
    connection: &'a mut Connection,
}

impl<'a> TaskRepository<'a> {
    pub(crate) fn new(connection: &'a mut Connection) -> Self {
        Self { connection }
    }

    pub fn append_events(&mut self, events: &[TaskEvent]) -> PersistenceResult<()> {
        if events.is_empty() {
            return Ok(());
        }

        let transaction = self.connection.transaction()?;
        for event in events {
            let task_id = event.task_id().to_string();
            let next_sequence = next_sequence(&transaction, &task_id)?;
            let event_json = serde_json::to_string(event)?;
            let occurred_at = event.occurred_at().format(&Rfc3339)?;
            transaction.execute(
                r#"
                INSERT INTO task_events(task_id, sequence, event_type, event_json, occurred_at)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
                params![
                    task_id,
                    next_sequence,
                    event.event_type(),
                    event_json,
                    occurred_at
                ],
            )?;
        }
        transaction.commit()?;

        Ok(())
    }

    pub fn save_snapshot(&mut self, projection: &TaskProjection) -> PersistenceResult<()> {
        let projection_json = serde_json::to_string(projection)?;
        let updated_at = projection.last_activity_at.format(&Rfc3339)?;
        self.connection.execute(
            r#"
            INSERT INTO task_snapshots(task_id, projection_json, updated_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(task_id) DO UPDATE SET
                projection_json = excluded.projection_json,
                updated_at = excluded.updated_at
            "#,
            params![projection.id.to_string(), projection_json, updated_at],
        )?;

        Ok(())
    }

    pub fn load_snapshot(&self, task_id: TaskId) -> PersistenceResult<Option<TaskProjection>> {
        self.connection
            .query_row(
                "SELECT projection_json FROM task_snapshots WHERE task_id = ?1",
                params![task_id.to_string()],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|json| serde_json::from_str(&json).map_err(Into::into))
            .transpose()
    }

    pub fn load_events(&self, task_id: TaskId) -> PersistenceResult<Vec<TaskEvent>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT event_json
            FROM task_events
            WHERE task_id = ?1
            ORDER BY sequence ASC
            "#,
        )?;
        let rows =
            statement.query_map(params![task_id.to_string()], |row| row.get::<_, String>(0))?;

        let mut events = Vec::new();
        for row in rows {
            events.push(serde_json::from_str(&row?)?);
        }
        Ok(events)
    }

    pub fn load_task(&self, task_id: TaskId) -> PersistenceResult<Option<Task>> {
        let events = self.load_events(task_id)?;
        if events.is_empty() {
            return Ok(None);
        }
        Ok(Some(Task::replay(events.iter())?))
    }

    pub fn list_tasks(&self) -> PersistenceResult<Vec<TaskProjection>> {
        let task_ids = self.task_ids()?;
        let mut projections = Vec::with_capacity(task_ids.len());
        for task_id in task_ids {
            if let Some(task) = self.load_task(task_id)? {
                projections.push(TaskProjection::from_task(&task));
            }
        }
        projections.sort_by(|a, b| b.last_activity_at.cmp(&a.last_activity_at));
        Ok(projections)
    }

    fn task_ids(&self) -> PersistenceResult<Vec<TaskId>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT event_json
            FROM task_events
            WHERE sequence = 0
            ORDER BY occurred_at DESC
            "#,
        )?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;

        let mut ids = Vec::new();
        for row in rows {
            let event: TaskEvent = serde_json::from_str(&row?)?;
            ids.push(event.task_id());
        }
        Ok(ids)
    }
}

fn next_sequence(connection: &Connection, task_id: &str) -> PersistenceResult<i64> {
    let current = connection
        .query_row(
            "SELECT MAX(sequence) FROM task_events WHERE task_id = ?1",
            params![task_id],
            |row| row.get::<_, Option<i64>>(0),
        )?
        .unwrap_or(-1);
    Ok(current + 1)
}

#[cfg(test)]
mod tests {
    use relay_core::{
        AgentKind, AgentRuntimeStatus, AgentSessionId, AgentStatusUpdate, ChangedFile, CreateTask,
        ProjectId, TaskCommand, TaskProjection, TaskSource, TaskStatus, TerminalSessionId,
        WorktreeId, WorktreeSnapshot,
    };
    use tempfile::tempdir;
    use time::OffsetDateTime;

    use crate::RelayDatabase;

    fn now() -> OffsetDateTime {
        OffsetDateTime::UNIX_EPOCH
    }

    #[test]
    fn append_events_should_survive_database_reopen() {
        let dir = tempdir().expect("tempdir should be created");
        let db_path = dir.path().join("relay.sqlite3");
        let task_id;

        {
            let mut database = RelayDatabase::open(&db_path).expect("database should open");
            let mut repository = database.task_repository();
            let (mut task, created_events) = relay_core::Task::create(CreateTask {
                id: None,
                project_id: ProjectId::new(),
                title: "Persist task".to_string(),
                source: TaskSource::Manual,
                now: now(),
            })
            .expect("task should be created");
            task_id = task.id;
            repository
                .append_events(&created_events)
                .expect("created event should persist");

            apply_and_persist(
                &mut repository,
                &mut task,
                TaskCommand::AttachWorktree {
                    snapshot: WorktreeSnapshot {
                        id: WorktreeId::new(),
                        path: "/repo/task".to_string(),
                        branch: "task/persist".to_string(),
                        base_ref: Some("main".to_string()),
                    },
                    now: now(),
                },
            );
            apply_and_persist(
                &mut repository,
                &mut task,
                TaskCommand::AttachTerminal {
                    id: TerminalSessionId::new(),
                    now: now(),
                },
            );
            apply_and_persist(
                &mut repository,
                &mut task,
                TaskCommand::AttachAgent {
                    id: AgentSessionId::new(),
                    kind: AgentKind::Codex,
                    started_at: now(),
                },
            );
            apply_and_persist(
                &mut repository,
                &mut task,
                TaskCommand::ApplyAgentStatus(AgentStatusUpdate {
                    state: AgentRuntimeStatus::Working,
                    prompt: "Persist me".to_string(),
                    agent_kind: Some(AgentKind::Codex),
                    observed_at: now(),
                }),
            );
            apply_and_persist(
                &mut repository,
                &mut task,
                TaskCommand::RefreshChangedFiles {
                    files: vec![ChangedFile {
                        path: "src/lib.rs".to_string(),
                        status: relay_core::ChangeStatus::Modified,
                    }],
                    now: now(),
                },
            );
            repository
                .save_snapshot(&TaskProjection::from_task(&task))
                .expect("snapshot should save");
        }

        let mut reopened = RelayDatabase::open(&db_path).expect("database should reopen");
        let repository = reopened.task_repository();
        let task = repository
            .load_task(task_id)
            .expect("task should load")
            .expect("task should exist");
        let tasks = repository.list_tasks().expect("task list should load");
        let snapshot = repository
            .load_snapshot(task_id)
            .expect("snapshot should load")
            .expect("snapshot should exist");

        assert_eq!(task.status, TaskStatus::Working);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].changed_file_count, 1);
        assert_eq!(snapshot.title, "Persist task");
    }

    fn apply_and_persist(
        repository: &mut crate::TaskRepository<'_>,
        task: &mut relay_core::Task,
        command: TaskCommand,
    ) {
        let events = task.handle(command).expect("command should produce events");
        repository
            .append_events(&events)
            .expect("events should persist");
        for event in &events {
            task.apply(event).expect("event should apply");
        }
    }
}
