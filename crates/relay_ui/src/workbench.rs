use relay_core::{
    LineIdentity, PreviewTargetId, TaskId, TaskProjection, TaskStatus, TerminalSessionId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneRoute {
    Terminal,
    Preview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextTab {
    Files,
    Diff,
    Review,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusTarget {
    Terminal,
    TaskList,
    ContextPane,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkbenchCommand {
    ActivateTask(TaskId),
    SetPaneRoute(PaneRoute),
    SetContextTab(ContextTab),
    SetContextFilter(String),
    AppendContextFilter(String),
    BackspaceContextFilter,
    ClearContextFilter,
    OpenChangedFile(String),
    AppendTaskTitleDraft(String),
    BackspaceTaskTitleDraft,
    ClearTaskTitleDraft,
    SelectReviewTarget(ReviewDraftTarget),
    AppendReviewDraft(String),
    BackspaceReviewDraft,
    ClearReviewDraft,
    SubmitReviewDraft,
    CreateTask,
    LaunchAgent(TaskId),
    DeliverReview(TaskId),
    AttachWorktreePreview(TaskId),
    OpenPreviewTarget {
        task_id: TaskId,
        target_id: PreviewTargetId,
    },
    WriteTerminal(TerminalSessionId, Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewDraftTarget {
    pub task_id: TaskId,
    pub line: LineIdentity,
    pub selected_text: Option<String>,
}

impl ReviewDraftTarget {
    pub fn path(&self) -> &str {
        &self.line.path
    }

    pub fn line_label(&self) -> String {
        let line_number = match self.line.side {
            relay_core::DiffSide::Old => self.line.old_line,
            relay_core::DiffSide::New => self.line.new_line,
        };

        line_number
            .map(|number| format!("{:?} line {}", self.line.side, number))
            .unwrap_or_else(|| "file".to_string())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReviewDraftState {
    pub target: Option<ReviewDraftTarget>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskListRow {
    Group { label: String, count: usize },
    Task(Box<TaskListItem>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskListItem {
    pub task: TaskProjection,
    pub active: bool,
    pub agent_label: String,
    pub worktree_label: String,
    pub changed_label: String,
    pub review_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceStatusSummary {
    pub task_count: usize,
    pub active_count: usize,
    pub attention_count: usize,
    pub review_count: usize,
    pub changed_file_count: usize,
    pub pending_review_count: usize,
    pub attached_terminal_count: usize,
    pub active_agent_count: usize,
    pub runtime_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceViewModel {
    pub project_label: String,
    pub tasks: Vec<TaskProjection>,
    pub active_task_id: Option<TaskId>,
    pub pane_route: PaneRoute,
    pub context_tab: ContextTab,
    pub task_title_draft: String,
    pub context_filter: String,
    pub review_draft: ReviewDraftState,
    pub focus: FocusTarget,
}

impl WorkspaceViewModel {
    pub fn new(tasks: Vec<TaskProjection>) -> Self {
        Self::for_project("Relay".to_string(), tasks)
    }

    pub fn for_project(project_label: String, tasks: Vec<TaskProjection>) -> Self {
        Self {
            project_label,
            active_task_id: tasks.first().map(|task| task.id),
            tasks,
            pane_route: PaneRoute::Terminal,
            context_tab: ContextTab::Files,
            task_title_draft: String::new(),
            context_filter: String::new(),
            review_draft: ReviewDraftState::default(),
            focus: FocusTarget::Terminal,
        }
    }

    pub fn active_task(&self) -> Option<&TaskProjection> {
        let active_task_id = self.active_task_id?;
        self.tasks.iter().find(|task| task.id == active_task_id)
    }

    pub fn active_worktree_label(&self) -> String {
        self.active_task()
            .and_then(|task| task.worktree_path.as_deref())
            .map(worktree_name)
            .unwrap_or_else(|| "no worktree".to_string())
    }

    pub fn active_worktree_path_label(&self) -> String {
        self.active_task()
            .and_then(|task| task.worktree_path.as_deref())
            .map(str::to_string)
            .unwrap_or_else(|| "No worktree attached".to_string())
    }

    pub fn active_branch_label(&self) -> String {
        self.active_task()
            .and_then(|task| task.worktree_branch.as_deref())
            .map(str::to_string)
            .unwrap_or_else(|| "no branch".to_string())
    }

    pub fn focus_label(&self) -> &'static str {
        match self.focus {
            FocusTarget::Terminal => "Terminal",
            FocusTarget::TaskList => "Tasks",
            FocusTarget::ContextPane => "Context",
        }
    }

    pub fn status_summary(&self) -> WorkspaceStatusSummary {
        let mut active_count = 0;
        let mut attention_count = 0;
        let mut review_count = 0;
        let mut changed_file_count = 0;
        let mut pending_review_count = 0;
        let mut attached_terminal_count = 0;
        let mut active_agent_count = 0;

        for task in &self.tasks {
            match task_bucket(task) {
                TaskBucket::Active => active_count += 1,
                TaskBucket::Attention => attention_count += 1,
                TaskBucket::Review => review_count += 1,
            }
            changed_file_count += task.changed_file_count;
            pending_review_count += task.pending_review_comment_count;
            if task.has_terminal {
                attached_terminal_count += 1;
            }
            if task.agent.is_some()
                && !matches!(
                    task.status,
                    TaskStatus::ReadyForAgent
                        | TaskStatus::Done
                        | TaskStatus::Archived
                        | TaskStatus::Failed
                )
            {
                active_agent_count += 1;
            }
        }

        let runtime_label = if attached_terminal_count == 0 {
            "no terminal".to_string()
        } else {
            count_label(attached_terminal_count, "terminal", "terminals")
        };

        WorkspaceStatusSummary {
            task_count: self.tasks.len(),
            active_count,
            attention_count,
            review_count,
            changed_file_count,
            pending_review_count,
            attached_terminal_count,
            active_agent_count,
            runtime_label,
        }
    }

    pub fn can_submit_review_draft(&self) -> bool {
        self.review_draft.target.is_some() && !self.review_draft.body.trim().is_empty()
    }

    pub fn can_create_task_from_draft(&self) -> bool {
        !self.task_title_draft.trim().is_empty()
    }

    pub fn apply_command(&mut self, command: WorkbenchCommand) {
        match command {
            WorkbenchCommand::ActivateTask(task_id) => {
                if self.tasks.iter().any(|task| task.id == task_id) {
                    self.active_task_id = Some(task_id);
                    if self
                        .review_draft
                        .target
                        .as_ref()
                        .is_some_and(|target| target.task_id != task_id)
                    {
                        self.review_draft = ReviewDraftState::default();
                    }
                    self.focus = FocusTarget::Terminal;
                }
            }
            WorkbenchCommand::SetPaneRoute(route) => {
                self.pane_route = route;
                self.focus = match route {
                    PaneRoute::Terminal => FocusTarget::Terminal,
                    PaneRoute::Preview => FocusTarget::ContextPane,
                };
            }
            WorkbenchCommand::SetContextTab(tab) => {
                self.context_tab = tab;
                self.focus = FocusTarget::ContextPane;
            }
            WorkbenchCommand::SetContextFilter(filter) => {
                self.context_filter = filter;
                self.focus = FocusTarget::ContextPane;
            }
            WorkbenchCommand::AppendContextFilter(text) => {
                self.context_filter.push_str(&text);
                self.focus = FocusTarget::ContextPane;
            }
            WorkbenchCommand::BackspaceContextFilter => {
                self.context_filter.pop();
                self.focus = FocusTarget::ContextPane;
            }
            WorkbenchCommand::ClearContextFilter => {
                self.context_filter.clear();
                self.focus = FocusTarget::ContextPane;
            }
            WorkbenchCommand::OpenChangedFile(path) => {
                self.context_filter = path;
                self.context_tab = ContextTab::Diff;
                self.focus = FocusTarget::ContextPane;
            }
            WorkbenchCommand::AppendTaskTitleDraft(text) => {
                self.task_title_draft.push_str(&text);
                self.focus = FocusTarget::TaskList;
            }
            WorkbenchCommand::BackspaceTaskTitleDraft => {
                self.task_title_draft.pop();
                self.focus = FocusTarget::TaskList;
            }
            WorkbenchCommand::ClearTaskTitleDraft => {
                self.task_title_draft.clear();
                self.focus = FocusTarget::TaskList;
            }
            WorkbenchCommand::SelectReviewTarget(target) => {
                if self.tasks.iter().any(|task| task.id == target.task_id) {
                    self.active_task_id = Some(target.task_id);
                    self.review_draft.target = Some(target);
                    self.context_tab = ContextTab::Review;
                    self.focus = FocusTarget::ContextPane;
                }
            }
            WorkbenchCommand::AppendReviewDraft(text) => {
                self.review_draft.body.push_str(&text);
                self.focus = FocusTarget::ContextPane;
            }
            WorkbenchCommand::BackspaceReviewDraft => {
                self.review_draft.body.pop();
                self.focus = FocusTarget::ContextPane;
            }
            WorkbenchCommand::ClearReviewDraft => {
                self.review_draft = ReviewDraftState::default();
                self.focus = FocusTarget::ContextPane;
            }
            WorkbenchCommand::SubmitReviewDraft => {}
            WorkbenchCommand::CreateTask => {}
            WorkbenchCommand::LaunchAgent(_) => {}
            WorkbenchCommand::DeliverReview(_) => {}
            WorkbenchCommand::AttachWorktreePreview(_) => {}
            WorkbenchCommand::OpenPreviewTarget { .. } => {}
            WorkbenchCommand::WriteTerminal(_, _) => {}
        }
    }

    pub fn task_list_rows(&self) -> Vec<TaskListRow> {
        let mut rows = Vec::new();
        let mut active = Vec::new();
        let mut waiting = Vec::new();
        let mut review = Vec::new();

        for task in &self.tasks {
            match task_bucket(task) {
                TaskBucket::Active => active.push(task),
                TaskBucket::Attention => waiting.push(task),
                TaskBucket::Review => review.push(task),
            }
        }

        append_group_rows(&mut rows, "Active", active, self.active_task_id);
        append_group_rows(&mut rows, "Needs attention", waiting, self.active_task_id);
        append_group_rows(&mut rows, "Review", review, self.active_task_id);
        rows
    }
}

fn append_group_rows(
    rows: &mut Vec<TaskListRow>,
    label: &str,
    tasks: Vec<&TaskProjection>,
    active_task_id: Option<TaskId>,
) {
    if tasks.is_empty() {
        return;
    }

    rows.push(TaskListRow::Group {
        label: label.to_string(),
        count: tasks.len(),
    });
    rows.extend(tasks.into_iter().map(|task| {
        TaskListRow::Task(Box::new(TaskListItem {
            task: task.clone(),
            active: Some(task.id) == active_task_id,
            agent_label: task
                .agent
                .as_ref()
                .map(|agent| agent.label().to_string())
                .unwrap_or_else(|| "no agent".to_string()),
            worktree_label: task
                .worktree_path
                .as_deref()
                .map(worktree_name)
                .unwrap_or_else(|| "no worktree".to_string()),
            changed_label: count_label(task.changed_file_count, "change", "changes"),
            review_label: count_label(task.pending_review_comment_count, "note", "notes"),
        }))
    }));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskBucket {
    Active,
    Attention,
    Review,
}

fn task_bucket(task: &TaskProjection) -> TaskBucket {
    if task.review_comment_count > 0 || task.pending_review_comment_count > 0 {
        TaskBucket::Review
    } else if matches!(
        task.status,
        TaskStatus::WaitingForUser | TaskStatus::Stale | TaskStatus::Blocked | TaskStatus::Failed
    ) {
        TaskBucket::Attention
    } else {
        TaskBucket::Active
    }
}

fn worktree_name(path: &str) -> String {
    path.rsplit(['\\', '/'])
        .find(|segment| !segment.is_empty())
        .unwrap_or(path)
        .to_string()
}

fn count_label(count: usize, singular: &str, plural: &str) -> String {
    match count {
        0 => "0".to_string(),
        1 => format!("1 {singular}"),
        value => format!("{value} {plural}"),
    }
}

#[cfg(test)]
mod tests {
    use relay_core::{
        AgentKind, AgentRuntimeStatus, AgentStatusUpdate, CreateTask, DiffSide, LineIdentity,
        ProjectId, Task, TaskCommand, TaskProjection, TaskSource, TerminalSessionId, Timestamp,
        WorktreeId, WorktreeSnapshot,
    };

    use super::*;

    #[test]
    fn task_list_rows_should_group_working_waiting_and_review_tasks() {
        let tasks = vec![
            demo_projection("Working", AgentRuntimeStatus::Working, 0),
            demo_projection("Waiting", AgentRuntimeStatus::Waiting, 0),
            demo_projection("Review", AgentRuntimeStatus::Working, 2),
        ];
        let view_model = WorkspaceViewModel::new(tasks);
        let rows = view_model.task_list_rows();
        let group_labels: Vec<_> = rows
            .iter()
            .filter_map(|row| match row {
                TaskListRow::Group { label, .. } => Some(label.as_str()),
                TaskListRow::Task(_) => None,
            })
            .collect();

        assert_eq!(group_labels, vec!["Active", "Needs attention", "Review"]);
    }

    #[test]
    fn apply_command_should_switch_active_task_and_focus_terminal() {
        let tasks = vec![
            demo_projection("One", AgentRuntimeStatus::Working, 0),
            demo_projection("Two", AgentRuntimeStatus::Waiting, 0),
        ];
        let target_id = tasks[1].id;
        let mut view_model = WorkspaceViewModel::new(tasks);

        view_model.apply_command(WorkbenchCommand::ActivateTask(target_id));

        assert_eq!(view_model.active_task_id, Some(target_id));
        assert_eq!(view_model.focus, FocusTarget::Terminal);
    }

    #[test]
    fn status_summary_should_count_runtime_and_review_state() {
        let tasks = vec![
            demo_projection("Working", AgentRuntimeStatus::Working, 0),
            demo_projection("Waiting", AgentRuntimeStatus::Waiting, 0),
            demo_projection("Review", AgentRuntimeStatus::Working, 2),
        ];
        let view_model = WorkspaceViewModel::new(tasks);
        let summary = view_model.status_summary();

        assert_eq!(
            summary,
            WorkspaceStatusSummary {
                task_count: 3,
                active_count: 1,
                attention_count: 1,
                review_count: 1,
                changed_file_count: 0,
                pending_review_count: 2,
                attached_terminal_count: 3,
                active_agent_count: 3,
                runtime_label: "3 terminals".to_string(),
            }
        );
    }

    #[test]
    fn apply_command_should_switch_route_and_context_tab() {
        let mut view_model =
            WorkspaceViewModel::new(vec![demo_projection("One", AgentRuntimeStatus::Working, 0)]);

        view_model.apply_command(WorkbenchCommand::SetPaneRoute(PaneRoute::Preview));
        view_model.apply_command(WorkbenchCommand::SetContextTab(ContextTab::Review));

        assert_eq!(view_model.pane_route, PaneRoute::Preview);
        assert_eq!(view_model.context_tab, ContextTab::Review);
    }

    #[test]
    fn context_filter_commands_should_update_query_and_focus_context() {
        let mut view_model =
            WorkspaceViewModel::new(vec![demo_projection("One", AgentRuntimeStatus::Working, 0)]);

        view_model.apply_command(WorkbenchCommand::SetContextFilter("app".to_string()));
        view_model.apply_command(WorkbenchCommand::AppendContextFilter("_shell".to_string()));
        view_model.apply_command(WorkbenchCommand::BackspaceContextFilter);

        assert_eq!(view_model.context_filter, "app_shel");
        assert_eq!(view_model.focus, FocusTarget::ContextPane);

        view_model.apply_command(WorkbenchCommand::ClearContextFilter);
        assert!(view_model.context_filter.is_empty());
    }

    #[test]
    fn open_changed_file_should_filter_diff_and_focus_context() {
        let mut view_model =
            WorkspaceViewModel::new(vec![demo_projection("One", AgentRuntimeStatus::Working, 0)]);

        view_model.apply_command(WorkbenchCommand::OpenChangedFile(
            "crates/relay_ui/src/diff_pane.rs".to_string(),
        ));

        assert_eq!(view_model.context_tab, ContextTab::Diff);
        assert_eq!(
            view_model.context_filter,
            "crates/relay_ui/src/diff_pane.rs"
        );
        assert_eq!(view_model.focus, FocusTarget::ContextPane);
    }

    #[test]
    fn task_title_draft_commands_should_track_title_and_focus_tasks() {
        let mut view_model =
            WorkspaceViewModel::new(vec![demo_projection("One", AgentRuntimeStatus::Working, 0)]);

        view_model.apply_command(WorkbenchCommand::AppendTaskTitleDraft(
            "Implement parser".to_string(),
        ));
        view_model.apply_command(WorkbenchCommand::BackspaceTaskTitleDraft);

        assert_eq!(view_model.task_title_draft, "Implement parse");
        assert_eq!(view_model.focus, FocusTarget::TaskList);
        assert!(view_model.can_create_task_from_draft());

        view_model.apply_command(WorkbenchCommand::ClearTaskTitleDraft);
        assert!(view_model.task_title_draft.is_empty());
    }

    #[test]
    fn review_draft_commands_should_select_target_and_track_body() {
        let task = demo_projection("Review target", AgentRuntimeStatus::Working, 0);
        let task_id = task.id;
        let mut view_model = WorkspaceViewModel::new(vec![task]);

        view_model.apply_command(WorkbenchCommand::SelectReviewTarget(review_target(task_id)));
        view_model.apply_command(WorkbenchCommand::AppendReviewDraft(
            "Tighten this branch.".to_string(),
        ));

        assert_eq!(view_model.context_tab, ContextTab::Review);
        assert_eq!(view_model.review_draft.body, "Tighten this branch.");
        assert!(view_model.can_submit_review_draft());
    }

    #[test]
    fn activate_task_should_clear_review_draft_for_previous_task() {
        let tasks = vec![
            demo_projection("One", AgentRuntimeStatus::Working, 0),
            demo_projection("Two", AgentRuntimeStatus::Waiting, 0),
        ];
        let first_id = tasks[0].id;
        let second_id = tasks[1].id;
        let mut view_model = WorkspaceViewModel::new(tasks);
        view_model.apply_command(WorkbenchCommand::SelectReviewTarget(review_target(
            first_id,
        )));
        view_model.apply_command(WorkbenchCommand::AppendReviewDraft(
            "Carry over?".to_string(),
        ));

        view_model.apply_command(WorkbenchCommand::ActivateTask(second_id));

        assert_eq!(view_model.active_task_id, Some(second_id));
        assert_eq!(view_model.review_draft, ReviewDraftState::default());
    }

    fn demo_projection(
        title: &str,
        state: AgentRuntimeStatus,
        review_count: usize,
    ) -> TaskProjection {
        let now = Timestamp::UNIX_EPOCH;
        let (mut task, _) = Task::create(CreateTask {
            id: None,
            project_id: ProjectId::new(),
            title: title.to_string(),
            source: TaskSource::Manual,
            now,
        })
        .expect("task should create");

        apply(
            &mut task,
            TaskCommand::AttachWorktree {
                snapshot: WorktreeSnapshot {
                    id: WorktreeId::new(),
                    path: "F:\\Workspace\\Relay".to_string(),
                    branch: "task/demo".to_string(),
                    base_ref: Some("origin/master".to_string()),
                },
                now,
            },
        );
        apply(
            &mut task,
            TaskCommand::AttachTerminal {
                id: TerminalSessionId::new(),
                now,
            },
        );
        apply(
            &mut task,
            TaskCommand::AttachAgent {
                id: relay_core::AgentSessionId::new(),
                kind: AgentKind::Codex,
                started_at: now,
            },
        );
        apply(
            &mut task,
            TaskCommand::ApplyAgentStatus(AgentStatusUpdate {
                state,
                prompt: title.to_string(),
                agent_kind: Some(AgentKind::Codex),
                observed_at: now,
            }),
        );
        for index in 0..review_count {
            let task_id = task.id;
            apply(
                &mut task,
                TaskCommand::AddReviewComment(relay_core::ReviewComment {
                    id: relay_core::ReviewCommentId::new(),
                    task_id,
                    path: format!("src/{index}.rs"),
                    line: None,
                    selected_range: None,
                    body: "Needs follow-up".to_string(),
                    created_at: now,
                }),
            );
        }

        TaskProjection::from_task(&task)
    }

    fn review_target(task_id: TaskId) -> ReviewDraftTarget {
        ReviewDraftTarget {
            task_id,
            line: LineIdentity {
                path: "src/lib.rs".to_string(),
                side: DiffSide::New,
                old_line: None,
                new_line: Some(42),
                hunk_header: "@@ -40,1 +42,1 @@".to_string(),
            },
            selected_text: Some("let value = compute();".to_string()),
        }
    }

    fn apply(task: &mut Task, command: TaskCommand) {
        for event in task.handle(command).expect("command should be valid") {
            task.apply(&event).expect("event should apply");
        }
    }
}
