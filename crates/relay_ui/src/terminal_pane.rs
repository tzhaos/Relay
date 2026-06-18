use gpui::{
    Context, InteractiveElement, IntoElement, StatefulInteractiveElement, div, prelude::*, px,
};
use relay_core::{TaskProjection, TaskStatus, TerminalSessionId};

use crate::{
    app_shell::AppShell,
    preview_pane::preview_content,
    theme::RelayTheme,
    workbench::{PaneRoute, WorkbenchCommand, WorkspaceViewModel},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalPaneProjection {
    pub session_id: Option<TerminalSessionId>,
    pub cwd: String,
    pub title: Option<String>,
    pub scrollback: String,
    pub exited: bool,
    pub connected: bool,
}

impl TerminalPaneProjection {
    pub fn detached() -> Self {
        Self {
            session_id: None,
            cwd: String::new(),
            title: None,
            scrollback: String::new(),
            exited: false,
            connected: false,
        }
    }
}

pub fn terminal_pane(
    theme: RelayTheme,
    view_model: &WorkspaceViewModel,
    projection: &TerminalPaneProjection,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let status = if projection.exited {
        "EXITED"
    } else if projection.connected {
        "SESSION"
    } else if projection.session_id.is_some() {
        "OFFLINE"
    } else {
        "DETACHED"
    };

    let cwd = if projection.cwd.is_empty() {
        "No worktree attached".to_string()
    } else {
        projection.cwd.clone()
    };
    let launchable_task = view_model.active_task().filter(|task| {
        task.status == TaskStatus::ReadyForAgent
            && task.agent.is_none()
            && projection.connected
            && !projection.exited
    });
    div()
        .flex_1()
        .min_w_0()
        .h_full()
        .bg(theme.bg)
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(42.0))
                .px_3()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(theme.line)
                .bg(theme.chrome)
                .child(
                    div()
                        .min_w_0()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(route_tab(
                            theme,
                            view_model.pane_route,
                            PaneRoute::Terminal,
                            "Terminal",
                            cx,
                        ))
                        .child(route_tab(
                            theme,
                            view_model.pane_route,
                            PaneRoute::Preview,
                            "Preview",
                            cx,
                        )),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .max_w(px(420.0))
                                .truncate()
                                .text_sm()
                                .text_color(theme.muted)
                                .child(cwd.clone()),
                        )
                        .children(
                            launchable_task.map(|task| {
                                launch_agent_button(theme, task, cx).into_any_element()
                            }),
                        )
                        .child(
                            div()
                                .h(px(24.0))
                                .px_2()
                                .rounded_sm()
                                .bg(if projection.exited {
                                    theme.chrome_alt
                                } else {
                                    theme.selection
                                })
                                .text_xs()
                                .text_color(if projection.exited {
                                    theme.muted
                                } else if projection.connected {
                                    theme.accent
                                } else {
                                    theme.warning
                                })
                                .font_weight(gpui::FontWeight::BOLD)
                                .flex()
                                .items_center()
                                .child(status),
                        ),
                ),
        )
        .child(match view_model.pane_route {
            PaneRoute::Terminal => terminal_content(theme, projection),
            PaneRoute::Preview => preview_content(theme, view_model.active_task(), cx),
        })
}

fn terminal_content(theme: RelayTheme, projection: &TerminalPaneProjection) -> gpui::Div {
    let body = if projection.scrollback.is_empty() {
        if projection.connected {
            "Terminal session is connected.".to_string()
        } else if projection.session_id.is_some() {
            "Terminal session is not running.".to_string()
        } else {
            "No terminal session attached.".to_string()
        }
    } else {
        projection.scrollback.clone()
    };

    div()
        .flex_1()
        .bg(theme.terminal_bg)
        .flex()
        .flex_col()
        .min_w_0()
        .child(
            div()
                .flex_1()
                .overflow_hidden()
                .font_family("Consolas")
                .text_color(theme.terminal_text)
                .bg(theme.terminal_bg)
                .p_4()
                .child(body),
        )
}

fn route_tab(
    theme: RelayTheme,
    active_route: PaneRoute,
    route: PaneRoute,
    label: &'static str,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    div()
        .h(px(26.0))
        .px_3()
        .border_1()
        .border_color(if active_route == route {
            theme.selection_line
        } else {
            theme.line
        })
        .bg(if active_route == route {
            theme.panel
        } else {
            theme.chrome
        })
        .text_sm()
        .text_color(if active_route == route {
            theme.text
        } else {
            theme.muted
        })
        .cursor_pointer()
        .hover(|style| style.bg(theme.panel))
        .id(("pane-route", route.index()))
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _, cx| {
            this.dispatch(WorkbenchCommand::SetPaneRoute(route), cx);
        }))
        .child(label)
}

fn launch_agent_button(
    theme: RelayTheme,
    task: &TaskProjection,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let task_id = task.id;
    div()
        .h(px(24.0))
        .px_2()
        .rounded_sm()
        .border_1()
        .border_color(theme.selection_line)
        .bg(theme.panel)
        .text_xs()
        .text_color(theme.text)
        .font_weight(gpui::FontWeight::MEDIUM)
        .flex()
        .items_center()
        .cursor_pointer()
        .hover(|style| style.bg(theme.selection))
        .id(task_id.as_uuid())
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _, cx| {
            this.dispatch(WorkbenchCommand::LaunchAgent(task_id), cx);
        }))
        .child("Launch")
}

impl PaneRoute {
    fn index(self) -> usize {
        match self {
            Self::Terminal => 0,
            Self::Preview => 1,
        }
    }
}
