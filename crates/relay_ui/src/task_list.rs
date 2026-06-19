use gpui::{
    Context, CursorStyle, FocusHandle, InteractiveElement, IntoElement, StatefulInteractiveElement,
    div, prelude::*, px,
};
use relay_core::StatusTone;

use crate::{
    app_shell::AppShell,
    components::{self, ButtonEmphasis, Tone},
    theme::{RelayTheme, mono_family, spacing},
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
        .w(px(spacing::RAIL_WIDTH))
        .h_full()
        .flex_shrink_0()
        .border_r_1()
        .border_color(theme.border)
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
        .h(px(spacing::PANE_HEADER))
        .px_3()
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .border_b_1()
        .border_color(theme.border)
        .child(
            div()
                .min_w_0()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w(px(18.0))
                        .h(px(18.0))
                        .rounded_md()
                        .bg(theme.accent)
                        .flex()
                        .items_center()
                        .justify_center()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_xs()
                        .text_color(theme.terminal_text)
                        .child("R"),
                )
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_sm()
                        .text_color(theme.text)
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(view_model.project_label.clone()),
                ),
        )
        .child(state_badge(theme, view_model.project_open))
}

fn project_section(theme: RelayTheme, view_model: &WorkspaceViewModel) -> gpui::Div {
    let has_worktree = view_model
        .active_task()
        .is_some_and(|task| task.worktree_path.is_some());
    let dot_tone = if has_worktree {
        Tone::Accent
    } else {
        Tone::Muted
    };
    let branch_label = view_model.active_branch_label();
    let worktree_label = view_model.active_worktree_label();
    let worktree_path = view_model.active_worktree_path_label();
    let badge_tone = if has_worktree {
        Tone::Accent
    } else {
        Tone::Muted
    };
    let badge_text = if has_worktree {
        branch_label.as_str()
    } else {
        "—"
    };

    div()
        .px_3()
        .py_3()
        .border_b_1()
        .border_color(theme.border)
        .flex()
        .flex_col()
        .gap_2()
        .child(section_label(theme, "Active Worktree"))
        .child(
            div()
                .rounded_md()
                .bg(theme.panel)
                .px_3()
                .py_2()
                .flex()
                .flex_col()
                .gap_1()
                .border_1()
                .border_color(theme.border)
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
                                .child(components::status_dot(theme, dot_tone))
                                .child(
                                    div()
                                        .min_w_0()
                                        .truncate()
                                        .text_sm()
                                        .text_color(theme.text)
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(if has_worktree {
                                            worktree_label
                                        } else {
                                            "No active worktree".to_string()
                                        }),
                                ),
                        )
                        .child(
                            div()
                                .flex_shrink_0()
                                .child(components::badge(theme, badge_text, badge_tone)),
                        ),
                )
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .font_family(mono_family())
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(if has_worktree {
                            worktree_path
                        } else {
                            "Open a task to create its worktree".to_string()
                        }),
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
                        .child(focus_task_title_button(
                            theme,
                            task_title_focus,
                            view_model.project_open,
                            cx,
                        )),
                ),
        )
        .child(task_title_composer(
            theme,
            view_model.project_open,
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
) -> impl IntoElement {
    let mut rows = div()
        .id("task-rows-scroll")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .overflow_x_hidden()
        .flex()
        .flex_col()
        .gap_1();
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
    } else if view_model.project_open {
        rows.child(components::empty_state(
            theme,
            "No tasks yet",
            "Type a title above and press Enter to create your first task.",
        ))
    } else {
        rows.child(components::empty_state(
            theme,
            "No project open",
            "Click Open Project in the title bar to load a repository.",
        ))
    }
}

fn workspace_metrics(
    theme: RelayTheme,
    summary: &crate::workbench::WorkspaceStatusSummary,
) -> gpui::Div {
    div()
        .px_3()
        .py_2()
        .border_t_1()
        .border_color(theme.border)
        .flex()
        .items_center()
        .gap_2()
        .child(components::metric_pill(
            theme,
            "Working",
            summary.working_count.to_string(),
        ))
        .child(components::metric_pill(
            theme,
            "Waiting",
            summary.waiting_count.to_string(),
        ))
        .child(components::metric_pill(
            theme,
            "Review",
            summary.reviewing_count.to_string(),
        ))
}

fn group_row(theme: RelayTheme, label: String, count: usize) -> gpui::Div {
    div()
        .mt_2()
        .px_1()
        .py_1()
        .flex()
        .items_center()
        .justify_between()
        .text_xs()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(theme.text_muted)
        .child(div().truncate().child(label))
        .child(count.to_string())
}

fn section_label(theme: RelayTheme, label: &'static str) -> gpui::Div {
    div()
        .py_1()
        .text_xs()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(theme.text_muted)
        .child(label)
}

fn state_badge(theme: RelayTheme, project_open: bool) -> gpui::AnyElement {
    components::badge(
        theme,
        if project_open { "OPEN" } else { "DETACHED" },
        if project_open {
            Tone::Accent
        } else {
            Tone::Muted
        },
    )
}

fn count_badge(theme: RelayTheme, value: String) -> gpui::Div {
    div()
        .flex_shrink_0()
        .min_w(px(20.0))
        .h(px(20.0))
        .px_1()
        .rounded_md()
        .bg(theme.panel_alt)
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(theme.text_secondary)
        .child(value)
}

fn focus_task_title_button(
    theme: RelayTheme,
    task_title_focus: &FocusHandle,
    project_open: bool,
    cx: &mut Context<AppShell>,
) -> gpui::AnyElement {
    if !project_open {
        return div()
            .h(px(24.0))
            .px_2()
            .rounded_md()
            .border_1()
            .border_color(theme.border)
            .bg(theme.panel_alt)
            .flex()
            .items_center()
            .text_xs()
            .text_color(theme.text_muted)
            .child("+ Task")
            .into_any_element();
    }
    let focus_handle = task_title_focus.clone();
    div()
        .h(px(24.0))
        .px_2()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.panel)
        .flex()
        .items_center()
        .text_xs()
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(theme.text)
        .cursor_pointer()
        .hover(|style| style.bg(theme.hover).border_color(theme.border_strong))
        .id("focus-task-title")
        .on_click(cx.listener(move |_, _: &gpui::ClickEvent, window, _| {
            window.focus(&focus_handle);
        }))
        .child("+ Task")
        .into_any_element()
}

fn task_title_composer(
    theme: RelayTheme,
    project_open: bool,
    draft: &str,
    task_title_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> gpui::Div {
    let focus_handle = task_title_focus.clone();
    let can_create = project_open && !draft.trim().is_empty();
    let label = if !project_open {
        "No project open".to_string()
    } else if draft.is_empty() {
        "Task title…".to_string()
    } else {
        draft.to_string()
    };
    let input = div()
        .h(px(30.0))
        .min_w_0()
        .flex_1()
        .rounded_md()
        .border_1()
        .border_color(if can_create {
            theme.border_strong
        } else {
            theme.border
        })
        .bg(theme.panel)
        .px_2()
        .flex()
        .items_center()
        .text_sm()
        .text_color(if !project_open || draft.is_empty() {
            theme.text_muted
        } else {
            theme.text
        })
        .id("task-title-input");
    let input = if project_open {
        input
            .track_focus(task_title_focus)
            .tab_index(0)
            .cursor(CursorStyle::IBeam)
            .key_context("TaskTitleDraft")
            .focus(|style| style.border_color(theme.accent_border))
            .on_key_down(cx.listener(|this, event, _, cx| {
                if this.handle_task_title_key(event, cx) {
                    cx.stop_propagation();
                }
            }))
            .on_click(cx.listener(move |_, _: &gpui::ClickEvent, window, _| {
                window.focus(&focus_handle);
            }))
    } else {
        input
    };

    div()
        .rounded_md()
        .border_1()
        .border_color(if can_create {
            theme.border_strong
        } else {
            theme.border
        })
        .bg(theme.panel_alt)
        .px_2()
        .py_2()
        .flex()
        .items_center()
        .gap_2()
        .child(input.child(div().min_w_0().truncate().child(label)))
        .child(if can_create {
            components::button(
                theme,
                "Create",
                ButtonEmphasis::Primary,
                "create-task",
                cx,
                |this, cx| this.dispatch(WorkbenchCommand::CreateTask, cx),
            )
        } else {
            components::badge(
                theme,
                if project_open { "TITLE" } else { "PROJECT" },
                Tone::Muted,
            )
        })
}

fn task_row(
    theme: RelayTheme,
    item: TaskListItem,
    terminal_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let task_id = item.task.id;
    let status_tone = tone_from_status(item.task.status_tone);
    let focus_handle = terminal_focus.clone();
    // Active and inactive rows share the same height (DESIGN.md: task rows must
    // not change height when active/inactive). Active uses an accent-tinted fill;
    // inactive is transparent until hover.
    let background = if item.active {
        theme.accent_bg
    } else {
        theme.chrome
    };
    let border_color = if item.active {
        theme.accent_border
    } else {
        theme.border
    };
    let hover_background = if item.active {
        theme.accent_bg
    } else {
        theme.hover
    };
    let hover_border = if item.active {
        theme.accent_border
    } else {
        theme.border_strong
    };

    div()
        .rounded_md()
        .bg(background)
        .h(px(spacing::TASK_ROW))
        .px_2()
        .py_1()
        .flex()
        .flex_col()
        .justify_center()
        .gap_1()
        .overflow_hidden()
        .border_1()
        .border_color(border_color)
        .cursor_pointer()
        .hover(move |style| style.bg(hover_background).border_color(hover_border))
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
                        .child(components::status_dot(theme, status_tone))
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .text_sm()
                                .text_color(theme.text)
                                .font_weight(if item.active {
                                    gpui::FontWeight::SEMIBOLD
                                } else {
                                    gpui::FontWeight::MEDIUM
                                })
                                .child(item.task.title),
                        ),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(components::flat_label(
                            theme,
                            &item.task.status_label,
                            status_tone,
                        ))
                        .children(
                            (item.active && item.can_archive).then(|| {
                                archive_task_button(theme, task_id, cx).into_any_element()
                            }),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .text_xs()
                .text_color(theme.text_muted)
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .font_family(mono_family())
                        .child(item.worktree_label),
                )
                .child(div().flex_shrink_0().truncate().child(item.agent_label)),
        )
}

fn archive_task_button(
    theme: RelayTheme,
    task_id: relay_core::TaskId,
    cx: &mut Context<AppShell>,
) -> gpui::AnyElement {
    // The shared button stops click propagation, so this won't also activate the
    // enclosing task row.
    components::button(
        theme,
        "Archive",
        ButtonEmphasis::Ghost,
        (gpui::ElementId::from(task_id.as_uuid()), "archive-task"),
        cx,
        move |this, cx| {
            this.dispatch(WorkbenchCommand::ArchiveTask(task_id), cx);
        },
    )
}

/// Map a domain [`StatusTone`] to a UI [`Tone`].
fn tone_from_status(tone: StatusTone) -> Tone {
    match tone {
        StatusTone::Accent => Tone::Accent,
        StatusTone::Warning => Tone::Warning,
        StatusTone::Danger => Tone::Danger,
        StatusTone::Muted => Tone::Muted,
        StatusTone::Neutral => Tone::Secondary,
    }
}
