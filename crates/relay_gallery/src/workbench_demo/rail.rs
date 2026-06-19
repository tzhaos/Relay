use gpui::{Entity, FontWeight, IntoElement, ParentElement, Styled, div, px};
use relay_ui_kit::{
    Divider, IconButton, IconName, NavRow, Pane, PaneSurface, PaneWidth, PanelHeader, TaskRow,
    TaskRowData, TerminalStatusBadge, Theme, TreeRow,
    theme::{radius, space},
};

use super::{
    WorkbenchState,
    data::{DEMO_TASKS, active_session, session_index_for_key},
};
use crate::GalleryApp;

pub(super) fn left_rail(
    state: &WorkbenchState,
    host: &Entity<GalleryApp>,
    theme: Theme,
) -> impl IntoElement {
    let body = div()
        .size_full()
        .flex()
        .flex_col()
        .child(nav_section(host, state, theme))
        .child(Divider::horizontal())
        .child(project_tree(theme))
        .child(Divider::horizontal())
        .child(tasks_header(host, theme))
        .child(task_rows(state, host));

    Pane::new(PaneWidth::Flex, body)
        .surface(PaneSurface::Chrome)
        .header(PanelHeader::new("Workspace").icon(IconName::Folder))
}

fn nav_section(
    host: &Entity<GalleryApp>,
    state: &WorkbenchState,
    theme: Theme,
) -> impl IntoElement {
    div()
        .px_2()
        .py_2()
        .flex()
        .flex_col()
        .gap(px(1.0))
        .child(
            NavRow::new("nav-tasks", IconName::ListChecks, "Tasks")
                .selected(true)
                .count(DEMO_TASKS.len()),
        )
        .child(
            NavRow::new("nav-terminals", IconName::Terminal, "Terminals").on_click({
                let host = host.clone();
                move |_event, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.workbench.route = "terminal";
                        cx.notify();
                    });
                }
            }),
        )
        .child(
            NavRow::new("nav-search", IconName::Search, "Search").on_click({
                let host = host.clone();
                move |_event, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.workbench.context_tab = "files";
                        cx.notify();
                    });
                }
            }),
        )
        .child(session_summary(state, theme))
}

fn session_summary(state: &WorkbenchState, theme: Theme) -> impl IntoElement {
    let session = active_session(state);

    div()
        .mt_1()
        .px_2()
        .py_1()
        .rounded(px(radius::MD))
        .bg(gpui::transparent_black())
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .child(
            div()
                .min_w_0()
                .truncate()
                .text_size(px(11.0))
                .text_color(theme.text_muted)
                .child(format!("Active: {}", session.label)),
        )
        .child(TerminalStatusBadge::new(session.tone))
}

fn project_tree(theme: Theme) -> impl IntoElement {
    div()
        .px_2()
        .py_2()
        .flex()
        .flex_col()
        .gap(px(1.0))
        .child(section_label("PROJECT", theme))
        .child(TreeRow::new("proj-relay", IconName::Folder, "Relay").expandable(true))
        .child(
            TreeRow::new("wt-main", IconName::GitBranch, "main")
                .depth(1)
                .selected(true),
        )
        .child(TreeRow::new("wt-shell", IconName::GitBranch, "ui/workbench-shell").depth(1))
        .child(TreeRow::new("wt-agent", IconName::GitBranch, "fix/agent-retry").depth(1))
}

fn tasks_header(host: &Entity<GalleryApp>, theme: Theme) -> impl IntoElement {
    div()
        .px_3()
        .h(px(space::PANE_HEADER))
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text_muted)
                .child("TASKS"),
        )
        .child(IconButton::new("add-task", IconName::Plus).on_click({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.workbench.active_task = 0;
                    this.workbench.active_session =
                        session_index_for_key(DEMO_TASKS[0].session_key);
                    cx.notify();
                });
            }
        }))
}

fn task_rows(state: &WorkbenchState, host: &Entity<GalleryApp>) -> impl IntoElement {
    div()
        .flex_1()
        .min_h_0()
        .px_2()
        .pb_2()
        .flex()
        .flex_col()
        .gap_1()
        .children(DEMO_TASKS.iter().enumerate().map(|(index, task)| {
            let host = host.clone();
            TaskRow::new(
                ("task", index),
                TaskRowData {
                    title: task.title.into(),
                    status_label: task.status.into(),
                    status_tone: task.tone,
                    branch: Some(task.branch.into()),
                    changed: task.changed,
                    review: task.review,
                },
            )
            .selected(index == state.active_task)
            .on_click(move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.workbench.active_task = index;
                    this.workbench.active_session = session_index_for_key(task.session_key);
                    this.workbench.route = "terminal";
                    cx.notify();
                });
            })
            .into_any_element()
        }))
}

fn section_label(label: &'static str, theme: Theme) -> impl IntoElement {
    div()
        .text_size(px(11.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme.text_muted)
        .child(label)
}
