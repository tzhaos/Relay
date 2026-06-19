use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use gpui::{
    App, Bounds, Context, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    PathPromptOptions, Render, Task as GpuiTask, Window, WindowBounds, WindowControlArea,
    WindowDecorations, WindowOptions, div, prelude::*, px, size,
};
use relay_core::{PreviewTargetId, TaskId, TaskProjection, TerminalSessionId};

use crate::{
    components::{ComposerKey, composer_key_handler},
    diff_pane::context_pane,
    task_list::task_list,
    terminal_pane::{TerminalPaneProjection, terminal_pane},
    theme::{RelayTheme, spacing},
    workbench::{PaneRoute, ReviewDraftTarget, WorkbenchCommand, WorkspaceViewModel},
};

pub struct AppShell {
    theme: RelayTheme,
    view_model: WorkspaceViewModel,
    task_data_source: Box<dyn TaskDataSource>,
    terminal_focus: FocusHandle,
    task_title_focus: FocusHandle,
    context_filter_focus: FocusHandle,
    review_draft_focus: FocusHandle,
    last_error: Option<String>,
    _runtime_poll_task: GpuiTask<()>,
}

pub trait TaskDataSource {
    fn open_project(&mut self, path: &Path) -> anyhow::Result<WorkspaceData>;
    fn refresh_changed_files(&mut self) -> anyhow::Result<Vec<TaskProjection>>;
    fn create_task(&mut self, title: &str) -> anyhow::Result<Vec<TaskProjection>>;
    fn launch_agent(&mut self, task_id: TaskId) -> anyhow::Result<Vec<TaskProjection>>;
    fn launch_agent_terminal(&mut self, session_id: TerminalSessionId) -> anyhow::Result<()>;
    fn deliver_review(&mut self, task_id: TaskId) -> anyhow::Result<Vec<TaskProjection>>;
    fn archive_task(&mut self, task_id: TaskId) -> anyhow::Result<Vec<TaskProjection>>;
    fn add_review_comment(
        &mut self,
        target: ReviewDraftTarget,
        body: &str,
    ) -> anyhow::Result<Vec<TaskProjection>>;
    fn attach_worktree_preview(&mut self, task_id: TaskId) -> anyhow::Result<Vec<TaskProjection>>;
    fn attach_file_preview(
        &mut self,
        task_id: TaskId,
        path: &str,
    ) -> anyhow::Result<Vec<TaskProjection>>;
    fn open_preview_target(
        &mut self,
        task_id: TaskId,
        target_id: PreviewTargetId,
    ) -> anyhow::Result<()>;
    fn write_terminal(&mut self, session_id: TerminalSessionId, bytes: &[u8])
    -> anyhow::Result<()>;
    fn poll_runtime(&mut self) -> anyhow::Result<bool>;
    fn terminal_projection(
        &mut self,
        session_id: TerminalSessionId,
    ) -> anyhow::Result<Option<TerminalPaneProjection>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceData {
    pub project_label: String,
    pub project_open: bool,
    pub workspace_terminal_session_id: Option<TerminalSessionId>,
    pub tasks: Vec<TaskProjection>,
}

impl WorkspaceData {
    pub fn detached() -> Self {
        Self {
            project_label: "No project".to_string(),
            project_open: false,
            workspace_terminal_session_id: None,
            tasks: Vec::new(),
        }
    }

    pub fn for_project(
        project_label: String,
        workspace_terminal_session_id: TerminalSessionId,
        tasks: Vec<TaskProjection>,
    ) -> Self {
        Self {
            project_label,
            project_open: true,
            workspace_terminal_session_id: Some(workspace_terminal_session_id),
            tasks,
        }
    }
}

impl AppShell {
    pub fn open(
        cx: &mut App,
        workspace: WorkspaceData,
        task_data_source: Box<dyn TaskDataSource>,
    ) -> anyhow::Result<()> {
        let bounds = Bounds::centered(None, size(px(1440.0), px(900.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                window_decorations: Some(WindowDecorations::Client),
                window_min_size: Some(size(px(1180.0), px(780.0))),
                app_id: Some("relay".to_string()),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Self::new(workspace, task_data_source, cx)),
        )?;
        cx.activate(true);
        Ok(())
    }

    fn new(
        workspace: WorkspaceData,
        task_data_source: Box<dyn TaskDataSource>,
        cx: &mut Context<Self>,
    ) -> Self {
        let runtime_poll_task = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
                if this
                    .update(cx, |this, cx| {
                        this.poll_runtime(cx);
                    })
                    .is_err()
                {
                    break;
                }
            }
        });

        Self {
            theme: RelayTheme::orca(),
            view_model: WorkspaceViewModel::from_workspace(
                workspace.project_label,
                workspace.project_open,
                workspace.workspace_terminal_session_id,
                workspace.tasks,
            ),
            task_data_source,
            terminal_focus: cx.focus_handle(),
            task_title_focus: cx.focus_handle(),
            context_filter_focus: cx.focus_handle(),
            review_draft_focus: cx.focus_handle(),
            last_error: None,
            _runtime_poll_task: runtime_poll_task,
        }
    }

    fn title_bar(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let summary = self.view_model.status_summary();
        let worktree_label = self.view_model.active_worktree_label();
        let branch_label = self.view_model.active_branch_label();
        let has_worktree = self
            .view_model
            .active_task()
            .is_some_and(|task| task.worktree_path.is_some());

        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(spacing::TITLE_BAR))
            .pl_3()
            .pr_1()
            .border_b_1()
            .border_color(self.theme.border)
            .bg(self.theme.chrome)
            .child(
                // Left: Relay identity + project name. Drag region for window moving.
                div()
                    .w(px(spacing::RAIL_WIDTH))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap_2()
                    .min_w_0()
                    .window_control_area(WindowControlArea::Drag)
                    .child(title_mark(self.theme))
                    .child(
                        div()
                            .min_w_0()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(self.theme.text)
                                    .child("Relay"),
                            )
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_sm()
                                    .text_color(self.theme.text_muted)
                                    .child(self.view_model.project_label.clone()),
                            ),
                    ),
            )
            .child(
                // Center: worktree + branch context badges (drag region). Shows
                // an operational hint when no worktree is active rather than a
                // raw "no worktree" label.
                div()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .window_control_area(WindowControlArea::Drag)
                    .child(title_badge(
                        self.theme,
                        if has_worktree {
                            worktree_label
                        } else {
                            "No active worktree".to_string()
                        },
                        has_worktree,
                    ))
                    .child(title_badge(
                        self.theme,
                        if has_worktree {
                            branch_label
                        } else {
                            format!("{} tasks", summary.task_count)
                        },
                        has_worktree,
                    )),
            )
            .child(
                // Right: Open Project action + window controls.
                div()
                    .flex_shrink_0()
                    .h_full()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(title_action_button(self.theme, "Open Project", cx))
                    .child(window_controls(self.theme, window)),
            )
    }

    fn terminal_projection(&mut self) -> TerminalPaneProjection {
        let task_terminal = self.view_model.active_task().and_then(|task| {
            task.terminal_session_id.map(|session_id| {
                (
                    session_id,
                    task.worktree_path.clone().unwrap_or_default(),
                    task.agent
                        .as_ref()
                        .map(|kind| format!("{} session", kind.label())),
                )
            })
        });
        let workspace_terminal = self
            .view_model
            .workspace_terminal_session_id
            .map(|session_id| {
                (
                    session_id,
                    self.view_model.project_label.clone(),
                    Some("Workspace terminal".to_string()),
                )
            });
        let Some((session_id, cwd, title)) = task_terminal.or(workspace_terminal) else {
            return TerminalPaneProjection::detached();
        };

        let mut projection = TerminalPaneProjection {
            session_id: Some(session_id),
            cwd,
            title,
            scrollback: String::new(),
            exited: false,
            connected: false,
        };

        match self.task_data_source.terminal_projection(session_id) {
            Ok(Some(mut runtime_projection)) => {
                if runtime_projection.title.is_none() {
                    runtime_projection.title = projection.title;
                }
                projection = runtime_projection;
                self.last_error = None;
            }
            Ok(None) => {}
            Err(error) => {
                self.last_error = Some(error.to_string());
            }
        }

        projection
    }

    pub(crate) fn dispatch(&mut self, command: WorkbenchCommand, cx: &mut Context<Self>) {
        if command == WorkbenchCommand::OpenProject {
            self.open_project_picker(cx);
            return;
        }
        if command == WorkbenchCommand::RefreshChangedFiles {
            self.refresh_changed_files(cx);
            return;
        }
        if command == WorkbenchCommand::CreateTask {
            self.create_task(cx);
            return;
        }
        if let WorkbenchCommand::LaunchAgent(task_id) = command {
            self.launch_agent(task_id, cx);
            return;
        }
        if let WorkbenchCommand::LaunchAgentTerminal(session_id) = command {
            self.launch_agent_terminal(session_id, cx);
            return;
        }
        if let WorkbenchCommand::DeliverReview(task_id) = command {
            self.deliver_review(task_id, cx);
            return;
        }
        if let WorkbenchCommand::ArchiveTask(task_id) = command {
            self.archive_task(task_id, cx);
            return;
        }
        if command == WorkbenchCommand::SubmitReviewDraft {
            self.submit_review_draft(cx);
            return;
        }
        if let WorkbenchCommand::AttachWorktreePreview(task_id) = command {
            self.attach_worktree_preview(task_id, cx);
            return;
        }
        if let WorkbenchCommand::AttachFilePreview { task_id, path } = command {
            self.attach_file_preview(task_id, &path, cx);
            return;
        }
        if let WorkbenchCommand::OpenPreviewTarget { task_id, target_id } = command {
            self.open_preview_target(task_id, target_id, cx);
            return;
        }
        if let WorkbenchCommand::WriteTerminal(session_id, bytes) = command {
            self.write_terminal(session_id, &bytes, cx);
            return;
        }

        self.view_model.apply_command(command);
        cx.notify();
    }

    fn open_project_picker(&mut self, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Open Project".into()),
        });

        cx.spawn(async move |this, cx| {
            let selected_path = match paths.await {
                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                Ok(Ok(None)) => None,
                Ok(Err(error)) => {
                    let _ = this.update(cx, |this, cx| {
                        this.last_error = Some(error.to_string());
                        cx.notify();
                    });
                    None
                }
                Err(error) => {
                    let _ = this.update(cx, |this, cx| {
                        this.last_error = Some(error.to_string());
                        cx.notify();
                    });
                    None
                }
            };

            if let Some(path) = selected_path {
                let _ = this.update(cx, |this, cx| {
                    this.open_project_path(path, cx);
                });
            }
        })
        .detach();
    }

    fn open_project_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        match self.task_data_source.open_project(&path) {
            Ok(workspace) => {
                self.view_model = WorkspaceViewModel::from_workspace(
                    workspace.project_label,
                    workspace.project_open,
                    workspace.workspace_terminal_session_id,
                    workspace.tasks,
                );
                self.last_error = None;
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
            }
        }
        cx.notify();
    }

    fn refresh_changed_files(&mut self, cx: &mut Context<Self>) {
        if !self.view_model.project_open {
            self.last_error = None;
            cx.notify();
            return;
        }

        match self.task_data_source.refresh_changed_files() {
            Ok(tasks) => {
                self.replace_tasks_preserving_active(tasks);
                self.last_error = None;
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
            }
        }
        cx.notify();
    }

    pub(crate) fn terminal_focus(&self) -> &FocusHandle {
        &self.terminal_focus
    }

    pub(crate) fn task_title_focus(&self) -> &FocusHandle {
        &self.task_title_focus
    }

    pub(crate) fn context_filter_focus(&self) -> &FocusHandle {
        &self.context_filter_focus
    }

    pub(crate) fn review_draft_focus(&self) -> &FocusHandle {
        &self.review_draft_focus
    }

    pub(crate) fn handle_context_filter_key(
        &mut self,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        match composer_key_handler(event, false) {
            Some(ComposerKey::Clear) => {
                self.dispatch(WorkbenchCommand::ClearContextFilter, cx);
                true
            }
            Some(ComposerKey::Backspace) => {
                self.dispatch(WorkbenchCommand::BackspaceContextFilter, cx);
                true
            }
            Some(ComposerKey::Append(text)) => {
                self.dispatch(WorkbenchCommand::AppendContextFilter(text), cx);
                true
            }
            // The context filter has no submit action (Enter is not handled here).
            Some(ComposerKey::Submit) | None => false,
        }
    }

    pub(crate) fn handle_task_title_key(
        &mut self,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        match composer_key_handler(event, true) {
            Some(ComposerKey::Clear) => {
                self.dispatch(WorkbenchCommand::ClearTaskTitleDraft, cx);
                true
            }
            Some(ComposerKey::Backspace) => {
                self.dispatch(WorkbenchCommand::BackspaceTaskTitleDraft, cx);
                true
            }
            Some(ComposerKey::Submit) => {
                self.dispatch(WorkbenchCommand::CreateTask, cx);
                true
            }
            Some(ComposerKey::Append(text)) => {
                self.dispatch(WorkbenchCommand::AppendTaskTitleDraft(text), cx);
                true
            }
            None => false,
        }
    }

    pub(crate) fn handle_review_draft_key(
        &mut self,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        match composer_key_handler(event, true) {
            Some(ComposerKey::Clear) => {
                self.dispatch(WorkbenchCommand::ClearReviewDraft, cx);
                true
            }
            Some(ComposerKey::Backspace) => {
                self.dispatch(WorkbenchCommand::BackspaceReviewDraft, cx);
                true
            }
            Some(ComposerKey::Submit) => {
                self.dispatch(WorkbenchCommand::SubmitReviewDraft, cx);
                true
            }
            Some(ComposerKey::Append(text)) => {
                self.dispatch(WorkbenchCommand::AppendReviewDraft(text), cx);
                true
            }
            None => false,
        }
    }

    fn create_task(&mut self, cx: &mut Context<Self>) {
        if !self.view_model.project_open {
            self.last_error = Some("Open a project before creating tasks.".to_string());
            cx.notify();
            return;
        }

        let title = self.view_model.task_title_draft.trim().to_string();
        if title.is_empty() {
            cx.notify();
            return;
        }

        match self.task_data_source.create_task(&title) {
            Ok(tasks) => {
                self.view_model.task_title_draft.clear();
                self.replace_tasks_preserving_active(tasks);
                self.last_error = None;
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
            }
        }
        cx.notify();
    }

    fn launch_agent(&mut self, task_id: TaskId, cx: &mut Context<Self>) {
        match self.task_data_source.launch_agent(task_id) {
            Ok(tasks) => {
                if !tasks.is_empty() {
                    let active_task_id = self.view_model.active_task_id;
                    self.replace_tasks_preserving_active(tasks);
                    if let Some(active_task_id) = active_task_id {
                        self.view_model
                            .apply_command(WorkbenchCommand::ActivateTask(active_task_id));
                    }
                }
                self.last_error = None;
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
            }
        }
        cx.notify();
    }

    fn launch_agent_terminal(&mut self, session_id: TerminalSessionId, cx: &mut Context<Self>) {
        match self.task_data_source.launch_agent_terminal(session_id) {
            Ok(()) => {
                self.last_error = None;
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
            }
        }
        cx.notify();
    }

    fn deliver_review(&mut self, task_id: TaskId, cx: &mut Context<Self>) {
        match self.task_data_source.deliver_review(task_id) {
            Ok(tasks) => {
                self.replace_tasks_preserving_active(tasks);
                self.last_error = None;
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
            }
        }
        cx.notify();
    }

    fn archive_task(&mut self, task_id: TaskId, cx: &mut Context<Self>) {
        match self.task_data_source.archive_task(task_id) {
            Ok(tasks) => {
                self.replace_tasks_preserving_active(tasks);
                self.last_error = None;
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
            }
        }
        cx.notify();
    }

    fn submit_review_draft(&mut self, cx: &mut Context<Self>) {
        let Some(target) = self.view_model.review_draft.target.clone() else {
            cx.notify();
            return;
        };
        let body = self.view_model.review_draft.body.trim().to_string();
        if body.is_empty() {
            cx.notify();
            return;
        }

        match self.task_data_source.add_review_comment(target, &body) {
            Ok(tasks) => {
                self.replace_tasks_preserving_active(tasks);
                self.view_model
                    .apply_command(WorkbenchCommand::ClearReviewDraft);
                self.last_error = None;
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
            }
        }
        cx.notify();
    }

    fn attach_worktree_preview(&mut self, task_id: TaskId, cx: &mut Context<Self>) {
        match self.task_data_source.attach_worktree_preview(task_id) {
            Ok(tasks) => {
                self.replace_tasks_preserving_active(tasks);
                self.last_error = None;
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
            }
        }
        cx.notify();
    }

    fn attach_file_preview(&mut self, task_id: TaskId, path: &str, cx: &mut Context<Self>) {
        match self.task_data_source.attach_file_preview(task_id, path) {
            Ok(tasks) => {
                self.replace_tasks_preserving_active(tasks);
                self.view_model
                    .apply_command(WorkbenchCommand::SetPaneRoute(PaneRoute::Preview));
                self.last_error = None;
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
            }
        }
        cx.notify();
    }

    fn open_preview_target(
        &mut self,
        task_id: TaskId,
        target_id: PreviewTargetId,
        cx: &mut Context<Self>,
    ) {
        match self
            .task_data_source
            .open_preview_target(task_id, target_id)
        {
            Ok(()) => {
                self.last_error = None;
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
            }
        }
        cx.notify();
    }

    fn replace_tasks_preserving_active(&mut self, tasks: Vec<TaskProjection>) {
        if tasks.is_empty() {
            return;
        }

        self.view_model.replace_tasks_preserving_active(tasks);
    }

    fn write_terminal(
        &mut self,
        session_id: TerminalSessionId,
        bytes: &[u8],
        cx: &mut Context<Self>,
    ) {
        match self.task_data_source.write_terminal(session_id, bytes) {
            Ok(()) => {
                if self.last_error.take().is_some() {
                    cx.notify();
                }
            }
            Err(error) => {
                self.last_error = Some(error.to_string());
                cx.notify();
            }
        }
    }

    fn poll_runtime(&mut self, cx: &mut Context<Self>) {
        match self.task_data_source.poll_runtime() {
            Ok(true) => {
                self.last_error = None;
                cx.notify();
            }
            Ok(false) => {}
            Err(error) => {
                self.last_error = Some(error.to_string());
                cx.notify();
            }
        }
    }

    fn status_bar(&self) -> impl IntoElement {
        let summary = self.view_model.status_summary();

        div()
            .h(px(spacing::STATUS_BAR))
            .flex_shrink_0()
            .px_3()
            .border_t_1()
            .border_color(self.theme.border)
            .bg(self.theme.chrome)
            .flex()
            .items_center()
            .justify_between()
            .text_xs()
            .child(
                div()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(status_bar_item(
                        self.theme,
                        "Runtime",
                        summary.runtime_label,
                    ))
                    .child(status_bar_item(
                        self.theme,
                        "Focus",
                        self.view_model.focus_label().to_string(),
                    ))
                    .child(status_bar_item(
                        self.theme,
                        "Worktree",
                        self.view_model.active_worktree_label(),
                    )),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(status_bar_item(
                        self.theme,
                        "Changes",
                        summary.changed_file_count.to_string(),
                    ))
                    .child(status_bar_item(
                        self.theme,
                        "Review",
                        summary.pending_review_count.to_string(),
                    ))
                    .children(self.last_error.as_ref().map(|error| {
                        status_bar_item(self.theme, "Error", error.clone()).into_any_element()
                    })),
            )
    }
}

impl Render for AppShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let terminal_projection = self.terminal_projection();

        div()
            .size_full()
            .bg(self.theme.app_bg)
            .flex()
            .flex_col()
            .child(self.title_bar(window, cx))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .child(task_list(
                        self.theme,
                        &self.view_model,
                        self.task_title_focus(),
                        self.terminal_focus(),
                        cx,
                    ))
                    .child(terminal_pane(
                        self.theme,
                        &self.view_model,
                        &terminal_projection,
                        self.terminal_focus(),
                        cx,
                    ))
                    .child(context_pane(
                        self.theme,
                        &self.view_model,
                        self.context_filter_focus(),
                        self.review_draft_focus(),
                        cx,
                    )),
            )
            .child(self.status_bar())
    }
}

fn title_mark(theme: RelayTheme) -> gpui::Div {
    div()
        .w(px(22.0))
        .h(px(22.0))
        .rounded_md()
        .bg(theme.accent)
        .flex()
        .items_center()
        .justify_center()
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(theme.terminal_text)
        .child("R")
}

/// A context badge in the title bar. `active` highlights the badge (green) when
/// there is a real worktree/branch; otherwise it renders quietly.
fn title_badge(theme: RelayTheme, label: String, active: bool) -> gpui::Div {
    let (bg, border, fg) = if active {
        (theme.accent_bg, theme.accent_border, theme.accent)
    } else {
        (theme.panel_alt, theme.border, theme.text_muted)
    };
    div()
        .h(px(24.0))
        .max_w(px(240.0))
        .px_2()
        .rounded_md()
        .border_1()
        .border_color(border)
        .bg(bg)
        .flex()
        .items_center()
        .gap_1()
        .text_sm()
        .font_weight(if active {
            gpui::FontWeight::SEMIBOLD
        } else {
            gpui::FontWeight::MEDIUM
        })
        .text_color(fg)
        .child(div().truncate().child(label))
}

fn title_action_button(
    theme: RelayTheme,
    label: &'static str,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    div()
        .h(px(26.0))
        .px_3()
        .rounded_md()
        .border_1()
        .border_color(theme.border_strong)
        .bg(theme.panel)
        .flex()
        .items_center()
        .text_sm()
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(theme.text)
        .cursor_pointer()
        .hover(|style| style.bg(theme.hover).border_color(theme.accent_border))
        .id("open-project")
        .on_click(cx.listener(|this, _: &gpui::ClickEvent, _, cx| {
            this.dispatch(WorkbenchCommand::OpenProject, cx);
        }))
        .child(label)
}

fn window_controls(theme: RelayTheme, window: &Window) -> gpui::Div {
    let max_label = if window.is_maximized() { "❐" } else { "□" };

    div()
        .h_full()
        .flex()
        .items_center()
        .ml_1()
        .window_control_area(WindowControlArea::Drag)
        .child(window_control_button(
            theme,
            WindowControlArea::Min,
            "−",
            false,
        ))
        .child(window_control_button(
            theme,
            WindowControlArea::Max,
            max_label,
            false,
        ))
        .child(window_control_button(
            theme,
            WindowControlArea::Close,
            "×",
            true,
        ))
}

fn window_control_button(
    theme: RelayTheme,
    area: WindowControlArea,
    label: &'static str,
    danger: bool,
) -> gpui::Div {
    div()
        .w(px(44.0))
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .text_lg()
        .text_color(theme.text_muted)
        .window_control_area(area)
        .hover(move |style| {
            if danger {
                style.bg(theme.danger).text_color(gpui::white())
            } else {
                style.bg(theme.hover).text_color(theme.text)
            }
        })
        .child(label)
}

fn status_bar_item(theme: RelayTheme, label: &'static str, value: String) -> gpui::Div {
    div()
        .min_w_0()
        .flex()
        .items_center()
        .gap_1()
        .child(
            div()
                .flex_shrink_0()
                .text_color(theme.text_muted)
                .child(label),
        )
        .child(
            div()
                .min_w_0()
                .truncate()
                .text_color(theme.text_secondary)
                .child(value),
        )
}
