use gpui::{IntoElement, div, prelude::*, px};
use relay_core::{ChangeStatus, ChangedFile, ReviewCommentProjection, TaskProjection};
use relay_diff::{DiffTree, DiffTreeRow, DiffTreeRowKind};

use crate::{
    theme::RelayTheme,
    workbench::{ContextTab, WorkspaceViewModel},
};

pub fn context_pane(theme: RelayTheme, view_model: &WorkspaceViewModel) -> impl IntoElement {
    let active_task = view_model.active_task();

    div()
        .w(px(340.0))
        .h_full()
        .border_l_1()
        .border_color(theme.line)
        .bg(theme.chrome)
        .flex()
        .flex_col()
        .child(header(theme, view_model.context_tab))
        .child(match view_model.context_tab {
            ContextTab::Files => files_tab(theme, active_task),
            ContextTab::Diff => diff_tab(theme, active_task),
            ContextTab::Review => review_tab(theme, active_task),
        })
}

fn header(theme: RelayTheme, active_tab: ContextTab) -> gpui::Div {
    div()
        .h(px(42.0))
        .px_3()
        .flex()
        .items_center()
        .justify_between()
        .border_b_1()
        .border_color(theme.line)
        .child(div().text_color(theme.text).child("Context"))
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(tab(theme, active_tab, ContextTab::Files, "Files"))
                .child(tab(theme, active_tab, ContextTab::Diff, "Diff"))
                .child(tab(theme, active_tab, ContextTab::Review, "Review")),
        )
}

fn tab(
    theme: RelayTheme,
    active_tab: ContextTab,
    tab: ContextTab,
    label: &'static str,
) -> impl IntoElement {
    div()
        .px_2()
        .py_1()
        .rounded_md()
        .bg(if active_tab == tab {
            theme.selection
        } else {
            theme.chrome
        })
        .text_sm()
        .text_color(if active_tab == tab {
            theme.text
        } else {
            theme.muted
        })
        .child(label)
}

fn files_tab(theme: RelayTheme, task: Option<&TaskProjection>) -> gpui::Div {
    let mut rows = div().flex().flex_col().gap_1();
    if let Some(task) = task {
        let tree = DiffTree::from_changed_files(&task.changed_files);
        for row in &tree.rows {
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
        .child(rows)
}

fn diff_tab(theme: RelayTheme, task: Option<&TaskProjection>) -> gpui::Div {
    let mut hunks = div().flex().flex_col().gap_2();
    if let Some(task) = task {
        for file in &task.changed_files {
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
                .child("Hunks")
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted)
                        .child("refresh preserves review history"),
                ),
        )
        .child(hunks)
}

fn review_tab(theme: RelayTheme, task: Option<&TaskProjection>) -> gpui::Div {
    let review_count = task.map_or(0, |task| task.review_comment_count);
    let pending_count = task.map_or(0, |task| task.pending_review_comment_count);
    let mut comments = div().flex().flex_col().gap_2();
    if let Some(task) = task {
        for comment in &task.review_comments {
            comments = comments.child(review_comment(theme, comment));
        }
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
                .rounded_sm()
                .border_1()
                .border_color(theme.line)
                .bg(theme.chrome_alt)
                .px_3()
                .py_2()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.text)
                        .child("Send pending notes"),
                )
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
        .child(comments)
}

fn summary(theme: RelayTheme, task: Option<&TaskProjection>) -> gpui::Div {
    let title = task
        .map(|task| task.title.clone())
        .unwrap_or_else(|| "No active task".to_string());
    let status = task
        .map(|task| task.status_label.clone())
        .unwrap_or_else(|| "DETACHED".to_string());

    div()
        .rounded_md()
        .border_1()
        .border_color(theme.line)
        .bg(theme.chrome_alt)
        .p_3()
        .flex()
        .items_center()
        .justify_between()
        .child(div().text_color(theme.text).child(title))
        .child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(theme.muted)
                .child(status),
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
                .child(hunk_preview(file.status)),
        )
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

fn hunk_preview(status: ChangeStatus) -> &'static str {
    match status {
        ChangeStatus::Added | ChangeStatus::Untracked => "+ new lines ready for review",
        ChangeStatus::Modified => "- previous line\n+ updated line",
        ChangeStatus::Deleted => "- removed lines",
        ChangeStatus::Renamed => "rename path with content preserved",
    }
}
