use gpui::{
    Context, CursorStyle, FocusHandle, InteractiveElement, IntoElement, StatefulInteractiveElement,
    div, prelude::*, px,
};
use relay_core::{ChangeStatus, ChangedFile, ReviewCommentProjection, TaskProjection};
use relay_diff::{DiffTree, DiffTreeRow, DiffTreeRowKind};

use crate::{
    app_shell::AppShell,
    theme::RelayTheme,
    workbench::{ContextTab, WorkbenchCommand, WorkspaceViewModel},
};

pub fn context_pane(
    theme: RelayTheme,
    view_model: &WorkspaceViewModel,
    filter_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let active_task = view_model.active_task();
    let filter = view_model.context_filter.as_str();

    div()
        .w(px(380.0))
        .h_full()
        .border_l_1()
        .border_color(theme.line)
        .bg(theme.chrome)
        .flex()
        .flex_col()
        .child(header(
            theme,
            view_model.context_tab,
            filter,
            filter_focus,
            cx,
        ))
        .child(match view_model.context_tab {
            ContextTab::Files => files_tab(theme, active_task, filter),
            ContextTab::Diff => diff_tab(theme, active_task, filter),
            ContextTab::Review => review_tab(theme, active_task, filter),
        })
}

fn header(
    theme: RelayTheme,
    active_tab: ContextTab,
    filter: &str,
    filter_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> gpui::Div {
    div()
        .px_3()
        .py_2()
        .flex()
        .flex_col()
        .gap_2()
        .border_b_1()
        .border_color(theme.line)
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_color(theme.text)
                        .font_weight(gpui::FontWeight::BOLD)
                        .child("Context"),
                )
                .child(
                    div()
                        .rounded_sm()
                        .bg(theme.chrome_alt)
                        .px_2()
                        .py_1()
                        .text_xs()
                        .text_color(theme.muted)
                        .child(active_tab.label()),
                ),
        )
        .child(search_field(theme, filter, filter_focus, cx))
        .child(
            div()
                .h(px(32.0))
                .rounded_md()
                .bg(theme.chrome_alt)
                .p_0p5()
                .flex()
                .items_center()
                .child(tab(theme, active_tab, ContextTab::Files, "Files", cx))
                .child(tab(theme, active_tab, ContextTab::Diff, "Diff", cx))
                .child(tab(theme, active_tab, ContextTab::Review, "Review", cx)),
        )
}

fn search_field(
    theme: RelayTheme,
    filter: &str,
    filter_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let focus_handle = filter_focus.clone();
    let label = if filter.is_empty() {
        "Filter files".to_string()
    } else {
        filter.to_string()
    };

    div()
        .h(px(32.0))
        .rounded_md()
        .border_1()
        .border_color(if filter.is_empty() {
            theme.line
        } else {
            theme.selection_line
        })
        .bg(theme.panel)
        .px_3()
        .flex()
        .items_center()
        .gap_2()
        .text_sm()
        .text_color(if filter.is_empty() {
            theme.muted
        } else {
            theme.text
        })
        .track_focus(filter_focus)
        .tab_index(0)
        .cursor(CursorStyle::IBeam)
        .key_context("ContextFilter")
        .hover(|style| style.border_color(theme.selection_line))
        .on_key_down(cx.listener(|this, event, _, cx| {
            if this.handle_context_filter_key(event, cx) {
                cx.stop_propagation();
            }
        }))
        .id("context-filter")
        .on_click(cx.listener(move |_, _: &gpui::ClickEvent, window, _| {
            window.focus(&focus_handle);
        }))
        .child(search_glyph(theme))
        .child(div().min_w_0().truncate().child(label))
}

fn search_glyph(theme: RelayTheme) -> gpui::Div {
    div()
        .w(px(14.0))
        .h(px(14.0))
        .rounded_md()
        .border_1()
        .border_color(theme.muted)
}

fn tab(
    theme: RelayTheme,
    active_tab: ContextTab,
    tab: ContextTab,
    label: &'static str,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    div()
        .h(px(28.0))
        .flex_1()
        .px_2()
        .rounded_sm()
        .border_1()
        .border_color(if active_tab == tab {
            theme.selection
        } else {
            theme.chrome_alt
        })
        .bg(if active_tab == tab {
            theme.panel
        } else {
            theme.chrome_alt
        })
        .text_sm()
        .text_color(if active_tab == tab {
            theme.text
        } else {
            theme.muted
        })
        .cursor_pointer()
        .hover(|style| style.bg(theme.panel))
        .flex()
        .items_center()
        .justify_center()
        .id(("context-tab", tab.index()))
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _, cx| {
            this.dispatch(WorkbenchCommand::SetContextTab(tab), cx);
        }))
        .child(label)
}

impl ContextTab {
    fn label(self) -> &'static str {
        match self {
            Self::Files => "Files",
            Self::Diff => "Diff",
            Self::Review => "Review",
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Files => 0,
            Self::Diff => 1,
            Self::Review => 2,
        }
    }
}

fn files_tab(theme: RelayTheme, task: Option<&TaskProjection>, filter: &str) -> gpui::Div {
    let mut rows = div().flex().flex_col().gap_1();
    let mut row_count = 0;
    if let Some(task) = task {
        let changed_files = filtered_changed_files(task, filter);
        let tree = DiffTree::from_changed_files(&changed_files);
        for row in &tree.rows {
            row_count += 1;
            rows = rows.child(tree_row(theme, row));
        }
    }

    div()
        .flex_1()
        .p_3()
        .flex()
        .flex_col()
        .gap_3()
        .child(summary(theme, task))
        .child(if row_count == 0 {
            empty_state(theme, "No matching files", "File list is empty.")
        } else {
            rows
        })
}

fn diff_tab(theme: RelayTheme, task: Option<&TaskProjection>, filter: &str) -> gpui::Div {
    let mut hunks = div().flex().flex_col().gap_2();
    let mut hunk_count = 0;
    if let Some(task) = task {
        let changed_files = filtered_changed_files(task, filter);
        for file in &changed_files {
            hunk_count += 1;
            hunks = hunks.child(hunk_card(theme, file));
        }
    }

    div()
        .flex_1()
        .p_3()
        .flex()
        .flex_col()
        .gap_3()
        .child(summary(theme, task))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .text_color(theme.text)
                .child("Changed files")
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted)
                        .child(hunk_count.to_string()),
                ),
        )
        .child(if hunk_count == 0 {
            empty_state(theme, "No matching diffs", "Changed file list is empty.")
        } else {
            hunks
        })
}

fn review_tab(theme: RelayTheme, task: Option<&TaskProjection>, filter: &str) -> gpui::Div {
    let review_comments = filtered_review_comments(task, filter);
    let review_count = review_comments.len();
    let pending_count = review_comments
        .iter()
        .filter(|comment| !comment.delivered)
        .count();
    let mut comments = div().flex().flex_col().gap_2();
    for comment in &review_comments {
        comments = comments.child(review_comment(theme, comment));
    }

    div()
        .flex_1()
        .p_3()
        .flex()
        .flex_col()
        .gap_3()
        .child(summary(theme, task))
        .child(metric_row(theme, "Comments", review_count.to_string()))
        .child(metric_row(
            theme,
            "Pending delivery",
            pending_count.to_string(),
        ))
        .child(
            div()
                .border_b_1()
                .border_color(theme.line)
                .py_2()
                .flex()
                .items_center()
                .justify_between()
                .child(div().text_sm().text_color(theme.text).child("Delivery"))
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(if pending_count == 0 {
                            theme.muted
                        } else {
                            theme.accent
                        })
                        .child(if pending_count == 0 { "CLEAN" } else { "DIRTY" }),
                ),
        )
        .child(if review_count == 0 {
            empty_state(theme, "No matching review notes", "Review list is empty.")
        } else {
            comments
        })
}

fn filtered_changed_files(task: &TaskProjection, filter: &str) -> Vec<ChangedFile> {
    let filter = filter.trim().to_lowercase();
    if filter.is_empty() {
        return task.changed_files.clone();
    }

    task.changed_files
        .iter()
        .filter(|file| file.path.to_lowercase().contains(&filter))
        .cloned()
        .collect()
}

fn filtered_review_comments<'a>(
    task: Option<&'a TaskProjection>,
    filter: &str,
) -> Vec<&'a ReviewCommentProjection> {
    let Some(task) = task else {
        return Vec::new();
    };
    let filter = filter.trim().to_lowercase();
    task.review_comments
        .iter()
        .filter(|comment| {
            filter.is_empty()
                || comment.path.to_lowercase().contains(&filter)
                || comment.body.to_lowercase().contains(&filter)
        })
        .collect()
}

fn summary(theme: RelayTheme, task: Option<&TaskProjection>) -> gpui::Div {
    let title = task
        .map(|task| task.title.clone())
        .unwrap_or_else(|| "No active task".to_string());
    let status = task
        .map(|task| task.status_label.clone())
        .unwrap_or_else(|| "DETACHED".to_string());

    div()
        .pb_2()
        .border_b_1()
        .border_color(theme.line)
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_color(theme.text)
                        .child(title),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .text_xs()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.muted)
                        .child(status),
                ),
        )
        .child(
            div()
                .min_w_0()
                .truncate()
                .font_family("Consolas")
                .text_xs()
                .text_color(theme.muted)
                .child(
                    task.map_or_else(|| "No task metadata".to_string(), |task| task.meta.clone()),
                ),
        )
}

fn file_row(theme: RelayTheme, file: &ChangedFile) -> gpui::Div {
    let (label, color) = match file.status {
        ChangeStatus::Added => ("A", theme.accent),
        ChangeStatus::Modified => ("M", theme.warning),
        ChangeStatus::Deleted => ("D", theme.danger),
        ChangeStatus::Renamed => ("R", theme.warning),
        ChangeStatus::Untracked => ("U", theme.accent),
    };

    div()
        .rounded_md()
        .px_3()
        .py_2()
        .flex()
        .items_center()
        .gap_2()
        .bg(theme.chrome)
        .child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(color)
                .child(label),
        )
        .child(
            div()
                .min_w_0()
                .truncate()
                .text_sm()
                .text_color(theme.text)
                .child(file.path.clone()),
        )
}

fn tree_row(theme: RelayTheme, row: &DiffTreeRow) -> gpui::Div {
    match row.kind {
        DiffTreeRowKind::Directory => div()
            .px_2()
            .py_1()
            .ml(px((row.depth as f32) * 12.0))
            .text_xs()
            .text_color(theme.muted)
            .child(format!("{}/  {}", row.label, row.file_count)),
        DiffTreeRowKind::File => {
            let status = row.status.unwrap_or(ChangeStatus::Modified);
            file_row(
                theme,
                &ChangedFile {
                    path: row.path.clone(),
                    status,
                },
            )
            .ml(px((row.depth as f32) * 12.0))
        }
    }
}

fn hunk_card(theme: RelayTheme, file: &ChangedFile) -> gpui::Div {
    let (label, color) = change_label(theme, file.status);
    div()
        .rounded_sm()
        .border_1()
        .border_color(theme.line)
        .bg(theme.chrome_alt)
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(color)
                        .child(label),
                )
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_sm()
                        .text_color(theme.text)
                        .child(file.path.clone()),
                ),
        )
        .child(
            div()
                .font_family("Consolas")
                .text_xs()
                .text_color(theme.muted)
                .child(diff_body_state(file.status)),
        )
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

fn review_comment(theme: RelayTheme, comment: &ReviewCommentProjection) -> gpui::Div {
    div()
        .rounded_sm()
        .border_1()
        .border_color(theme.line)
        .bg(theme.chrome_alt)
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted)
                        .child(format!("{} · {}", comment.path, comment.line_label)),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(if comment.delivered {
                            theme.muted
                        } else {
                            theme.warning
                        })
                        .child(if comment.delivered { "SENT" } else { "PENDING" }),
                ),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.text)
                .child(comment.body.clone()),
        )
}

fn metric_row(theme: RelayTheme, label: &'static str, value: String) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .border_b_1()
        .border_color(theme.line)
        .py_2()
        .child(div().text_color(theme.muted).child(label))
        .child(
            div()
                .text_color(theme.text)
                .font_weight(gpui::FontWeight::BOLD)
                .child(value),
        )
}

fn change_label(theme: RelayTheme, status: ChangeStatus) -> (&'static str, gpui::Hsla) {
    match status {
        ChangeStatus::Added => ("A", theme.accent),
        ChangeStatus::Modified => ("M", theme.warning),
        ChangeStatus::Deleted => ("D", theme.danger),
        ChangeStatus::Renamed => ("R", theme.warning),
        ChangeStatus::Untracked => ("U", theme.accent),
    }
}

fn diff_body_state(status: ChangeStatus) -> &'static str {
    match status {
        ChangeStatus::Added | ChangeStatus::Untracked => "content diff pending",
        ChangeStatus::Modified => "line diff not loaded",
        ChangeStatus::Deleted => "deleted file body not loaded",
        ChangeStatus::Renamed => "rename details not loaded",
    }
}
