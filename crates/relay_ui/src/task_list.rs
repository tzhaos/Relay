use gpui::{
    Context, CursorStyle, FocusHandle, InteractiveElement, IntoElement, StatefulInteractiveElement,
    div, prelude::*, px,
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
    task_title_focus: &FocusHandle,
    terminal_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let summary = view_model.status_summary();

    div()
        .w(px(320.0))
        .h_full()
        .border_r_1()
        .border_color(theme.line)
        .bg(theme.chrome)
        .flex()
        .flex_col()
        .child(left_header(theme, view_model))
        .child(project_section(theme, view_model))
        .child(task_section(
            theme,
            view_model,
            task_title_focus,
            terminal_focus,
            cx,
        ))
        .child(workspace_metrics(theme, &summary))
}

fn left_header(theme: RelayTheme, view_model: &WorkspaceViewModel) -> gpui::Div {
    div()
        .h(px(48.0))
        .px_3()
        .flex()
        .items_center()
        .gap_2()
        .border_b_1()
        .border_color(theme.line)
        .child(brand_mark(theme))
        .child(
            div()
                .min_w_0()
                .flex()
                .flex_col()
                .child(
                    div()
                        .text_color(theme.text)
                        .font_weight(gpui::FontWeight::BOLD)
                        .child("Relay"),
                )
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_xs()
                        .text_color(theme.muted)
                        .child(view_model.project_label.clone()),
                ),
        )
}

fn project_section(theme: RelayTheme, view_model: &WorkspaceViewModel) -> gpui::Div {
    let has_worktree = view_model
        .active_task()
        .is_some_and(|task| task.worktree_path.is_some());

    div()
        .px_3()
        .py_3()
        .border_b_1()
        .border_color(theme.line)
        .flex()
        .flex_col()
        .gap_2()
        .child(section_label(theme, "Workspace"))
        .child(project_group(
            theme,
            view_model.project_label.clone(),
            view_model.tasks.len(),
        ))
        .child(
            div()
                .rounded_md()
                .bg(theme.selection)
                .px_3()
                .py_3()
                .flex()
                .flex_col()
                .gap_1()
                .border_1()
                .border_color(theme.selection_line)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .min_w_0()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(status_dot(
                                    theme,
                                    if has_worktree {
                                        theme.accent
                                    } else {
                                        theme.muted
                                    },
                                ))
                                .child(
                                    div()
                                        .min_w_0()
                                        .truncate()
                                        .text_color(theme.text)
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(view_model.active_worktree_label()),
                                ),
                        )
                        .child(badge(theme, view_model.active_branch_label())),
                )
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .font_family("Consolas")
                        .text_xs()
                        .text_color(theme.muted)
                        .child(view_model.active_worktree_path_label()),
                ),
        )
}

fn task_section(
    theme: RelayTheme,
    view_model: &WorkspaceViewModel,
    task_title_focus: &FocusHandle,
    terminal_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> gpui::Div {
    div()
        .flex_1()
        .min_h_0()
        .px_3()
        .py_3()
        .overflow_hidden()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(section_label(theme, "Tasks"))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(count_badge(theme, view_model.tasks.len().to_string()))
                        .child(focus_task_title_button(theme, task_title_focus, cx)),
                ),
        )
        .child(task_title_composer(
            theme,
            view_model.task_title_draft.as_str(),
            task_title_focus,
            cx,
        ))
        .child(task_rows(theme, view_model, terminal_focus, cx))
}

fn task_rows(
    theme: RelayTheme,
    view_model: &WorkspaceViewModel,
    terminal_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> gpui::Div {
    let mut rows = div().flex().flex_col().gap_1();
    let mut has_rows = false;

    for row in view_model.task_list_rows() {
        has_rows = true;
        rows = rows.child(match row {
            TaskListRow::Group { label, count } => {
                group_row(theme, label, count).into_any_element()
            }
            TaskListRow::Task(item) => {
                task_row(theme, *item, terminal_focus, cx).into_any_element()
            }
        });
    }

    if has_rows {
        rows
    } else {
        rows.child(empty_state(theme, "No tasks", "Task list is empty."))
    }
}

fn workspace_metrics(
    theme: RelayTheme,
    summary: &crate::workbench::WorkspaceStatusSummary,
) -> gpui::Div {
    div()
        .px_3()
        .py_3()
        .border_t_1()
        .border_color(theme.line)
        .flex()
        .items_center()
        .justify_between()
        .text_xs()
        .text_color(theme.muted)
        .child(metric_pill(
            theme,
            "Active",
            summary.active_count.to_string(),
        ))
        .child(metric_pill(
            theme,
            "Attention",
            summary.attention_count.to_string(),
        ))
        .child(metric_pill(
            theme,
            "Review",
            summary.review_count.to_string(),
        ))
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

fn section_label(theme: RelayTheme, label: &'static str) -> gpui::Div {
    div().py_1().text_xs().text_color(theme.muted).child(label)
}

fn brand_mark(theme: RelayTheme) -> gpui::Div {
    div()
        .w(px(28.0))
        .h(px(28.0))
        .rounded_md()
        .bg(theme.panel)
        .border_1()
        .border_color(theme.line)
        .flex()
        .items_center()
        .justify_center()
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(theme.text)
        .child("R")
}

fn project_group(theme: RelayTheme, label: String, count: usize) -> gpui::Div {
    div()
        .h(px(30.0))
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .min_w_0()
                .flex()
                .items_center()
                .gap_2()
                .text_color(theme.text)
                .child(project_glyph(theme))
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .font_weight(gpui::FontWeight::BOLD)
                        .child(label),
                ),
        )
        .child(count_badge(theme, count.to_string()))
}

fn project_glyph(theme: RelayTheme) -> gpui::Div {
    div()
        .w(px(18.0))
        .h(px(14.0))
        .rounded_sm()
        .border_1()
        .border_color(theme.selection_line)
        .bg(theme.panel)
}

fn badge(theme: RelayTheme, label: String) -> gpui::Div {
    div()
        .flex_shrink_0()
        .h(px(22.0))
        .max_w(px(112.0))
        .px_2()
        .rounded_sm()
        .bg(theme.panel)
        .flex()
        .items_center()
        .text_xs()
        .text_color(theme.muted)
        .child(div().truncate().child(label))
}

fn count_badge(theme: RelayTheme, value: String) -> gpui::Div {
    div()
        .flex_shrink_0()
        .min_w(px(20.0))
        .h(px(20.0))
        .px_2()
        .rounded_md()
        .bg(theme.chrome_alt)
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .text_color(theme.muted)
        .child(value)
}

fn focus_task_title_button(
    theme: RelayTheme,
    task_title_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let focus_handle = task_title_focus.clone();
    div()
        .h(px(24.0))
        .px_2()
        .rounded_sm()
        .border_1()
        .border_color(theme.line)
        .bg(theme.panel)
        .flex()
        .items_center()
        .text_xs()
        .text_color(theme.text)
        .cursor_pointer()
        .hover(|style| style.bg(theme.selection))
        .id("focus-task-title")
        .on_click(cx.listener(move |_, _: &gpui::ClickEvent, window, _| {
            window.focus(&focus_handle);
        }))
        .child("New")
}

fn task_title_composer(
    theme: RelayTheme,
    draft: &str,
    task_title_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> gpui::Div {
    let focus_handle = task_title_focus.clone();
    let can_create = !draft.trim().is_empty();
    let label = if draft.is_empty() {
        "Task title".to_string()
    } else {
        draft.to_string()
    };

    div()
        .rounded_md()
        .border_1()
        .border_color(if can_create {
            theme.selection_line
        } else {
            theme.line
        })
        .bg(theme.panel)
        .px_2()
        .py_2()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .h(px(30.0))
                .min_w_0()
                .flex_1()
                .rounded_sm()
                .border_1()
                .border_color(if can_create {
                    theme.selection_line
                } else {
                    theme.line
                })
                .bg(theme.chrome)
                .px_2()
                .flex()
                .items_center()
                .text_sm()
                .text_color(if draft.is_empty() {
                    theme.muted
                } else {
                    theme.text
                })
                .track_focus(task_title_focus)
                .tab_index(0)
                .cursor(CursorStyle::IBeam)
                .key_context("TaskTitleDraft")
                .focus(|style| style.border_color(theme.selection_line))
                .on_key_down(cx.listener(|this, event, _, cx| {
                    if this.handle_task_title_key(event, cx) {
                        cx.stop_propagation();
                    }
                }))
                .id("task-title-input")
                .on_click(cx.listener(move |_, _: &gpui::ClickEvent, window, _| {
                    window.focus(&focus_handle);
                }))
                .child(div().min_w_0().truncate().child(label)),
        )
        .child(if can_create {
            create_task_button(theme, cx).into_any_element()
        } else {
            task_title_state_badge(theme).into_any_element()
        })
}

fn create_task_button(theme: RelayTheme, cx: &mut Context<AppShell>) -> impl IntoElement {
    div()
        .h(px(28.0))
        .px_2()
        .rounded_sm()
        .border_1()
        .border_color(theme.selection_line)
        .bg(theme.panel)
        .flex()
        .items_center()
        .text_xs()
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(theme.text)
        .cursor_pointer()
        .hover(|style| style.bg(theme.selection))
        .id("create-task")
        .on_click(cx.listener(|this, _: &gpui::ClickEvent, _, cx| {
            this.dispatch(WorkbenchCommand::CreateTask, cx);
        }))
        .child("Create")
}

fn task_title_state_badge(theme: RelayTheme) -> gpui::Div {
    div()
        .h(px(28.0))
        .px_2()
        .rounded_sm()
        .border_1()
        .border_color(theme.line)
        .bg(theme.chrome_alt)
        .flex()
        .items_center()
        .text_xs()
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(theme.muted)
        .child("TITLE")
}

fn metric_pill(theme: RelayTheme, label: &'static str, value: String) -> gpui::Div {
    div()
        .min_w(px(80.0))
        .h(px(28.0))
        .rounded_md()
        .bg(theme.chrome_alt)
        .border_1()
        .border_color(theme.line)
        .px_2()
        .flex()
        .items_center()
        .justify_between()
        .child(label)
        .child(
            div()
                .text_color(theme.text)
                .font_weight(gpui::FontWeight::BOLD)
                .child(value),
        )
}

fn status_dot(theme: RelayTheme, color: gpui::Hsla) -> gpui::Div {
    div()
        .w(px(8.0))
        .h(px(8.0))
        .rounded_md()
        .bg(color)
        .border_1()
        .border_color(theme.panel)
}

fn empty_state(theme: RelayTheme, title: &'static str, detail: &'static str) -> gpui::Div {
    div()
        .rounded_md()
        .border_1()
        .border_color(theme.line)
        .bg(theme.chrome_alt)
        .p_3()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_sm()
                .text_color(theme.text)
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(title),
        )
        .child(div().text_xs().text_color(theme.muted).child(detail))
}

fn task_row(
    theme: RelayTheme,
    item: TaskListItem,
    terminal_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let task_id = item.task.id;
    let status_color = status_color(theme, item.task.status_tone);
    let focus_handle = terminal_focus.clone();
    let background = if item.active {
        theme.selection
    } else {
        theme.chrome
    };
    let hover_background = if item.active {
        theme.selection
    } else {
        theme.panel
    };

    div()
        .rounded_md()
        .bg(background)
        .h(px(66.0))
        .px_3()
        .py_2()
        .flex()
        .flex_col()
        .gap_1()
        .overflow_hidden()
        .border_1()
        .border_color(if item.active {
            theme.selection_line
        } else {
            background
        })
        .cursor_pointer()
        .hover(|style| {
            style
                .bg(hover_background)
                .border_color(theme.selection_line)
        })
        .id(task_id.as_uuid())
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, window, cx| {
            this.dispatch(WorkbenchCommand::ActivateTask(task_id), cx);
            window.focus(&focus_handle);
        }))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .min_w_0()
                        .flex_1()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(status_dot(theme, status_color))
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .text_color(theme.text)
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(item.task.title),
                        ),
                )
                .child(
                    div()
                        .flex_shrink_0()
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
                .gap_2()
                .text_sm()
                .text_color(theme.muted)
                .child(div().min_w_0().truncate().child(item.worktree_label))
                .child(div().flex_shrink_0().truncate().child(item.agent_label)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .text_xs()
                .text_color(theme.muted)
                .child(item.changed_label)
                .child(item.review_label),
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
