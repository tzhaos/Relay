use gpui::{
    Context, InteractiveElement, IntoElement, StatefulInteractiveElement, div, prelude::*, px,
};
use relay_core::TerminalSessionId;

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
}

impl TerminalPaneProjection {
    pub fn detached() -> Self {
        Self {
            session_id: None,
            cwd: String::new(),
            title: None,
            scrollback: String::new(),
            exited: false,
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
    } else if projection.session_id.is_some() {
        "ATTACHED"
    } else {
        "DETACHED"
    };

    let title = projection
        .title
        .clone()
        .unwrap_or_else(|| "Relay terminal".to_string());
    let cwd = if projection.cwd.is_empty() {
        "No worktree attached".to_string()
    } else {
        projection.cwd.clone()
    };
    let scrollback = if projection.scrollback.is_empty() {
        "relay $ waiting for terminal runtime...".to_string()
    } else {
        projection.scrollback.clone()
    };

    div()
        .flex_1()
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
                        .text_xs()
                        .text_color(if projection.exited {
                            theme.muted
                        } else {
                            theme.accent
                        })
                        .font_weight(gpui::FontWeight::BOLD)
                        .child(status),
                ),
        )
        .child(match view_model.pane_route {
            PaneRoute::Terminal => terminal_content(theme, title, cwd, scrollback),
            PaneRoute::Preview => preview_content(theme, view_model.active_task()),
        })
}

fn terminal_content(
    theme: RelayTheme,
    title: String,
    cwd: String,
    scrollback: String,
) -> gpui::Div {
    div()
        .flex_1()
        .bg(theme.bg)
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(40.0))
                .px_4()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(theme.line)
                .child(div().text_color(theme.text).child(title))
                .child(div().text_sm().text_color(theme.muted).child(cwd)),
        )
        .child(
            div()
                .flex_1()
                .font_family("Consolas")
                .text_color(theme.terminal_text)
                .bg(theme.terminal_bg)
                .p_4()
                .child(scrollback),
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
        .px_3()
        .py_1()
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
        .id(("pane-route", route.index()))
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _, cx| {
            this.dispatch(WorkbenchCommand::SetPaneRoute(route), cx);
        }))
        .child(label)
}

impl PaneRoute {
    fn index(self) -> usize {
        match self {
            Self::Terminal => 0,
            Self::Preview => 1,
        }
    }
}
