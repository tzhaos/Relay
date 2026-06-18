use gpui::{
    Context, InteractiveElement, IntoElement, StatefulInteractiveElement, div, prelude::*, px,
};
use relay_core::StatusTone;

use crate::{
    app_shell::AppShell,
    theme::RelayTheme,
    workbench::{TaskListItem, TaskListRow, WorkbenchCommand, WorkspaceViewModel},
};

pub fn task_list(
    theme: RelayTheme,
    view_model: &WorkspaceViewModel,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let mut rows = div().flex().flex_col().gap_1();
    for row in view_model.task_list_rows() {
        rows = rows.child(match row {
            TaskListRow::Group { label, count } => {
                group_row(theme, label, count).into_any_element()
            }
            TaskListRow::Task(item) => task_row(theme, *item, cx).into_any_element(),
        });
    }

    div()
        .w(px(312.0))
        .h_full()
        .border_r_1()
        .border_color(theme.line)
        .bg(theme.chrome)
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
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().text_color(theme.text).child("Projects"))
                        .child(div().text_xs().text_color(theme.muted).child("worktrees")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_color(theme.muted)
                        .child("+")
                        .child("..."),
                ),
        )
        .child(
            div()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(theme.line)
                .child(div().text_sm().text_color(theme.muted).child(format!(
                    "{} / {}",
                    view_model.project_label, view_model.branch_label
                ))),
        )
        .child(div().flex_1().p_2().child(rows))
}

fn group_row(theme: RelayTheme, label: String, count: usize) -> gpui::Div {
    div()
        .mt_2()
        .px_2()
        .py_1()
        .flex()
        .items_center()
        .justify_between()
        .text_xs()
        .text_color(theme.muted)
        .child(label)
        .child(count.to_string())
}

fn task_row(theme: RelayTheme, item: TaskListItem, cx: &mut Context<AppShell>) -> impl IntoElement {
    let task_id = item.task.id;
    let status_color = status_color(theme, item.task.status_tone);
    let background = if item.active {
        theme.selection
    } else {
        theme.chrome
    };
    let branch = item
        .task
        .worktree_path
        .as_ref()
        .and_then(|path| path.rsplit(['\\', '/']).next())
        .unwrap_or("no-worktree")
        .to_string();

    div()
        .rounded_md()
        .bg(background)
        .px_3()
        .py_2()
        .flex()
        .flex_col()
        .gap_1()
        .border_1()
        .border_color(if item.active {
            theme.selection_line
        } else {
            background
        })
        .cursor_pointer()
        .id(task_id.as_uuid())
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _, cx| {
            this.dispatch(WorkbenchCommand::ActivateTask(task_id), cx);
        }))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().text_color(theme.accent).child("●"))
                        .child(
                            div()
                                .text_color(theme.text)
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(item.task.title),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(status_color)
                        .font_weight(gpui::FontWeight::BOLD)
                        .child(item.task.status_label),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .text_sm()
                .text_color(theme.muted)
                .child(branch)
                .child(item.agent_label),
        )
}

fn status_color(theme: RelayTheme, tone: StatusTone) -> gpui::Hsla {
    match tone {
        StatusTone::Accent => theme.accent,
        StatusTone::Warning => theme.warning,
        StatusTone::Danger => theme.danger,
        StatusTone::Muted => theme.muted,
        StatusTone::Neutral => theme.text,
    }
}
