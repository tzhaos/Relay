use std::{path::PathBuf, time::Duration};

use anyhow::{Context as _, Result};
use relay_core::{
    CreateTask, ProjectId, ProviderFailure, Task, TaskCommand, TaskProjection, TaskSource,
    TaskStatus, TerminalSessionId,
};
use relay_infra::paths::RelayPaths;
use relay_persistence::RelayDatabase;
use relay_project::{CreateTaskWorktree, Project, ProjectService};
use relay_terminal::{PtyProvider, TerminalError, TerminalRuntime, TerminalSpawn};
use relay_ui::{app_shell::TaskDataSource, terminal_pane::TerminalPaneProjection};
use time::OffsetDateTime;

const WORKSPACE_NAMESPACE: &str = "workspace";
const DEFAULT_PROJECT_ID_KEY: &str = "default_project_id";
const DEFAULT_PROJECT_ROOT_KEY: &str = "default_project_root";
const TASK_BRANCH_PREFIX: &str = "relay";
const TERMINAL_COLS: u16 = 120;
const TERMINAL_ROWS: u16 = 32;
const TERMINAL_SCROLLBACK_LIMIT: usize = 128 * 1024;

pub struct RelayRuntime {
    database: RelayDatabase,
    project_service: ProjectService,
    terminal_runtime: TerminalRuntime,
    project: Project,
    worktrees_dir: PathBuf,
}

impl RelayRuntime {
    pub fn open(mut database: RelayDatabase, paths: &RelayPaths) -> Result<Self> {
        let project_service = ProjectService::default();
        let project_id = load_or_create_project_id(&mut database)?;
        let project = load_or_open_project(&mut database, &project_service, project_id)?;
        let worktrees_dir = paths
            .data_dir
            .join("worktrees")
            .join(project.id.to_string());

        let mut runtime = Self {
            database,
            project_service,
            terminal_runtime: TerminalRuntime::new(),
            project,
            worktrees_dir,
        };
        runtime.restore_terminal_sessions()?;
        Ok(runtime)
    }

    pub fn project_label(&self) -> &str {
        &self.project.display_name
    }

    pub fn load_tasks(&mut self) -> Result<Vec<TaskProjection>> {
        self.refresh_changed_files()?;
        Ok(self.database.task_repository().list_tasks()?)
    }

    fn create_task_with_worktree(&mut self, title: &str) -> Result<Vec<TaskProjection>> {
        let now = OffsetDateTime::now_utc();
        let (mut task, mut events) = Task::create(CreateTask {
            id: None,
            project_id: self.project.id,
            title: title.to_string(),
            source: TaskSource::Manual,
            now,
        })?;

        let snapshot = self.project_service.create_task_worktree(
            &self.project,
            &CreateTaskWorktree {
                task_title: worktree_title(&task),
                worktrees_dir: self.worktrees_dir.clone(),
                branch_prefix: TASK_BRANCH_PREFIX.to_string(),
            },
        )?;
        let worktree_events = task.handle(TaskCommand::AttachWorktree {
            snapshot,
            now: OffsetDateTime::now_utc(),
        })?;
        apply_events(&mut task, &worktree_events)?;
        events.extend(worktree_events);

        match self.spawn_task_terminal(&task) {
            Ok(terminal_session_id) => {
                let terminal_events = task.handle(TaskCommand::AttachTerminal {
                    id: terminal_session_id,
                    now: OffsetDateTime::now_utc(),
                })?;
                apply_events(&mut task, &terminal_events)?;
                events.extend(terminal_events);
            }
            Err(error) => {
                let failure_events = task.handle(TaskCommand::MarkFailed {
                    failure: ProviderFailure {
                        provider: Some("terminal".to_string()),
                        message: error.to_string(),
                    },
                    now: OffsetDateTime::now_utc(),
                })?;
                apply_events(&mut task, &failure_events)?;
                events.extend(failure_events);
            }
        }

        let projection = TaskProjection::from_task(&task);
        let mut repository = self.database.task_repository();
        repository.append_events(&events)?;
        repository.save_snapshot(&projection)?;
        Ok(repository.list_tasks()?)
    }

    fn refresh_changed_files(&mut self) -> Result<()> {
        let task_ids = self
            .database
            .task_repository()
            .list_tasks()?
            .into_iter()
            .map(|task| task.id)
            .collect::<Vec<_>>();

        for task_id in task_ids {
            let Some(mut task) = self.database.task_repository().load_task(task_id)? else {
                continue;
            };
            if matches!(task.status, TaskStatus::Archived | TaskStatus::Failed) {
                continue;
            }
            let Some(worktree_path) = task
                .worktree
                .as_ref()
                .map(|worktree| PathBuf::from(&worktree.path))
            else {
                continue;
            };
            if !worktree_path.exists() {
                continue;
            }

            let files = self.project_service.changed_files(&worktree_path)?;
            if files == task.changed_files {
                continue;
            }

            let events = task.handle(TaskCommand::RefreshChangedFiles {
                files,
                now: OffsetDateTime::now_utc(),
            })?;
            apply_events(&mut task, &events)?;
            let projection = TaskProjection::from_task(&task);
            let mut repository = self.database.task_repository();
            repository.append_events(&events)?;
            repository.save_snapshot(&projection)?;
        }

        Ok(())
    }

    fn restore_terminal_sessions(&mut self) -> Result<()> {
        let task_ids = self
            .database
            .task_repository()
            .list_tasks()?
            .into_iter()
            .map(|task| task.id)
            .collect::<Vec<_>>();

        for task_id in task_ids {
            let Some(task) = self.database.task_repository().load_task(task_id)? else {
                continue;
            };
            if matches!(task.status, TaskStatus::Archived | TaskStatus::Failed) {
                continue;
            }
            let Some(session_id) = task.terminal_session_id else {
                continue;
            };
            if self.terminal_runtime.has_session(session_id) {
                continue;
            }
            let Some(worktree_path) = task
                .worktree
                .as_ref()
                .map(|worktree| PathBuf::from(&worktree.path))
            else {
                continue;
            };
            if worktree_path.exists()
                && let Err(error) = self.spawn_terminal_with_id(session_id, worktree_path)
            {
                tracing::warn!(
                    ?error,
                    session_id = %session_id,
                    "failed to restore terminal session"
                );
            }
        }

        Ok(())
    }

    fn spawn_task_terminal(&mut self, task: &Task) -> Result<TerminalSessionId> {
        let worktree_path = task
            .worktree
            .as_ref()
            .map(|worktree| PathBuf::from(&worktree.path))
            .context("task has no worktree for terminal")?;

        Ok(self.terminal_runtime.spawn(shell_spawn(worktree_path))?)
    }

    fn spawn_terminal_with_id(
        &mut self,
        session_id: TerminalSessionId,
        worktree_path: PathBuf,
    ) -> Result<TerminalSessionId> {
        Ok(self
            .terminal_runtime
            .spawn_with_id(session_id, shell_spawn(worktree_path))?)
    }

    fn drain_terminal_events(&mut self) -> bool {
        let mut changed = false;
        while self.terminal_runtime.poll_event(Duration::ZERO).is_some() {
            changed = true;
        }
        changed
    }

    fn terminal_projection_for(
        &mut self,
        session_id: TerminalSessionId,
    ) -> Result<Option<TerminalPaneProjection>> {
        self.drain_terminal_events();
        match self.terminal_runtime.snapshot(session_id) {
            Ok(snapshot) => Ok(Some(TerminalPaneProjection {
                session_id: Some(snapshot.session_id),
                cwd: snapshot.cwd.to_string_lossy().to_string(),
                title: snapshot.title,
                scrollback: snapshot.scrollback,
                exited: snapshot.exited,
                connected: true,
            })),
            Err(TerminalError::MissingSession(_)) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}

impl TaskDataSource for RelayRuntime {
    fn create_task(&mut self, title: &str) -> Result<Vec<TaskProjection>> {
        self.create_task_with_worktree(title)
    }

    fn poll_runtime(&mut self) -> Result<bool> {
        Ok(self.drain_terminal_events())
    }

    fn terminal_projection(
        &mut self,
        session_id: TerminalSessionId,
    ) -> Result<Option<TerminalPaneProjection>> {
        self.terminal_projection_for(session_id)
    }
}

fn load_or_create_project_id(database: &mut RelayDatabase) -> Result<ProjectId> {
    if let Some(project_id) = database
        .settings_repository()
        .get_json(WORKSPACE_NAMESPACE, DEFAULT_PROJECT_ID_KEY)?
    {
        return Ok(project_id);
    }

    let project_id = ProjectId::new();
    database.settings_repository().set_json(
        WORKSPACE_NAMESPACE,
        DEFAULT_PROJECT_ID_KEY,
        &project_id,
    )?;
    Ok(project_id)
}

fn load_or_open_project(
    database: &mut RelayDatabase,
    project_service: &ProjectService,
    project_id: ProjectId,
) -> Result<Project> {
    let stored_root = database
        .settings_repository()
        .get_json::<PathBuf>(WORKSPACE_NAMESPACE, DEFAULT_PROJECT_ROOT_KEY)?;
    let current_dir = std::env::current_dir().context("failed to read current directory")?;
    let candidate = stored_root.as_deref().unwrap_or(&current_dir);

    let mut project = match project_service.open_repo(candidate) {
        Ok(project) => project,
        Err(error) if stored_root.is_some() => {
            project_service.open_repo(&current_dir).with_context(|| {
                format!(
                    "stored project root {} could not be opened: {error}",
                    candidate.display()
                )
            })?
        }
        Err(error) => return Err(error).context("failed to open current directory as git repo"),
    };
    project.id = project_id;
    database.settings_repository().set_json(
        WORKSPACE_NAMESPACE,
        DEFAULT_PROJECT_ROOT_KEY,
        &project.root,
    )?;

    Ok(project)
}

fn apply_events(task: &mut Task, events: &[relay_core::TaskEvent]) -> Result<()> {
    for event in events {
        task.apply(event)?;
    }
    Ok(())
}

fn worktree_title(task: &Task) -> String {
    let suffix = task.id.to_string().chars().take(8).collect::<String>();
    format!("{} {}", task.title, suffix)
}

fn shell_spawn(cwd: PathBuf) -> TerminalSpawn {
    TerminalSpawn {
        cwd,
        program: default_shell_program(),
        args: default_shell_args(),
        env: Vec::new(),
        cols: TERMINAL_COLS,
        rows: TERMINAL_ROWS,
        scrollback_limit: TERMINAL_SCROLLBACK_LIMIT,
    }
}

#[cfg(windows)]
fn default_shell_program() -> String {
    std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
}

#[cfg(windows)]
fn default_shell_args() -> Vec<String> {
    vec!["/Q".to_string(), "/K".to_string()]
}

#[cfg(not(windows))]
fn default_shell_program() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string())
}

#[cfg(not(windows))]
fn default_shell_args() -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path, process::Command};

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn runtime_should_create_task_with_real_worktree() {
        let temp = tempdir().expect("tempdir should exist");
        let repo = init_git_repo(temp.path().join("repo"));
        let paths = RelayPaths {
            data_dir: temp.path().join("data"),
            config_dir: temp.path().join("config"),
            log_dir: temp.path().join("logs"),
        };
        let mut database =
            RelayDatabase::open(temp.path().join("relay.sqlite3")).expect("database should open");
        database
            .settings_repository()
            .set_json(WORKSPACE_NAMESPACE, DEFAULT_PROJECT_ROOT_KEY, &repo)
            .expect("project root should save");
        let mut runtime = RelayRuntime::open(database, &paths).expect("runtime should open");

        let tasks = runtime
            .create_task_with_worktree("Implement runtime")
            .expect("task should create");

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::ReadyForAgent);
        assert!(tasks[0].has_terminal);
        let worktree_path = tasks[0]
            .worktree_path
            .as_ref()
            .map(PathBuf::from)
            .expect("worktree path should exist");
        assert!(worktree_path.exists());
    }

    #[test]
    fn load_tasks_should_refresh_changed_files_from_worktree() {
        let temp = tempdir().expect("tempdir should exist");
        let repo = init_git_repo(temp.path().join("repo"));
        let paths = RelayPaths {
            data_dir: temp.path().join("data"),
            config_dir: temp.path().join("config"),
            log_dir: temp.path().join("logs"),
        };
        let mut database =
            RelayDatabase::open(temp.path().join("relay.sqlite3")).expect("database should open");
        database
            .settings_repository()
            .set_json(WORKSPACE_NAMESPACE, DEFAULT_PROJECT_ROOT_KEY, &repo)
            .expect("project root should save");
        let mut runtime = RelayRuntime::open(database, &paths).expect("runtime should open");
        let tasks = runtime
            .create_task_with_worktree("Refresh files")
            .expect("task should create");
        let worktree_path = tasks[0]
            .worktree_path
            .as_ref()
            .map(PathBuf::from)
            .expect("worktree path should exist");
        fs::write(worktree_path.join("changed.txt"), "changed\n").expect("file should write");

        let tasks = runtime.load_tasks().expect("tasks should load");

        assert_eq!(tasks[0].changed_file_count, 1);
    }

    #[test]
    fn terminal_projection_should_use_live_pty_session() {
        let temp = tempdir().expect("tempdir should exist");
        let repo = init_git_repo(temp.path().join("repo"));
        let paths = RelayPaths {
            data_dir: temp.path().join("data"),
            config_dir: temp.path().join("config"),
            log_dir: temp.path().join("logs"),
        };
        let mut database =
            RelayDatabase::open(temp.path().join("relay.sqlite3")).expect("database should open");
        database
            .settings_repository()
            .set_json(WORKSPACE_NAMESPACE, DEFAULT_PROJECT_ROOT_KEY, &repo)
            .expect("project root should save");
        let mut runtime = RelayRuntime::open(database, &paths).expect("runtime should open");
        let tasks = runtime
            .create_task_with_worktree("Connect terminal")
            .expect("task should create");
        let session_id = tasks[0]
            .terminal_session_id
            .expect("terminal id should be attached");

        let projection = runtime
            .terminal_projection_for(session_id)
            .expect("terminal projection should load")
            .expect("terminal should be live");

        assert!(projection.connected);
    }

    #[test]
    fn runtime_open_should_restore_terminal_sessions_for_existing_tasks() {
        let temp = tempdir().expect("tempdir should exist");
        let repo = init_git_repo(temp.path().join("repo"));
        let paths = RelayPaths {
            data_dir: temp.path().join("data"),
            config_dir: temp.path().join("config"),
            log_dir: temp.path().join("logs"),
        };
        let database_path = temp.path().join("relay.sqlite3");
        let mut database = RelayDatabase::open(&database_path).expect("database should open");
        database
            .settings_repository()
            .set_json(WORKSPACE_NAMESPACE, DEFAULT_PROJECT_ROOT_KEY, &repo)
            .expect("project root should save");
        let session_id = {
            let mut runtime = RelayRuntime::open(database, &paths).expect("runtime should open");
            runtime
                .create_task_with_worktree("Restore terminal")
                .expect("task should create")[0]
                .terminal_session_id
                .expect("terminal id should be attached")
        };
        let database = RelayDatabase::open(&database_path).expect("database should reopen");
        let mut runtime = RelayRuntime::open(database, &paths).expect("runtime should reopen");

        let projection = runtime
            .terminal_projection_for(session_id)
            .expect("terminal projection should load")
            .expect("terminal should be restored");

        assert!(projection.connected);
    }

    fn init_git_repo(repo: PathBuf) -> PathBuf {
        fs::create_dir_all(&repo).expect("repo dir should exist");
        git(&repo, ["init", "-b", "main"]);
        git(&repo, ["config", "user.email", "relay@example.com"]);
        git(&repo, ["config", "user.name", "Relay Test"]);
        fs::write(repo.join("README.md"), "hello\n").expect("file should write");
        git(&repo, ["add", "."]);
        git(&repo, ["commit", "-m", "initial"]);
        repo
    }

    fn git<const N: usize>(cwd: &Path, args: [&str; N]) {
        let output = Command::new("git")
            .current_dir(cwd)
            .args(args)
            .output()
            .expect("git command should run");
        assert!(
            output.status.success(),
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
