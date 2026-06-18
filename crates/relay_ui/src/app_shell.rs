use std::time::Duration;

use gpui::{
    App, Bounds, Context, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, Render,
    Task as GpuiTask, TitlebarOptions, Window, WindowBounds, WindowControlArea, WindowDecorations,
    WindowOptions, div, prelude::*, px, size,
};
use relay_core::{TaskId, TaskProjection, TerminalSessionId};

use crate::{
    diff_pane::context_pane,
    task_list::task_list,
    terminal_pane::{TerminalPaneProjection, terminal_pane},
    theme::RelayTheme,
    workbench::{WorkbenchCommand, WorkspaceViewModel},
};

pub struct AppShell {
    theme: RelayTheme,
    view_model: WorkspaceViewModel,
    task_data_source: Box<dyn TaskDataSource>,
    context_filter_focus: FocusHandle,
    last_error: Option<String>,
    _runtime_poll_task: GpuiTask<()>,
}

pub trait TaskDataSource {
    fn create_task(&mut self, title: &str) -> anyhow::Result<Vec<TaskProjection>>;
    fn launch_agent(&mut self, task_id: TaskId) -> anyhow::Result<Vec<TaskProjection>>;
    fn deliver_review(&mut self, task_id: TaskId) -> anyhow::Result<Vec<TaskProjection>>;
    fn attach_worktree_preview(&mut self, task_id: TaskId) -> anyhow::Result<Vec<TaskProjection>>;
    fn poll_runtime(&mut self) -> anyhow::Result<bool>;
    fn terminal_projection(
        &mut self,
        session_id: TerminalSessionId,
    ) -> anyhow::Result<Option<TerminalPaneProjection>>;
}

impl AppShell {
    pub fn open(
        cx: &mut App,
        project_label: String,
        tasks: Vec<TaskProjection>,
        task_data_source: Box<dyn TaskDataSource>,
    ) -> anyhow::Result<()> {
        let bounds = Bounds::centered(None, size(px(1440.0), px(900.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Relay".into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                window_decorations: Some(WindowDecorations::Client),
                window_min_size: Some(size(px(1180.0), px(780.0))),
                app_id: Some("relay".to_string()),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Self::new(project_label, tasks, task_data_source, cx)),
        )?;
        cx.activate(true);
        Ok(())
    }

    fn new(
        project_label: String,
        tasks: Vec<TaskProjection>,
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
            view_model: WorkspaceViewModel::for_project(project_label, tasks),
            task_data_source,
            context_filter_focus: cx.focus_handle(),
            last_error: None,
            _runtime_poll_task: runtime_poll_task,
        }
    }

    fn title_bar(&self, window: &mut Window) -> impl IntoElement {
        let summary = self.view_model.status_summary();

        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(42.0))
            .pl_3()
            .border_b_1()
            .border_color(self.theme.line)
            .bg(self.theme.chrome)
            .window_control_area(WindowControlArea::Drag)
            .child(
                div()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(title_mark(self.theme))
                    .child(
                        div()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .truncate()
                                    .text_sm()
                                    .text_color(self.theme.text)
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child("Relay"),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_xs()
                                    .text_color(self.theme.muted)
                                    .child(self.view_model.active_worktree_label()),
                            ),
                    ),
            )
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .child(title_badge(
                        self.theme,
                        self.view_model.project_label.clone(),
                    ))
                    .child(title_badge(
                        self.theme,
                        self.view_model.active_branch_label(),
                    )),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .h_full()
                    .flex()
                    .items_center()
                    .child(header_stat(
                        self.theme,
                        "Tasks",
                        summary.task_count.to_string(),
                    ))
                    .child(header_stat(
                        self.theme,
                        "Agents",
                        summary.active_agent_count.to_string(),
                    ))
                    .child(window_controls(self.theme, window)),
            )
    }

    fn terminal_projection(&mut self) -> TerminalPaneProjection {
        let Some(active_task) = self.view_model.active_task() else {
            return TerminalPaneProjection::detached();
        };

        let mut projection = TerminalPaneProjection {
            session_id: active_task.terminal_session_id,
            cwd: active_task.worktree_path.clone().unwrap_or_default(),
            title: active_task
                .agent
                .as_ref()
                .map(|kind| format!("{} session", kind.label())),
            scrollback: String::new(),
            exited: false,
            connected: false,
        };

        if let Some(session_id) = active_task.terminal_session_id {
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
        }

        projection
    }

    pub(crate) fn dispatch(&mut self, command: WorkbenchCommand, cx: &mut Context<Self>) {
        if command == WorkbenchCommand::CreateTask {
            self.create_task(cx);
            return;
        }
        if let WorkbenchCommand::LaunchAgent(task_id) = command {
            self.launch_agent(task_id, cx);
            return;
        }
        if let WorkbenchCommand::DeliverReview(task_id) = command {
            self.deliver_review(task_id, cx);
            return;
        }
        if let WorkbenchCommand::AttachWorktreePreview(task_id) = command {
            self.attach_worktree_preview(task_id, cx);
            return;
        }

        self.view_model.apply_command(command);
        cx.notify();
    }

    pub(crate) fn context_filter_focus(&self) -> &FocusHandle {
        &self.context_filter_focus
    }

    pub(crate) fn handle_context_filter_key(
        &mut self,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        let keystroke = event.keystroke.clone().with_simulated_ime();
        match keystroke.key.as_str() {
            "escape" => {
                self.dispatch(WorkbenchCommand::ClearContextFilter, cx);
                true
            }
            "backspace" => {
                self.dispatch(WorkbenchCommand::BackspaceContextFilter, cx);
                true
            }
            _ if !keystroke.modifiers.control
                && !keystroke.modifiers.alt
                && !keystroke.modifiers.platform
                && !keystroke.modifiers.function =>
            {
                if let Some(text) = keystroke
                    .key_char
                    .filter(|text| text.chars().all(|character| !character.is_control()))
                {
                    self.dispatch(WorkbenchCommand::AppendContextFilter(text), cx);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn create_task(&mut self, cx: &mut Context<Self>) {
        let title = format!("New task {}", self.view_model.tasks.len() + 1);
        match self.task_data_source.create_task(&title) {
            Ok(tasks) => {
                let project_label = self.view_model.project_label.clone();
                self.view_model = WorkspaceViewModel::for_project(project_label, tasks);
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
                    let project_label = self.view_model.project_label.clone();
                    let active_task_id = self.view_model.active_task_id;
                    self.view_model = WorkspaceViewModel::for_project(project_label, tasks);
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

    fn deliver_review(&mut self, task_id: TaskId, cx: &mut Context<Self>) {
        match self.task_data_source.deliver_review(task_id) {
            Ok(tasks) => {
                if !tasks.is_empty() {
                    let project_label = self.view_model.project_label.clone();
                    let active_task_id = self.view_model.active_task_id;
                    self.view_model = WorkspaceViewModel::for_project(project_label, tasks);
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

    fn attach_worktree_preview(&mut self, task_id: TaskId, cx: &mut Context<Self>) {
        match self.task_data_source.attach_worktree_preview(task_id) {
            Ok(tasks) => {
                if !tasks.is_empty() {
                    let project_label = self.view_model.project_label.clone();
                    let active_task_id = self.view_model.active_task_id;
                    self.view_model = WorkspaceViewModel::for_project(project_label, tasks);
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
            .h(px(28.0))
            .flex_shrink_0()
            .px_3()
            .border_t_1()
            .border_color(self.theme.line)
            .bg(self.theme.chrome_alt)
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
            .bg(self.theme.bg)
            .flex()
            .flex_col()
            .child(self.title_bar(window))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .child(task_list(self.theme, &self.view_model, cx))
                    .child(terminal_pane(
                        self.theme,
                        &self.view_model,
                        &terminal_projection,
                        cx,
                    ))
                    .child(context_pane(
                        self.theme,
                        &self.view_model,
                        self.context_filter_focus(),
                        cx,
                    )),
            )
            .child(self.status_bar())
    }
}

fn title_mark(theme: RelayTheme) -> gpui::Div {
    div()
        .w(px(24.0))
        .h(px(24.0))
        .rounded_sm()
        .border_1()
        .border_color(theme.line)
        .bg(theme.panel)
        .flex()
        .items_center()
        .justify_center()
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(theme.text)
        .child("R")
}

fn title_badge(theme: RelayTheme, label: String) -> gpui::Div {
    div()
        .h(px(26.0))
        .max_w(px(220.0))
        .px_3()
        .rounded_md()
        .border_1()
        .border_color(theme.line)
        .bg(theme.panel)
        .flex()
        .items_center()
        .text_sm()
        .text_color(theme.text)
        .child(div().truncate().child(label))
}

fn header_stat(theme: RelayTheme, label: &'static str, value: String) -> gpui::Div {
    div()
        .h(px(26.0))
        .mr_2()
        .px_2()
        .rounded_sm()
        .border_1()
        .border_color(theme.line)
        .bg(theme.chrome_alt)
        .flex()
        .items_center()
        .gap_1()
        .text_sm()
        .child(div().text_color(theme.muted).child(label))
        .child(
            div()
                .max_w(px(80.0))
                .truncate()
                .text_color(theme.text)
                .child(value),
        )
}

fn window_controls(theme: RelayTheme, window: &Window) -> gpui::Div {
    let max_label = if window.is_maximized() { "[]" } else { "[ ]" };

    div()
        .h_full()
        .flex()
        .items_center()
        .window_control_area(WindowControlArea::Drag)
        .child(window_control_button(
            theme,
            WindowControlArea::Min,
            "_",
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
            "X",
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
        .text_sm()
        .text_color(theme.muted)
        .window_control_area(area)
        .hover(move |style| {
            if danger {
                style.bg(theme.danger).text_color(gpui::white())
            } else {
                style.bg(theme.selection).text_color(theme.text)
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
        .child(div().flex_shrink_0().text_color(theme.muted).child(label))
        .child(
            div()
                .min_w_0()
                .truncate()
                .text_color(theme.text)
                .child(value),
        )
}
