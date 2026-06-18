use gpui::{IntoElement, div, prelude::*, px};
use relay_core::{ChangeStatus, ChangedFile, TaskProjection};

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
        for file in &task.changed_files {
            rows = rows.child(file_row(theme, file));
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
    let changed_count = task.map_or(0, |task| task.changed_file_count);
    div()
        .flex_1()
        .p_3()
        .flex()
        .flex_col()
        .gap_3()
        .child(summary(theme, task))
        .child(
            div()
                .rounded_md()
                .border_1()
                .border_color(theme.line)
                .bg(theme.chrome_alt)
                .p_3()
                .text_color(theme.text)
                .child(format!("{changed_count} changed files ready for hunk view")),
        )
        .child(
            div()
                .font_family("Consolas")
                .text_sm()
                .text_color(theme.muted)
                .child("+ relay_diff will expand this tab into file tree + hunks in Step 8"),
        )
}

fn review_tab(theme: RelayTheme, task: Option<&TaskProjection>) -> gpui::Div {
    let review_count = task.map_or(0, |task| task.review_comment_count);
    let pending_count = task.map_or(0, |task| task.pending_review_comment_count);
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
                .rounded_md()
                .border_1()
                .border_color(theme.line)
                .bg(theme.chrome_alt)
                .p_3()
                .text_color(theme.muted)
                .child("Review notes are task-scoped and can be sent to the active agent."),
        )
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
