use gpui::{
    App, Bounds, Context, IntoElement, Render, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, size,
};
use relay_core::{
    AgentKind, AgentRuntimeStatus, AgentSessionId, AgentStatusUpdate, ChangeStatus, ChangedFile,
    CreateTask, PreviewTarget, PreviewTargetId, ProjectId, ReviewComment, ReviewCommentId, Task,
    TaskCommand, TaskProjection, TaskSource, TerminalSessionId, Timestamp, WorktreeId,
    WorktreeSnapshot,
};

use crate::{
    diff_pane::context_pane,
    task_list::task_list,
    terminal_pane::{TerminalPaneProjection, terminal_pane},
    theme::RelayTheme,
    workbench::WorkspaceViewModel,
};

pub struct AppShell {
    theme: RelayTheme,
    view_model: WorkspaceViewModel,
}

impl AppShell {
    pub fn open(cx: &mut App) -> anyhow::Result<()> {
        let bounds = Bounds::centered(None, size(px(1180.0), px(780.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(Default::default()),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Self::new()),
        )?;
        cx.activate(true);
        Ok(())
    }

    fn new() -> Self {
        Self {
            theme: RelayTheme::dark(),
            view_model: WorkspaceViewModel::new(demo_task_projections()),
        }
    }

    fn header(&self) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(44.0))
            .px_4()
            .border_b_1()
            .border_color(self.theme.line)
            .bg(self.theme.chrome_alt)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(self.theme.text)
                            .child("Relay"),
                    )
                    .child(div().text_sm().text_color(self.theme.muted).child(format!(
                        "{} / {}",
                        self.view_model.project_label, self.view_model.branch_label
                    ))),
            )
            .child(
                div().flex().items_center().gap_3().child(
                    div()
                        .text_sm()
                        .text_color(self.theme.muted)
                        .child("New task / Agent / Review"),
                ),
            )
    }

    fn terminal_projection(&self) -> TerminalPaneProjection {
        let Some(active_task) = self.view_model.active_task() else {
            return TerminalPaneProjection::detached();
        };

        TerminalPaneProjection {
            session_id: active_task.terminal_session_id,
            cwd: active_task
                .worktree_path
                .clone()
                .unwrap_or_else(|| "F:\\Workspace\\Relay".to_string()),
            title: active_task
                .agent
                .as_ref()
                .map(|kind| format!("{kind:?} session")),
            scrollback: format!(
                "relay $ attach-terminal {}\nrelay $ agent status: {}\n{}",
                active_task
                    .terminal_session_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                active_task.status_label,
                active_task.agent_prompt
            ),
            exited: false,
        }
    }
}

impl Render for AppShell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(self.theme.bg)
            .flex()
            .flex_col()
            .child(self.header())
            .child(
                div()
                    .flex()
                    .flex_1()
                    .child(task_list(self.theme, &self.view_model))
                    .child(terminal_pane(
                        self.theme,
                        &self.view_model,
                        &self.terminal_projection(),
                    ))
                    .child(context_pane(self.theme, &self.view_model)),
            )
    }
}

fn demo_task_projections() -> Vec<TaskProjection> {
    let project_id = ProjectId::new();
    let now = Timestamp::UNIX_EPOCH;

    let mut working = create_demo_task(project_id, "Design GPUI shell", now);
    apply_demo_event(
        &mut working,
        TaskCommand::AttachWorktree {
            snapshot: WorktreeSnapshot {
                id: WorktreeId::new(),
                path: "F:\\Workspace\\Relay".to_string(),
                branch: "main".to_string(),
                base_ref: Some("origin/master".to_string()),
            },
            now,
        },
    );
    apply_demo_event(
        &mut working,
        TaskCommand::AttachTerminal {
            id: TerminalSessionId::new(),
            now,
        },
    );
    apply_demo_event(
        &mut working,
        TaskCommand::AttachAgent {
            id: AgentSessionId::new(),
            kind: AgentKind::Claude,
            started_at: now,
        },
    );
    apply_demo_event(
        &mut working,
        TaskCommand::ApplyAgentStatus(AgentStatusUpdate {
            state: AgentRuntimeStatus::Working,
            prompt: "Build GPUI shell".to_string(),
            agent_kind: Some(AgentKind::Claude),
            observed_at: now,
        }),
    );
    apply_demo_event(
        &mut working,
        TaskCommand::RefreshChangedFiles {
            files: vec![
                ChangedFile {
                    path: "crates/relay_ui/src/app_shell.rs".to_string(),
                    status: ChangeStatus::Modified,
                },
                ChangedFile {
                    path: "crates/relay_core/src/task.rs".to_string(),
                    status: ChangeStatus::Added,
                },
                ChangedFile {
                    path: "crates/relay_core/src/task_event.rs".to_string(),
                    status: ChangeStatus::Added,
                },
            ],
            now,
        },
    );

    let mut waiting = create_demo_task(project_id, "Codex provider spike", now);
    apply_demo_event(
        &mut waiting,
        TaskCommand::AttachWorktree {
            snapshot: WorktreeSnapshot {
                id: WorktreeId::new(),
                path: "F:\\Workspace\\Relay\\.worktrees\\codex-spike".to_string(),
                branch: "task/codex-provider".to_string(),
                base_ref: Some("origin/master".to_string()),
            },
            now,
        },
    );
    apply_demo_event(
        &mut waiting,
        TaskCommand::AttachTerminal {
            id: TerminalSessionId::new(),
            now,
        },
    );
    apply_demo_event(
        &mut waiting,
        TaskCommand::AttachAgent {
            id: AgentSessionId::new(),
            kind: AgentKind::Codex,
            started_at: now,
        },
    );
    apply_demo_event(
        &mut waiting,
        TaskCommand::ApplyAgentStatus(AgentStatusUpdate {
            state: AgentRuntimeStatus::Waiting,
            prompt: "Probe Codex CLI launch".to_string(),
            agent_kind: Some(AgentKind::Codex),
            observed_at: now,
        }),
    );

    let mut reviewing = create_demo_task(project_id, "Diff review model", now);
    let reviewing_id = reviewing.id;
    apply_demo_event(
        &mut reviewing,
        TaskCommand::AddReviewComment(ReviewComment {
            id: ReviewCommentId::new(),
            task_id: reviewing_id,
            path: "crates/relay_diff/src/lib.rs".to_string(),
            line: None,
            selected_range: None,
            body: "Keep review comments task-scoped.".to_string(),
            created_at: now,
        }),
    );
    apply_demo_event(
        &mut reviewing,
        TaskCommand::AttachPreview {
            target: PreviewTarget {
                id: PreviewTargetId::new(),
                label: "Relay shell".to_string(),
                uri: "relay://preview/app-shell".to_string(),
            },
            now,
        },
    );

    vec![
        TaskProjection::from_task(&working),
        TaskProjection::from_task(&waiting),
        TaskProjection::from_task(&reviewing),
    ]
}

fn create_demo_task(project_id: ProjectId, title: &str, now: Timestamp) -> Task {
    let (task, _) = Task::create(CreateTask {
        id: None,
        project_id,
        title: title.to_string(),
        source: TaskSource::Manual,
        now,
    })
    .expect("demo task should be valid");
    task
}

fn apply_demo_event(task: &mut Task, command: TaskCommand) {
    for event in task
        .handle(command)
        .expect("demo transition should be valid")
    {
        task.apply(&event).expect("demo event should apply");
    }
}
