#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use gpui::{App, Application};
use relay_core::{CreateTask, ProjectId, Task, TaskProjection, TaskSource};
use relay_infra::{logging, paths::RelayPaths};
use relay_persistence::RelayDatabase;
use relay_ui::{AppShell, app_shell::TaskDataSource};
use time::OffsetDateTime;

struct SqliteTaskDataSource {
    database: RelayDatabase,
    project_id: ProjectId,
}

impl TaskDataSource for SqliteTaskDataSource {
    fn create_task(&mut self, title: &str) -> anyhow::Result<Vec<TaskProjection>> {
        let now = OffsetDateTime::now_utc();
        let (task, events) = Task::create(CreateTask {
            id: None,
            project_id: self.project_id,
            title: title.to_string(),
            source: TaskSource::Manual,
            now,
        })?;

        let projection = TaskProjection::from_task(&task);
        let mut repository = self.database.task_repository();
        repository.append_events(&events)?;
        repository.save_snapshot(&projection)?;
        Ok(repository.list_tasks()?)
    }
}

fn main() -> Result<()> {
    let paths = RelayPaths::discover()?;
    let _logging_guard = logging::init(&paths)?;
    let mut database = RelayDatabase::open(paths.database_path())?;
    let project_id = load_or_create_project_id(&mut database)?;
    let tasks = database.task_repository().list_tasks()?;
    let mut task_data_source = Some(Box::new(SqliteTaskDataSource {
        database,
        project_id,
    }) as Box<dyn TaskDataSource>);

    tracing::info!("starting Relay");
    Application::new().run(move |cx: &mut App| {
        let task_data_source = task_data_source
            .take()
            .expect("Relay window should open once");
        if let Err(error) = AppShell::open(cx, tasks, task_data_source) {
            tracing::error!(?error, "failed to open Relay window");
        }
    });

    Ok(())
}

fn load_or_create_project_id(database: &mut RelayDatabase) -> Result<ProjectId> {
    if let Some(project_id) = database
        .settings_repository()
        .get_json("workspace", "default_project_id")?
    {
        return Ok(project_id);
    }

    let project_id = ProjectId::new();
    database
        .settings_repository()
        .set_json("workspace", "default_project_id", &project_id)?;
    Ok(project_id)
}
