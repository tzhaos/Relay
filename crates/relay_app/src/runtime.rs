use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context as _, Result, anyhow};
use relay_agent::{
    AgentLaunchPlan, AgentMessage, AgentRegistry, PromptDelivery, RuntimeEnvironment,
    initial_prompt_request,
};
use relay_core::{
    AgentRuntimeStatus, AgentSessionId, CreateTask, DiffFileProjection, DiffHunkProjection,
    DiffLineProjection, DiffLineProjectionKind, DiffStatsProjection, PreviewTargetId, ProjectId,
    ProviderFailure, SelectedRange, Task, TaskCommand, TaskDiffProjection, TaskId, TaskProjection,
    TaskSource, TaskStatus, TerminalSessionId,
};
use relay_diff::{DiffEngine, DiffLineKind, DiffSnapshot, ReviewService};
use relay_infra::paths::RelayPaths;
use relay_persistence::RelayDatabase;
use relay_preview::{
    LocalPreviewProvider, PreviewOpener, PreviewProvider, PreviewRequest, SystemPreviewOpener,
};
use relay_project::{CreateTaskWorktree, Project, ProjectService};
use relay_terminal::{PtyProvider, TerminalError, TerminalEvent, TerminalRuntime, TerminalSpawn};
use relay_ui::{
    app_shell::{TaskDataSource, WorkspaceData},
    terminal_pane::TerminalPaneProjection,
    workbench::ReviewDraftTarget,
};
use time::OffsetDateTime;
use url::Url;

const WORKSPACE_NAMESPACE: &str = "workspace";
const DEFAULT_PROJECT_ID_KEY: &str = "default_project_id";
const DEFAULT_PROJECT_ROOT_KEY: &str = "default_project_root";
const PROJECT_IDS_BY_ROOT_KEY: &str = "project_ids_by_root";
const TASK_BRANCH_PREFIX: &str = "relay";
const TERMINAL_COLS: u16 = 120;
const TERMINAL_ROWS: u16 = 32;
const TERMINAL_SCROLLBACK_LIMIT: usize = 128 * 1024;

pub struct RelayRuntime {
    database: RelayDatabase,
    project_service: ProjectService,
    diff_engine: DiffEngine,
    terminal_runtime: TerminalRuntime,
    agent_registry: AgentRegistry,
    preview_provider: LocalPreviewProvider,
    preview_opener: Box<dyn PreviewOpener>,
    terminal_task_ids: HashMap<TerminalSessionId, TaskId>,
    project: Project,
    worktrees_root: PathBuf,
    worktrees_dir: PathBuf,
}

impl RelayRuntime {
    pub fn open(database: RelayDatabase, paths: &RelayPaths) -> Result<Self> {
        Self::open_with_agent_registry(database, paths, AgentRegistry::built_in())
    }

    fn open_with_agent_registry(
        mut database: RelayDatabase,
        paths: &RelayPaths,
        agent_registry: AgentRegistry,
    ) -> Result<Self> {
        let project_service = ProjectService::default();
        let project = load_or_open_project(&mut database, &project_service)?;
        let worktrees_root = paths.data_dir.join("worktrees");
        let worktrees_dir = worktrees_root.join(project.id.to_string());
        let terminal_task_ids = load_terminal_task_ids(&mut database, project.id)?;

        let mut runtime = Self {
            database,
            project_service,
            diff_engine: DiffEngine::default(),
            terminal_runtime: TerminalRuntime::new(),
            agent_registry,
            preview_provider: LocalPreviewProvider,
            preview_opener: Box::new(SystemPreviewOpener),
            terminal_task_ids,
            project,
            worktrees_root,
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
        self.list_task_projections()
    }

    fn open_project_root(&mut self, path: &Path) -> Result<WorkspaceData> {
        let project = open_project_at(&mut self.database, &self.project_service, path)?;
        self.terminal_runtime = TerminalRuntime::new();
        self.terminal_task_ids = load_terminal_task_ids(&mut self.database, project.id)?;
        self.worktrees_dir = self.worktrees_root.join(project.id.to_string());
        self.project = project;
        self.restore_terminal_sessions()?;

        Ok(WorkspaceData {
            project_label: self.project_label().to_string(),
            tasks: self.load_tasks()?,
        })
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
        {
            let mut repository = self.database.task_repository();
            repository.append_events(&events)?;
            repository.save_snapshot(&projection)?;
        }
        if let Some(session_id) = task.terminal_session_id {
            self.terminal_task_ids.insert(session_id, task.id);
        }
        self.list_task_projections()
    }

    fn launch_agent_for_task(&mut self, task_id: TaskId) -> Result<Vec<TaskProjection>> {
        let mut task = self
            .database
            .task_repository()
            .load_task(task_id)?
            .context("task not found")?;
        let terminal_session_id = task
            .terminal_session_id
            .context("task has no terminal session")?;
        let worktree_path = task
            .worktree
            .as_ref()
            .map(|worktree| PathBuf::from(&worktree.path))
            .context("task has no worktree")?;
        if !self.terminal_runtime.has_session(terminal_session_id) {
            self.spawn_terminal_with_id(terminal_session_id, worktree_path.clone())?;
        }

        let plan = self.agent_launch_plan(&task, worktree_path)?;
        self.write_agent_launch(terminal_session_id, &plan)?;

        let mut events = task.handle(TaskCommand::AttachAgent {
            id: AgentSessionId::new(),
            kind: plan.agent.clone(),
            started_at: OffsetDateTime::now_utc(),
        })?;
        apply_events(&mut task, &events)?;
        let status_events = task.handle(TaskCommand::ApplyAgentStatus(
            relay_core::AgentStatusUpdate {
                state: AgentRuntimeStatus::Working,
                prompt: format!("Launched {}", plan.agent.label()),
                agent_kind: Some(plan.agent),
                observed_at: OffsetDateTime::now_utc(),
            },
        ))?;
        apply_events(&mut task, &status_events)?;
        events.extend(status_events);

        let projection = TaskProjection::from_task(&task);
        {
            let mut repository = self.database.task_repository();
            repository.append_events(&events)?;
            repository.save_snapshot(&projection)?;
        }
        self.list_task_projections()
    }

    fn deliver_review_for_task(&mut self, task_id: TaskId) -> Result<Vec<TaskProjection>> {
        let mut task = self
            .database
            .task_repository()
            .load_task(task_id)?
            .context("task not found")?;
        let terminal_session_id = task
            .terminal_session_id
            .context("task has no terminal session")?;
        let agent_kind = task
            .agent_kind
            .clone()
            .context("task has no active agent")?;
        if !self.terminal_runtime.has_session(terminal_session_id) {
            let worktree_path = task
                .worktree
                .as_ref()
                .map(|worktree| PathBuf::from(&worktree.path))
                .context("task has no worktree")?;
            self.spawn_terminal_with_id(terminal_session_id, worktree_path)?;
        }

        let delivery = ReviewService::deliver_comments_to_agent(&task)?;
        let input = self
            .agent_registry
            .resolve(&agent_kind)?
            .format_followup(AgentMessage {
                body: delivery.prompt.clone(),
            })?;
        self.terminal_runtime
            .write(terminal_session_id, &input.bytes)?;

        let events = task.handle(ReviewService::mark_delivered(
            &delivery,
            OffsetDateTime::now_utc(),
        ))?;
        apply_events(&mut task, &events)?;
        let projection = TaskProjection::from_task(&task);
        {
            let mut repository = self.database.task_repository();
            repository.append_events(&events)?;
            repository.save_snapshot(&projection)?;
        }
        self.list_task_projections()
    }

    fn archive_task_for_task(&mut self, task_id: TaskId) -> Result<Vec<TaskProjection>> {
        let mut task = self
            .database
            .task_repository()
            .load_task(task_id)?
            .context("task not found")?;
        let terminal_session_id = task.terminal_session_id;
        let events = task.handle(TaskCommand::Archive {
            now: OffsetDateTime::now_utc(),
        })?;
        apply_events(&mut task, &events)?;
        let projection = TaskProjection::from_task(&task);
        {
            let mut repository = self.database.task_repository();
            repository.append_events(&events)?;
            repository.save_snapshot(&projection)?;
        }

        if let Some(session_id) = terminal_session_id {
            self.terminal_task_ids.remove(&session_id);
            if self.terminal_runtime.has_session(session_id) {
                self.terminal_runtime.kill(session_id)?;
            }
        }

        self.list_task_projections()
    }

    fn add_review_comment_for_target(
        &mut self,
        target: ReviewDraftTarget,
        body: &str,
    ) -> Result<Vec<TaskProjection>> {
        let mut task = self
            .database
            .task_repository()
            .load_task(target.task_id)?
            .context("task not found")?;
        let selected_range = SelectedRange {
            start: target.line.clone(),
            end: target.line.clone(),
            selected_text: target.selected_text.clone(),
        };
        let comment = ReviewService::add_comment(
            task.id,
            target.line.path,
            body,
            Some(selected_range),
            OffsetDateTime::now_utc(),
        )?;
        let events = task.handle(TaskCommand::AddReviewComment(comment))?;
        apply_events(&mut task, &events)?;
        let projection = TaskProjection::from_task(&task);
        {
            let mut repository = self.database.task_repository();
            repository.append_events(&events)?;
            repository.save_snapshot(&projection)?;
        }
        self.list_task_projections()
    }

    fn attach_worktree_preview_for_task(&mut self, task_id: TaskId) -> Result<Vec<TaskProjection>> {
        let mut task = self
            .database
            .task_repository()
            .load_task(task_id)?
            .context("task not found")?;
        let worktree_path = task
            .worktree
            .as_ref()
            .map(|worktree| PathBuf::from(&worktree.path))
            .context("task has no worktree")?;
        let uri = worktree_file_uri(&worktree_path)?;
        if task
            .preview_targets
            .iter()
            .any(|target| target.label == "Worktree" && target.uri == uri)
        {
            return self.list_task_projections();
        }

        let command = self.preview_provider.attach_target(
            task.id,
            PreviewRequest {
                label: "Worktree".to_string(),
                uri,
            },
            OffsetDateTime::now_utc(),
        )?;
        let events = task.handle(command)?;
        apply_events(&mut task, &events)?;
        let projection = TaskProjection::from_task(&task);
        {
            let mut repository = self.database.task_repository();
            repository.append_events(&events)?;
            repository.save_snapshot(&projection)?;
        }
        self.list_task_projections()
    }

    fn open_preview_target_for_task(
        &mut self,
        task_id: TaskId,
        target_id: PreviewTargetId,
    ) -> Result<()> {
        let task = self
            .database
            .task_repository()
            .load_task(task_id)?
            .context("task not found")?;
        let target = task
            .preview_targets
            .iter()
            .find(|target| target.id == target_id)
            .context("preview target not found")?;

        self.preview_opener.open_target(&target.uri)?;
        Ok(())
    }

    fn refresh_changed_files(&mut self) -> Result<()> {
        let task_ids = self
            .database
            .task_repository()
            .list_tasks_for_project(self.project.id)?
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
            .list_tasks_for_project(self.project.id)?
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

    fn list_task_projections(&mut self) -> Result<Vec<TaskProjection>> {
        let mut tasks = self
            .database
            .task_repository()
            .list_tasks_for_project(self.project.id)?;
        self.enrich_diff_projections(&mut tasks)?;
        Ok(tasks)
    }

    fn enrich_diff_projections(&self, tasks: &mut [TaskProjection]) -> Result<()> {
        for task in tasks {
            if task.changed_file_count == 0 {
                task.diff = TaskDiffProjection::default();
                continue;
            }
            let Some(worktree_path) = task.worktree_path.as_ref().map(PathBuf::from) else {
                continue;
            };
            if !worktree_path.exists() {
                continue;
            }
            let snapshot = self.diff_engine.load(&worktree_path, None)?;
            task.diff = task_diff_projection(&snapshot);
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

    fn drain_terminal_events(&mut self) -> Result<bool> {
        let mut changed = false;
        while let Some(event) = self.terminal_runtime.poll_event(Duration::ZERO) {
            self.apply_agent_status_from_terminal_event(&event)?;
            changed = true;
        }
        if self.mark_stale_agent_statuses()? {
            changed = true;
        }
        Ok(changed)
    }

    fn mark_stale_agent_statuses(&mut self) -> Result<bool> {
        let now = OffsetDateTime::now_utc();
        let task_ids = self
            .database
            .task_repository()
            .list_tasks_for_project(self.project.id)?
            .into_iter()
            .filter(|task| {
                matches!(
                    task.status,
                    TaskStatus::StartingAgent
                        | TaskStatus::Working
                        | TaskStatus::WaitingForUser
                        | TaskStatus::Blocked
                )
            })
            .filter_map(|task| {
                task.agent
                    .map(|agent| (task.id, agent, task.last_activity_at))
            })
            .filter_map(|(task_id, agent, last_activity_at)| {
                self.agent_registry
                    .stale_status(agent, last_activity_at, now)
                    .map(|update| (task_id, update))
            })
            .collect::<Vec<_>>();

        let mut changed = false;
        for (task_id, update) in task_ids {
            let Some(mut task) = self.database.task_repository().load_task(task_id)? else {
                continue;
            };
            let events = task.handle(TaskCommand::ApplyAgentStatus(update))?;
            apply_events(&mut task, &events)?;
            let projection = TaskProjection::from_task(&task);
            let mut repository = self.database.task_repository();
            repository.append_events(&events)?;
            repository.save_snapshot(&projection)?;
            changed = true;
        }

        Ok(changed)
    }

    fn terminal_projection_for(
        &mut self,
        session_id: TerminalSessionId,
    ) -> Result<Option<TerminalPaneProjection>> {
        self.drain_terminal_events()?;
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

    fn write_terminal_input(&mut self, session_id: TerminalSessionId, bytes: &[u8]) -> Result<()> {
        if !self.terminal_runtime.has_session(session_id) {
            let task = self
                .task_for_terminal(session_id)?
                .context("terminal session is not attached to a task")?;
            let worktree_path = task
                .worktree
                .as_ref()
                .map(|worktree| PathBuf::from(&worktree.path))
                .context("task has no worktree")?;
            self.spawn_terminal_with_id(session_id, worktree_path)?;
        }

        self.terminal_runtime.write(session_id, bytes)?;
        Ok(())
    }

    fn agent_launch_plan(&self, task: &Task, cwd: PathBuf) -> Result<AgentLaunchPlan> {
        let env = RuntimeEnvironment::current()?;
        let availability = self
            .agent_registry
            .detect_available(&env)
            .into_iter()
            .find(|candidate| candidate.available)
            .ok_or_else(|| anyhow!("no available agent CLI found: {}", self.agent_labels()))?;

        Ok(self.agent_registry.launch_plan(
            &availability.kind,
            initial_prompt_request(cwd, task.title.clone())?,
        )?)
    }

    fn write_agent_launch(
        &mut self,
        session_id: TerminalSessionId,
        plan: &AgentLaunchPlan,
    ) -> Result<()> {
        let command = shell_command_line(&plan.program, &plan.args);
        self.terminal_runtime
            .write(session_id, command.as_bytes())?;
        self.terminal_runtime
            .write(session_id, terminal_submit_sequence())?;
        if matches!(plan.prompt_delivery, PromptDelivery::StdinAfterStart)
            && let Some(bytes) = &plan.stdin_after_start
        {
            self.terminal_runtime.write(session_id, bytes)?;
        }
        Ok(())
    }

    fn apply_agent_status_from_terminal_event(&mut self, event: &TerminalEvent) -> Result<()> {
        let session_id = match event {
            TerminalEvent::Output { session_id, .. }
            | TerminalEvent::Title { session_id, .. }
            | TerminalEvent::Cwd { session_id, .. }
            | TerminalEvent::Exited { session_id } => *session_id,
        };
        let Some(mut task) = self.task_for_terminal(session_id)? else {
            return Ok(());
        };
        let Some(agent_kind) = task.agent_kind.clone() else {
            return Ok(());
        };
        let Some(update) = self.agent_registry.parse_terminal_event(
            &agent_kind,
            event,
            OffsetDateTime::now_utc(),
        )?
        else {
            return Ok(());
        };

        let events = task.handle(TaskCommand::ApplyAgentStatus(update))?;
        apply_events(&mut task, &events)?;
        let projection = TaskProjection::from_task(&task);
        let mut repository = self.database.task_repository();
        repository.append_events(&events)?;
        repository.save_snapshot(&projection)?;
        Ok(())
    }

    fn task_for_terminal(&mut self, session_id: TerminalSessionId) -> Result<Option<Task>> {
        let Some(task_id) = self.terminal_task_ids.get(&session_id).copied() else {
            return Ok(None);
        };

        Ok(self.database.task_repository().load_task(task_id)?)
    }

    fn agent_labels(&self) -> String {
        let labels = self
            .agent_registry
            .adapters()
            .iter()
            .map(|adapter| adapter.kind().label().to_string())
            .collect::<Vec<_>>();
        if labels.is_empty() {
            "none configured".to_string()
        } else {
            labels.join(", ")
        }
    }
}

impl TaskDataSource for RelayRuntime {
    fn open_project(&mut self, path: &Path) -> Result<WorkspaceData> {
        self.open_project_root(path)
    }

    fn refresh_changed_files(&mut self) -> Result<Vec<TaskProjection>> {
        self.load_tasks()
    }

    fn create_task(&mut self, title: &str) -> Result<Vec<TaskProjection>> {
        self.create_task_with_worktree(title)
    }

    fn launch_agent(&mut self, task_id: TaskId) -> Result<Vec<TaskProjection>> {
        self.launch_agent_for_task(task_id)
    }

    fn deliver_review(&mut self, task_id: TaskId) -> Result<Vec<TaskProjection>> {
        self.deliver_review_for_task(task_id)
    }

    fn archive_task(&mut self, task_id: TaskId) -> Result<Vec<TaskProjection>> {
        self.archive_task_for_task(task_id)
    }

    fn add_review_comment(
        &mut self,
        target: ReviewDraftTarget,
        body: &str,
    ) -> Result<Vec<TaskProjection>> {
        self.add_review_comment_for_target(target, body)
    }

    fn attach_worktree_preview(&mut self, task_id: TaskId) -> Result<Vec<TaskProjection>> {
        self.attach_worktree_preview_for_task(task_id)
    }

    fn open_preview_target(&mut self, task_id: TaskId, target_id: PreviewTargetId) -> Result<()> {
        self.open_preview_target_for_task(task_id, target_id)
    }

    fn write_terminal(&mut self, session_id: TerminalSessionId, bytes: &[u8]) -> Result<()> {
        self.write_terminal_input(session_id, bytes)
    }

    fn poll_runtime(&mut self) -> Result<bool> {
        self.drain_terminal_events()
    }

    fn terminal_projection(
        &mut self,
        session_id: TerminalSessionId,
    ) -> Result<Option<TerminalPaneProjection>> {
        self.terminal_projection_for(session_id)
    }
}

fn load_or_open_project(
    database: &mut RelayDatabase,
    project_service: &ProjectService,
) -> Result<Project> {
    let stored_root = database
        .settings_repository()
        .get_json::<PathBuf>(WORKSPACE_NAMESPACE, DEFAULT_PROJECT_ROOT_KEY)?;
    let current_dir = std::env::current_dir().context("failed to read current directory")?;
    let candidate = stored_root.as_deref().unwrap_or(&current_dir);

    match open_project_at(database, project_service, candidate) {
        Ok(project) => Ok(project),
        Err(error) if stored_root.is_some() => {
            open_project_at(database, project_service, &current_dir).with_context(|| {
                format!(
                    "stored project root {} could not be opened: {error}",
                    candidate.display()
                )
            })
        }
        Err(error) => Err(error).context("failed to open current directory as git repo"),
    }
}

fn open_project_at(
    database: &mut RelayDatabase,
    project_service: &ProjectService,
    path: &Path,
) -> Result<Project> {
    let mut project = project_service.open_repo(path)?;
    project.id = load_or_create_project_id_for_root(database, &project.root)?;
    save_default_project(database, &project)?;

    Ok(project)
}

fn load_or_create_project_id_for_root(
    database: &mut RelayDatabase,
    root: &Path,
) -> Result<ProjectId> {
    let root_key = project_root_key(root);
    let mut project_ids = database
        .settings_repository()
        .get_json::<HashMap<String, ProjectId>>(WORKSPACE_NAMESPACE, PROJECT_IDS_BY_ROOT_KEY)?
        .unwrap_or_default();
    if let Some(project_id) = project_ids.get(&root_key).copied() {
        return Ok(project_id);
    }

    let stored_root = database
        .settings_repository()
        .get_json::<PathBuf>(WORKSPACE_NAMESPACE, DEFAULT_PROJECT_ROOT_KEY)?;
    let legacy_project_id = database
        .settings_repository()
        .get_json::<ProjectId>(WORKSPACE_NAMESPACE, DEFAULT_PROJECT_ID_KEY)?;
    let project_id = if project_ids.is_empty()
        && stored_root
            .as_deref()
            .is_some_and(|stored_root| project_root_key(stored_root) == root_key)
    {
        legacy_project_id.unwrap_or_default()
    } else {
        ProjectId::new()
    };

    project_ids.insert(root_key, project_id);
    database.settings_repository().set_json(
        WORKSPACE_NAMESPACE,
        PROJECT_IDS_BY_ROOT_KEY,
        &project_ids,
    )?;
    Ok(project_id)
}

fn save_default_project(database: &mut RelayDatabase, project: &Project) -> Result<()> {
    database.settings_repository().set_json(
        WORKSPACE_NAMESPACE,
        DEFAULT_PROJECT_ROOT_KEY,
        &project.root,
    )?;
    database.settings_repository().set_json(
        WORKSPACE_NAMESPACE,
        DEFAULT_PROJECT_ID_KEY,
        &project.id,
    )?;
    Ok(())
}

fn project_root_key(root: &Path) -> String {
    root.to_string_lossy().to_string()
}

fn worktree_file_uri(path: &Path) -> Result<String> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("failed to resolve worktree path: {}", path.display()))?;
    let url = Url::from_directory_path(&canonical)
        .map_err(|()| anyhow!("failed to convert worktree path to file URI"))?;
    Ok(url.to_string())
}

fn load_terminal_task_ids(
    database: &mut RelayDatabase,
    project_id: ProjectId,
) -> Result<HashMap<TerminalSessionId, TaskId>> {
    Ok(database
        .task_repository()
        .list_tasks_for_project(project_id)?
        .into_iter()
        .filter_map(|task| {
            task.terminal_session_id
                .map(|terminal_session_id| (terminal_session_id, task.id))
        })
        .collect())
}

fn task_diff_projection(snapshot: &DiffSnapshot) -> TaskDiffProjection {
    TaskDiffProjection {
        files: snapshot
            .files
            .iter()
            .map(|file| DiffFileProjection {
                path: file.display_path().to_string(),
                status: file.status,
                is_binary: file.is_binary,
                hunks: file
                    .hunks
                    .iter()
                    .map(|hunk| DiffHunkProjection {
                        header: hunk.header.clone(),
                        lines: hunk
                            .lines
                            .iter()
                            .map(|line| DiffLineProjection {
                                kind: diff_line_kind(line.kind),
                                old_line: line.old_line,
                                new_line: line.new_line,
                                content: line.content.clone(),
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
        stats: DiffStatsProjection {
            files_changed: snapshot.stats.files_changed,
            additions: snapshot.stats.additions,
            deletions: snapshot.stats.deletions,
        },
    }
}

fn diff_line_kind(kind: DiffLineKind) -> DiffLineProjectionKind {
    match kind {
        DiffLineKind::Context => DiffLineProjectionKind::Context,
        DiffLineKind::Added => DiffLineProjectionKind::Added,
        DiffLineKind::Deleted => DiffLineProjectionKind::Deleted,
        DiffLineKind::NoNewline => DiffLineProjectionKind::NoNewline,
    }
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

fn shell_command_line(program: &str, args: &[String]) -> String {
    std::iter::once(shell_quote(program))
        .chain(args.iter().map(|arg| shell_quote(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(windows)]
fn terminal_submit_sequence() -> &'static [u8] {
    b"\r\n"
}

#[cfg(not(windows))]
fn terminal_submit_sequence() -> &'static [u8] {
    b"\n"
}

#[cfg(windows)]
fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    let needs_quotes = value.chars().any(|character| {
        character.is_whitespace()
            || matches!(character, '"' | '&' | '|' | '<' | '>' | '^' | '(' | ')')
    });
    if !needs_quotes {
        return value.to_string();
    }

    let escaped = value.replace('"', "\\\"");
    format!("\"{escaped}\"")
}

#[cfg(not(windows))]
fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/' | ':')
    }) {
        return value.to_string();
    }

    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        fs,
        path::{Path, PathBuf},
        process::Command,
        rc::Rc,
        thread,
        time::{Duration, Instant},
    };

    use relay_agent::{
        AgentAdapter, AgentInput, AgentLaunchPlan, AgentLaunchRequest, AgentMessage, AgentResult,
        PromptDelivery,
    };
    use relay_core::{AgentKind, AgentRuntimeStatus, AgentStatusUpdate, Timestamp};
    use relay_terminal::TerminalEvent;
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
    fn open_project_root_should_switch_project_and_filter_tasks() {
        let temp = tempdir().expect("tempdir should exist");
        let repo_one = init_git_repo(temp.path().join("repo-one"));
        let repo_two = init_git_repo(temp.path().join("repo-two"));
        let paths = RelayPaths {
            data_dir: temp.path().join("data"),
            config_dir: temp.path().join("config"),
            log_dir: temp.path().join("logs"),
        };
        let mut database =
            RelayDatabase::open(temp.path().join("relay.sqlite3")).expect("database should open");
        database
            .settings_repository()
            .set_json(WORKSPACE_NAMESPACE, DEFAULT_PROJECT_ROOT_KEY, &repo_one)
            .expect("project root should save");
        let mut runtime = RelayRuntime::open(database, &paths).expect("runtime should open");
        let repo_one_tasks = runtime
            .create_task_with_worktree("Repo one task")
            .expect("task should create");
        let repo_one_task_id = repo_one_tasks[0].id;

        let repo_two_workspace = runtime
            .open_project_root(&repo_two)
            .expect("second project should open");

        assert_eq!(repo_two_workspace.project_label, "repo-two");
        assert!(repo_two_workspace.tasks.is_empty());

        let repo_one_workspace = runtime
            .open_project_root(&repo_one)
            .expect("first project should reopen");

        assert_eq!(repo_one_workspace.project_label, "repo-one");
        assert_eq!(repo_one_workspace.tasks[0].id, repo_one_task_id);
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
        assert_eq!(tasks[0].diff.stats.files_changed, 1);
        assert!(
            tasks[0].diff.files[0].hunks[0]
                .lines
                .iter()
                .any(|line| line.content.contains("changed")),
            "diff projection should include file content"
        );
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
    fn write_terminal_input_should_send_bytes_to_task_pty() {
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
            .create_task_with_worktree("Type in terminal")
            .expect("task should create");
        let session_id = tasks[0]
            .terminal_session_id
            .expect("terminal id should be attached");

        runtime
            .write_terminal_input(session_id, b"echo relay-input")
            .expect("terminal input should write");
        runtime
            .write_terminal_input(session_id, terminal_submit_sequence())
            .expect("terminal enter should write");

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut scrollback = String::new();
        while Instant::now() < deadline {
            runtime
                .drain_terminal_events()
                .expect("terminal events should drain");
            scrollback = runtime
                .terminal_projection_for(session_id)
                .expect("terminal projection should load")
                .expect("terminal should be connected")
                .scrollback;
            if scrollback.contains("relay-input") {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        assert!(
            scrollback.contains("relay-input"),
            "scrollback did not contain echoed input: {scrollback:?}"
        );
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

    #[test]
    fn launch_agent_should_attach_real_runtime_to_task_terminal() {
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
        let mut runtime = RelayRuntime::open_with_agent_registry(
            database,
            &paths,
            AgentRegistry::with_adapters(vec![Box::new(TestAgentAdapter)]),
        )
        .expect("runtime should open");
        let tasks = runtime
            .create_task_with_worktree("Run real agent")
            .expect("task should create");
        let task_id = tasks[0].id;

        let tasks = runtime
            .launch_agent_for_task(task_id)
            .expect("agent should launch");
        let task = tasks
            .iter()
            .find(|task| task.id == task_id)
            .expect("task should remain listed");

        assert_eq!(task.status, TaskStatus::Working);
        assert_eq!(
            task.agent.as_ref().map(AgentKind::label),
            Some("test-agent")
        );
    }

    #[test]
    fn launch_agent_should_report_done_when_terminal_output_matches_adapter() {
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
        let mut runtime = RelayRuntime::open_with_agent_registry(
            database,
            &paths,
            AgentRegistry::with_adapters(vec![Box::new(TestAgentAdapter)]),
        )
        .expect("runtime should open");
        let tasks = runtime
            .create_task_with_worktree("Observe agent")
            .expect("task should create");
        let task_id = tasks[0].id;
        let session_id = tasks[0]
            .terminal_session_id
            .expect("terminal id should be attached");
        runtime
            .launch_agent_for_task(task_id)
            .expect("agent should launch");

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut status = TaskStatus::Working;
        while Instant::now() < deadline {
            runtime
                .drain_terminal_events()
                .expect("terminal events should drain");
            let tasks = runtime.load_tasks().expect("tasks should reload");
            status = tasks
                .iter()
                .find(|task| task.id == task_id)
                .expect("task should remain listed")
                .status;
            if status == TaskStatus::Done {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        let scrollback = runtime
            .terminal_projection_for(session_id)
            .expect("terminal projection should load")
            .expect("terminal should be connected")
            .scrollback;
        assert_eq!(status, TaskStatus::Done, "scrollback: {scrollback:?}");
    }

    #[test]
    fn poll_runtime_should_mark_silent_agent_stale_after_threshold() {
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
        let mut runtime = RelayRuntime::open_with_agent_registry(
            database,
            &paths,
            AgentRegistry::with_adapters(vec![Box::new(TestAgentAdapter)])
                .with_stale_after(Duration::ZERO),
        )
        .expect("runtime should open");
        let tasks = runtime
            .create_task_with_worktree("Stale agent")
            .expect("task should create");
        let task_id = tasks[0].id;
        attach_test_agent(&mut runtime, task_id);

        let changed = runtime
            .drain_terminal_events()
            .expect("runtime should poll stale state");
        let task = runtime
            .database
            .task_repository()
            .load_task(task_id)
            .expect("task should load")
            .expect("task should exist");

        assert!(changed);
        assert_eq!(task.status, TaskStatus::Stale);
    }

    #[test]
    fn launch_agent_should_fail_when_no_agent_cli_is_available() {
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
        let mut runtime = RelayRuntime::open_with_agent_registry(
            database,
            &paths,
            AgentRegistry::with_adapters(vec![Box::new(MissingAgentAdapter)]),
        )
        .expect("runtime should open");
        let tasks = runtime
            .create_task_with_worktree("Missing agent")
            .expect("task should create");
        let task_id = tasks[0].id;

        let error = runtime
            .launch_agent_for_task(task_id)
            .expect_err("missing cli should not fake success");

        assert!(
            error.to_string().contains("no available agent CLI found"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn deliver_review_should_mark_pending_comments_delivered() {
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
        let mut runtime = RelayRuntime::open_with_agent_registry(
            database,
            &paths,
            AgentRegistry::with_adapters(vec![Box::new(TestAgentAdapter)]),
        )
        .expect("runtime should open");
        let tasks = runtime
            .create_task_with_worktree("Deliver review")
            .expect("task should create");
        let task_id = tasks[0].id;
        attach_test_agent(&mut runtime, task_id);
        add_review_comment(&mut runtime, task_id, "Tighten error handling");

        let tasks = runtime
            .deliver_review_for_task(task_id)
            .expect("review should deliver");
        let task = tasks
            .iter()
            .find(|task| task.id == task_id)
            .expect("task should remain listed");

        assert_eq!(task.pending_review_comment_count, 0);
        assert_eq!(task.status, TaskStatus::ReadyToCommit);
    }

    #[test]
    fn deliver_review_should_fail_without_agent() {
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
        let mut runtime = RelayRuntime::open_with_agent_registry(
            database,
            &paths,
            AgentRegistry::with_adapters(vec![Box::new(TestAgentAdapter)]),
        )
        .expect("runtime should open");
        let tasks = runtime
            .create_task_with_worktree("Undeliverable review")
            .expect("task should create");
        let task_id = tasks[0].id;
        add_review_comment(&mut runtime, task_id, "Needs an agent");

        let error = runtime
            .deliver_review_for_task(task_id)
            .expect_err("review delivery requires an agent");

        assert!(
            error.to_string().contains("task has no active agent"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn archive_task_should_persist_status_and_stop_terminal() {
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
        let mut runtime = RelayRuntime::open_with_agent_registry(
            database,
            &paths,
            AgentRegistry::with_adapters(vec![Box::new(TestAgentAdapter)]),
        )
        .expect("runtime should open");
        let tasks = runtime
            .create_task_with_worktree("Archive terminal")
            .expect("task should create");
        let task_id = tasks[0].id;
        let session_id = tasks[0]
            .terminal_session_id
            .expect("terminal should be attached");

        let tasks = runtime
            .archive_task_for_task(task_id)
            .expect("task should archive");
        let task = tasks
            .iter()
            .find(|task| task.id == task_id)
            .expect("task should remain listed");
        let persisted = runtime
            .database
            .task_repository()
            .load_task(task_id)
            .expect("task should load")
            .expect("task should exist");
        let terminal = runtime
            .terminal_runtime
            .snapshot(session_id)
            .expect("terminal snapshot should remain inspectable");

        assert_eq!(task.status, TaskStatus::Archived);
        assert_eq!(persisted.status, TaskStatus::Archived);
        assert!(terminal.exited, "archiving should stop the task terminal");
        assert!(!runtime.terminal_task_ids.contains_key(&session_id));
    }

    #[test]
    fn add_review_comment_should_persist_line_target() {
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
        let mut runtime = RelayRuntime::open_with_agent_registry(
            database,
            &paths,
            AgentRegistry::with_adapters(vec![Box::new(TestAgentAdapter)]),
        )
        .expect("runtime should open");
        let tasks = runtime
            .create_task_with_worktree("Review line")
            .expect("task should create");
        let task_id = tasks[0].id;

        let tasks = runtime
            .add_review_comment_for_target(review_draft_target(task_id), "Tighten this error path.")
            .expect("review comment should persist");
        let task = tasks
            .iter()
            .find(|task| task.id == task_id)
            .expect("task should remain listed");
        let persisted = runtime
            .database
            .task_repository()
            .load_task(task_id)
            .expect("task should load")
            .expect("task should exist");
        let line = persisted.diff_review.comments[0]
            .line
            .as_deref()
            .expect("line target should persist");

        assert_eq!(task.pending_review_comment_count, 1);
        assert_eq!(line.new_line, Some(42));
    }

    #[test]
    fn attach_worktree_preview_should_persist_file_target() {
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
        let mut runtime = RelayRuntime::open_with_agent_registry(
            database,
            &paths,
            AgentRegistry::with_adapters(vec![Box::new(TestAgentAdapter)]),
        )
        .expect("runtime should open");
        let tasks = runtime
            .create_task_with_worktree("Preview worktree")
            .expect("task should create");
        let task_id = tasks[0].id;

        let tasks = runtime
            .attach_worktree_preview_for_task(task_id)
            .expect("preview should attach");
        let task = tasks
            .iter()
            .find(|task| task.id == task_id)
            .expect("task should remain listed");
        let target = task
            .preview_targets
            .first()
            .expect("preview target should exist");

        assert_eq!(task.preview_target_count, 1);
        assert_eq!(target.label, "Worktree");
        assert!(target.uri.starts_with("file://"));
    }

    #[test]
    fn attach_worktree_preview_should_not_duplicate_existing_target() {
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
        let mut runtime = RelayRuntime::open_with_agent_registry(
            database,
            &paths,
            AgentRegistry::with_adapters(vec![Box::new(TestAgentAdapter)]),
        )
        .expect("runtime should open");
        let tasks = runtime
            .create_task_with_worktree("Preview once")
            .expect("task should create");
        let task_id = tasks[0].id;
        runtime
            .attach_worktree_preview_for_task(task_id)
            .expect("preview should attach");

        let tasks = runtime
            .attach_worktree_preview_for_task(task_id)
            .expect("duplicate preview attach should be ignored");
        let task = tasks
            .iter()
            .find(|task| task.id == task_id)
            .expect("task should remain listed");

        assert_eq!(task.preview_target_count, 1);
    }

    #[test]
    fn open_preview_target_should_delegate_attached_uri_to_opener() {
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
        let mut runtime = RelayRuntime::open_with_agent_registry(
            database,
            &paths,
            AgentRegistry::with_adapters(vec![Box::new(TestAgentAdapter)]),
        )
        .expect("runtime should open");
        let opened = Rc::new(RefCell::new(Vec::new()));
        runtime.preview_opener = Box::new(RecordingPreviewOpener {
            opened: Rc::clone(&opened),
        });
        let tasks = runtime
            .create_task_with_worktree("Open preview")
            .expect("task should create");
        let task_id = tasks[0].id;
        let tasks = runtime
            .attach_worktree_preview_for_task(task_id)
            .expect("preview should attach");
        let target = tasks[0]
            .preview_targets
            .first()
            .expect("preview target should exist");

        runtime
            .open_preview_target_for_task(task_id, target.id)
            .expect("preview should open");

        assert_eq!(opened.borrow().as_slice(), [target.uri.as_str()]);
    }

    #[cfg(windows)]
    #[test]
    fn windows_shell_quote_should_preserve_path_backslashes() {
        assert_eq!(
            shell_quote(r"C:\Program Files\Relay Agent\agent.exe"),
            r#""C:\Program Files\Relay Agent\agent.exe""#
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn unix_shell_quote_should_escape_single_quotes() {
        assert_eq!(shell_quote("run agent's task"), "'run agent'\\''s task'");
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

    fn attach_test_agent(runtime: &mut RelayRuntime, task_id: TaskId) {
        apply_task_command(
            runtime,
            task_id,
            TaskCommand::AttachAgent {
                id: AgentSessionId::new(),
                kind: AgentKind::Custom("test-agent".to_string()),
                started_at: OffsetDateTime::now_utc(),
            },
        );
        apply_task_command(
            runtime,
            task_id,
            TaskCommand::ApplyAgentStatus(AgentStatusUpdate {
                state: AgentRuntimeStatus::Working,
                prompt: "Ready for review".to_string(),
                agent_kind: Some(AgentKind::Custom("test-agent".to_string())),
                observed_at: OffsetDateTime::now_utc(),
            }),
        );
    }

    fn add_review_comment(runtime: &mut RelayRuntime, task_id: TaskId, body: &str) {
        let comment = ReviewService::add_comment(
            task_id,
            "src/lib.rs",
            body,
            None,
            OffsetDateTime::now_utc(),
        )
        .expect("review comment should be valid");
        apply_task_command(runtime, task_id, TaskCommand::AddReviewComment(comment));
    }

    fn review_draft_target(task_id: TaskId) -> ReviewDraftTarget {
        ReviewDraftTarget {
            task_id,
            line: relay_core::LineIdentity {
                path: "src/lib.rs".to_string(),
                side: relay_core::DiffSide::New,
                old_line: None,
                new_line: Some(42),
                hunk_header: "@@ -40,1 +42,1 @@".to_string(),
            },
            selected_text: Some("return Err(error);".to_string()),
        }
    }

    fn apply_task_command(runtime: &mut RelayRuntime, task_id: TaskId, command: TaskCommand) {
        let mut task = runtime
            .database
            .task_repository()
            .load_task(task_id)
            .expect("task should load")
            .expect("task should exist");
        let events = task.handle(command).expect("command should produce events");
        apply_events(&mut task, &events).expect("events should apply");
        let projection = TaskProjection::from_task(&task);
        let mut repository = runtime.database.task_repository();
        repository
            .append_events(&events)
            .expect("events should persist");
        repository
            .save_snapshot(&projection)
            .expect("snapshot should persist");
    }

    #[derive(Debug)]
    struct TestAgentAdapter;

    impl AgentAdapter for TestAgentAdapter {
        fn kind(&self) -> AgentKind {
            AgentKind::Custom("test-agent".to_string())
        }

        fn command(&self) -> &str {
            test_agent_command()
        }

        fn launch_plan(&self, request: AgentLaunchRequest) -> AgentResult<AgentLaunchPlan> {
            Ok(AgentLaunchPlan {
                agent: self.kind(),
                program: self.command().to_string(),
                args: test_agent_args(),
                env: request.env,
                cwd: request.cwd,
                expected_process: Some(self.command().to_string()),
                prompt_delivery: PromptDelivery::Argv,
                stdin_after_start: None,
            })
        }

        fn parse_terminal_event(
            &self,
            event: &TerminalEvent,
            observed_at: Timestamp,
        ) -> Option<AgentStatusUpdate> {
            let TerminalEvent::Output { data, .. } = event else {
                return None;
            };
            let output = String::from_utf8_lossy(data).to_ascii_lowercase();
            let state = if output.contains("done") {
                AgentRuntimeStatus::Done
            } else if output.contains("working") {
                AgentRuntimeStatus::Working
            } else {
                return None;
            };

            Some(AgentStatusUpdate {
                state,
                prompt: output.trim().to_string(),
                agent_kind: Some(self.kind()),
                observed_at,
            })
        }

        fn format_followup(&self, message: AgentMessage) -> AgentResult<AgentInput> {
            Ok(AgentInput::stdin_line(message.body))
        }
    }

    struct MissingAgentAdapter;

    impl AgentAdapter for MissingAgentAdapter {
        fn kind(&self) -> AgentKind {
            AgentKind::Custom("missing-agent".to_string())
        }

        fn command(&self) -> &str {
            "relay-missing-agent-cli"
        }

        fn launch_plan(&self, request: AgentLaunchRequest) -> AgentResult<AgentLaunchPlan> {
            Ok(AgentLaunchPlan {
                agent: self.kind(),
                program: self.command().to_string(),
                args: Vec::new(),
                env: request.env,
                cwd: request.cwd,
                expected_process: Some(self.command().to_string()),
                prompt_delivery: PromptDelivery::Argv,
                stdin_after_start: None,
            })
        }

        fn parse_terminal_event(
            &self,
            _event: &TerminalEvent,
            _observed_at: Timestamp,
        ) -> Option<AgentStatusUpdate> {
            None
        }

        fn format_followup(&self, message: AgentMessage) -> AgentResult<AgentInput> {
            Ok(AgentInput::stdin_line(message.body))
        }
    }

    struct RecordingPreviewOpener {
        opened: Rc<RefCell<Vec<String>>>,
    }

    impl PreviewOpener for RecordingPreviewOpener {
        fn open_target(&mut self, uri: &str) -> relay_preview::PreviewResult<()> {
            self.opened.borrow_mut().push(uri.to_string());
            Ok(())
        }
    }

    #[cfg(windows)]
    fn test_agent_command() -> &'static str {
        "cmd.exe"
    }

    #[cfg(windows)]
    fn test_agent_args() -> Vec<String> {
        vec![
            "/Q".to_string(),
            "/C".to_string(),
            "echo working && echo done".to_string(),
        ]
    }

    #[cfg(not(windows))]
    fn test_agent_command() -> &'static str {
        "sh"
    }

    #[cfg(not(windows))]
    fn test_agent_args() -> Vec<String> {
        vec!["-c".to_string(), "printf 'working\ndone\n'".to_string()]
    }
}
