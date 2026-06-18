use relay_core::{AgentKind, TaskId, TaskProjection};

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
    TaskList,
    Terminal,
    ContextPane,
    CommandPalette,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkbenchCommand {
    ActivateTask(TaskId),
    SetPaneRoute(PaneRoute),
    SetContextTab(ContextTab),
    Focus(FocusTarget),
    ToggleCommandPalette,
    CreateTask,
    LaunchAgent(AgentKind),
    SendReviewToAgent,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceViewModel {
    pub project_label: String,
    pub branch_label: String,
    pub tasks: Vec<TaskProjection>,
    pub active_task_id: Option<TaskId>,
    pub pane_route: PaneRoute,
    pub context_tab: ContextTab,
    pub focus: FocusTarget,
    pub command_palette_open: bool,
    pub last_command: Option<WorkbenchCommand>,
}

impl WorkspaceViewModel {
    pub fn new(tasks: Vec<TaskProjection>) -> Self {
        Self {
            project_label: "Relay".to_string(),
            branch_label: "master".to_string(),
            active_task_id: tasks.first().map(|task| task.id),
            tasks,
            pane_route: PaneRoute::Terminal,
            context_tab: ContextTab::Files,
            focus: FocusTarget::Terminal,
            command_palette_open: false,
            last_command: None,
        }
    }

    pub fn active_task(&self) -> Option<&TaskProjection> {
        let active_task_id = self.active_task_id?;
        self.tasks.iter().find(|task| task.id == active_task_id)
    }

    pub fn apply_command(&mut self, command: WorkbenchCommand) {
        match &command {
            WorkbenchCommand::ActivateTask(task_id) => {
                if self.tasks.iter().any(|task| task.id == *task_id) {
                    self.active_task_id = Some(*task_id);
                    self.focus = FocusTarget::Terminal;
                }
            }
            WorkbenchCommand::SetPaneRoute(route) => {
                self.pane_route = *route;
                self.focus = match route {
                    PaneRoute::Terminal => FocusTarget::Terminal,
                    PaneRoute::Preview => FocusTarget::ContextPane,
                };
            }
            WorkbenchCommand::SetContextTab(tab) => {
                self.context_tab = *tab;
                self.focus = FocusTarget::ContextPane;
            }
            WorkbenchCommand::Focus(target) => {
                self.focus = *target;
            }
            WorkbenchCommand::ToggleCommandPalette => {
                self.command_palette_open = !self.command_palette_open;
                self.focus = if self.command_palette_open {
                    FocusTarget::CommandPalette
                } else {
                    FocusTarget::Terminal
                };
            }
            WorkbenchCommand::CreateTask
            | WorkbenchCommand::LaunchAgent(_)
            | WorkbenchCommand::SendReviewToAgent => {}
        }
        self.last_command = Some(command);
    }

    pub fn task_list_rows(&self) -> Vec<TaskListRow> {
        let mut rows = Vec::new();
        let mut active = Vec::new();
        let mut waiting = Vec::new();
        let mut review = Vec::new();

        for task in &self.tasks {
            if task.review_comment_count > 0 || task.pending_review_comment_count > 0 {
                review.push(task);
            } else if matches!(task.status_label.as_str(), "WAITING" | "STALE" | "BLOCKED") {
                waiting.push(task);
            } else {
                active.push(task);
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
        }))
    }));
}

#[cfg(test)]
mod tests {
    use relay_core::{
        AgentKind, AgentRuntimeStatus, AgentStatusUpdate, CreateTask, ProjectId, Task, TaskCommand,
        TaskProjection, TaskSource, TerminalSessionId, Timestamp, WorktreeId, WorktreeSnapshot,
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
    fn command_palette_toggle_should_move_focus() {
        let mut view_model =
            WorkspaceViewModel::new(vec![demo_projection("One", AgentRuntimeStatus::Working, 0)]);

        view_model.apply_command(WorkbenchCommand::ToggleCommandPalette);

        assert!(view_model.command_palette_open);
        assert_eq!(view_model.focus, FocusTarget::CommandPalette);
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

    fn apply(task: &mut Task, command: TaskCommand) {
        for event in task.handle(command).expect("command should be valid") {
            task.apply(&event).expect("event should apply");
        }
    }
}
