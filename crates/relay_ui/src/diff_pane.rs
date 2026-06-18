use gpui::{
    Context, CursorStyle, FocusHandle, InteractiveElement, IntoElement, StatefulInteractiveElement,
    div, prelude::*, px,
};
use relay_core::{
    ChangeStatus, ChangedFile, DiffFileProjection, DiffLineProjection, DiffLineProjectionKind,
    DiffSide, LineIdentity, ReviewCommentProjection, TaskId, TaskProjection,
};
use relay_diff::{DiffTree, DiffTreeRow, DiffTreeRowKind};

use crate::{
    app_shell::AppShell,
    theme::RelayTheme,
    workbench::{
        ContextTab, ReviewDraftState, ReviewDraftTarget, WorkbenchCommand, WorkspaceViewModel,
    },
};

pub fn context_pane(
    theme: RelayTheme,
    view_model: &WorkspaceViewModel,
    filter_focus: &FocusHandle,
    review_draft_focus: &FocusHandle,
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
            ContextTab::Files => files_tab(theme, active_task, filter, cx),
            ContextTab::Diff => diff_tab(
                theme,
                active_task,
                filter,
                view_model.review_draft.target.as_ref(),
                review_draft_focus,
                cx,
            ),
            ContextTab::Review => review_tab(
                theme,
                active_task,
                filter,
                &view_model.review_draft,
                review_draft_focus,
                cx,
            ),
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

fn files_tab(
    theme: RelayTheme,
    task: Option<&TaskProjection>,
    filter: &str,
    cx: &mut Context<AppShell>,
) -> gpui::AnyElement {
    let mut rows = div().flex().flex_col().gap_1();
    let mut row_count = 0;
    if let Some(task) = task {
        let changed_files = filtered_changed_files(task, filter);
        let tree = DiffTree::from_changed_files(&changed_files);
        for row in &tree.rows {
            row_count += 1;
            rows = rows.child(tree_row(theme, row, cx));
        }
    }

    div()
        .id("files-tab-scroll")
        .flex_1()
        .min_h_0()
        .p_3()
        .overflow_y_scroll()
        .overflow_x_hidden()
        .flex()
        .flex_col()
        .gap_3()
        .child(summary(theme, task))
        .child(if row_count == 0 {
            empty_state(theme, "No matching files", "File list is empty.")
        } else {
            rows
        })
        .into_any_element()
}

fn diff_tab(
    theme: RelayTheme,
    task: Option<&TaskProjection>,
    filter: &str,
    review_target: Option<&ReviewDraftTarget>,
    review_draft_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> gpui::AnyElement {
    let diff_files = filtered_diff_files(task, filter);
    let task_id = task.map(|task| task.id);
    let review_context = ReviewTargetContext {
        task_id,
        selected: review_target,
        focus: review_draft_focus,
    };
    let file_count = diff_files.len();
    let (additions, deletions) = task
        .map(|task| (task.diff.stats.additions, task.diff.stats.deletions))
        .unwrap_or_default();
    let mut files = div().flex().flex_col().gap_2();
    for file in &diff_files {
        files = files.child(diff_file(theme, file, review_context, cx));
    }

    div()
        .id("diff-tab-scroll")
        .flex_1()
        .min_h_0()
        .p_3()
        .overflow_y_scroll()
        .overflow_x_hidden()
        .flex()
        .flex_col()
        .gap_3()
        .child(summary(theme, task))
        .child(diff_stats_row(theme, file_count, additions, deletions))
        .child(if file_count == 0 {
            empty_state(theme, "No matching diffs", "Changed file list is empty.")
        } else {
            files
        })
        .into_any_element()
}

fn review_tab(
    theme: RelayTheme,
    task: Option<&TaskProjection>,
    filter: &str,
    draft: &ReviewDraftState,
    review_draft_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> gpui::AnyElement {
    let review_comments = filtered_review_comments(task, filter);
    let review_count = review_comments.len();
    let pending_count = review_comments
        .iter()
        .filter(|comment| !comment.delivered)
        .count();
    let deliverable_task = task.filter(|task| {
        pending_count > 0 && task.agent.is_some() && task.terminal_session_id.is_some()
    });
    let mut comments = div().flex().flex_col().gap_2();
    for comment in &review_comments {
        comments = comments.child(review_comment(theme, comment, cx));
    }

    div()
        .id("review-tab-scroll")
        .flex_1()
        .min_h_0()
        .p_3()
        .overflow_y_scroll()
        .overflow_x_hidden()
        .flex()
        .flex_col()
        .gap_3()
        .child(summary(theme, task))
        .child(review_composer(theme, draft, review_draft_focus, cx))
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
                .child(delivery_control(theme, pending_count, deliverable_task, cx)),
        )
        .child(if review_count == 0 {
            empty_state(theme, "No matching review notes", "Review list is empty.")
        } else {
            comments
        })
        .into_any_element()
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

fn filtered_diff_files<'a>(
    task: Option<&'a TaskProjection>,
    filter: &str,
) -> Vec<&'a DiffFileProjection> {
    let Some(task) = task else {
        return Vec::new();
    };
    let filter = filter.trim().to_lowercase();
    task.diff
        .files
        .iter()
        .filter(|file| filter.is_empty() || file.path.to_lowercase().contains(&filter))
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

fn file_row(theme: RelayTheme, file: &ChangedFile, cx: &mut Context<AppShell>) -> impl IntoElement {
    let (label, color) = match file.status {
        ChangeStatus::Added => ("A", theme.accent),
        ChangeStatus::Modified => ("M", theme.warning),
        ChangeStatus::Deleted => ("D", theme.danger),
        ChangeStatus::Renamed => ("R", theme.warning),
        ChangeStatus::Untracked => ("U", theme.accent),
    };
    let path = file.path.clone();

    div()
        .rounded_md()
        .px_3()
        .py_2()
        .flex()
        .items_center()
        .gap_2()
        .bg(theme.chrome)
        .cursor_pointer()
        .hover(|style| style.bg(theme.panel).border_color(theme.selection_line))
        .id((
            gpui::ElementId::from(gpui::SharedString::from(path.clone())),
            "changed-file",
        ))
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _, cx| {
            this.dispatch(WorkbenchCommand::OpenChangedFile(path.clone()), cx);
        }))
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

fn tree_row(theme: RelayTheme, row: &DiffTreeRow, cx: &mut Context<AppShell>) -> gpui::AnyElement {
    match row.kind {
        DiffTreeRowKind::Directory => div()
            .px_2()
            .py_1()
            .ml(px((row.depth as f32) * 12.0))
            .text_xs()
            .text_color(theme.muted)
            .child(format!("{}/  {}", row.label, row.file_count))
            .into_any_element(),
        DiffTreeRowKind::File => {
            let status = row.status.unwrap_or(ChangeStatus::Modified);
            div()
                .ml(px((row.depth as f32) * 12.0))
                .child(file_row(
                    theme,
                    &ChangedFile {
                        path: row.path.clone(),
                        status,
                    },
                    cx,
                ))
                .into_any_element()
        }
    }
}

fn delivery_control(
    theme: RelayTheme,
    pending_count: usize,
    task: Option<&TaskProjection>,
    cx: &mut Context<AppShell>,
) -> gpui::AnyElement {
    if let Some(task) = task {
        return deliver_review_button(theme, task.id, cx).into_any_element();
    }

    let (label, color) = if pending_count == 0 {
        ("CLEAN", theme.muted)
    } else {
        ("NEEDS AGENT", theme.warning)
    };
    div()
        .h(px(24.0))
        .px_2()
        .rounded_sm()
        .border_1()
        .border_color(theme.line)
        .bg(theme.chrome_alt)
        .flex()
        .items_center()
        .text_xs()
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(color)
        .child(label)
        .into_any_element()
}

fn deliver_review_button(
    theme: RelayTheme,
    task_id: relay_core::TaskId,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    div()
        .h(px(24.0))
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
        .id((gpui::ElementId::from(task_id.as_uuid()), "deliver-review"))
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _, cx| {
            this.dispatch(WorkbenchCommand::DeliverReview(task_id), cx);
        }))
        .child("Deliver")
}

fn diff_stats_row(
    theme: RelayTheme,
    file_count: usize,
    additions: usize,
    deletions: usize,
) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .text_color(theme.text)
        .child("Diff")
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .text_xs()
                .text_color(theme.muted)
                .child(format!("{file_count} files"))
                .child(
                    div()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.accent)
                        .child(format!("+{additions}")),
                )
                .child(
                    div()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.danger)
                        .child(format!("-{deletions}")),
                ),
        )
}

#[derive(Clone, Copy)]
struct ReviewTargetContext<'a> {
    task_id: Option<TaskId>,
    selected: Option<&'a ReviewDraftTarget>,
    focus: &'a FocusHandle,
}

fn diff_file(
    theme: RelayTheme,
    file: &DiffFileProjection,
    review_context: ReviewTargetContext<'_>,
    cx: &mut Context<AppShell>,
) -> gpui::Div {
    let (label, color) = change_label(theme, file.status);
    let mut hunks = div().flex().flex_col().gap_1();
    for hunk in &file.hunks {
        hunks = hunks.child(diff_hunk(theme, &file.path, hunk, review_context, cx));
    }

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
        .child(if file.is_binary {
            div()
                .font_family("Consolas")
                .text_xs()
                .text_color(theme.muted)
                .child("binary file")
        } else if file.hunks.is_empty() {
            div()
                .font_family("Consolas")
                .text_xs()
                .text_color(theme.muted)
                .child("no line hunks")
        } else {
            hunks
        })
}

fn diff_hunk(
    theme: RelayTheme,
    path: &str,
    hunk: &relay_core::DiffHunkProjection,
    review_context: ReviewTargetContext<'_>,
    cx: &mut Context<AppShell>,
) -> gpui::Div {
    let mut lines = div().flex().flex_col();
    for line in &hunk.lines {
        lines = lines.child(diff_line(
            theme,
            path,
            &hunk.header,
            line,
            review_context,
            cx,
        ));
    }

    div()
        .border_1()
        .border_color(theme.line)
        .bg(theme.panel)
        .flex()
        .flex_col()
        .child(
            div()
                .px_2()
                .py_1()
                .font_family("Consolas")
                .text_xs()
                .text_color(theme.muted)
                .bg(theme.chrome)
                .child(hunk.header.clone()),
        )
        .child(lines)
}

fn diff_line(
    theme: RelayTheme,
    path: &str,
    hunk_header: &str,
    line: &DiffLineProjection,
    review_context: ReviewTargetContext<'_>,
    cx: &mut Context<AppShell>,
) -> gpui::AnyElement {
    let (marker, color, background) = match line.kind {
        DiffLineProjectionKind::Added => ("+", theme.accent, theme.selection),
        DiffLineProjectionKind::Deleted => ("-", theme.danger, theme.chrome_alt),
        DiffLineProjectionKind::NoNewline => ("\\", theme.muted, theme.panel),
        DiffLineProjectionKind::Context => (" ", theme.muted, theme.panel),
    };
    let line_label = match (line.old_line, line.new_line) {
        (Some(old), Some(new)) => format!("{old:>3} {new:>3}"),
        (Some(old), None) => format!("{old:>3}    "),
        (None, Some(new)) => format!("    {new:>3}"),
        (None, None) => "       ".to_string(),
    };
    let line_identity = line_identity(path, hunk_header, line);
    let selected = review_context.selected.is_some_and(|target| {
        review_context
            .task_id
            .is_some_and(|task_id| target.task_id == task_id)
            && target.line == line_identity
    });
    let selected_text = if line.content.is_empty() {
        None
    } else {
        Some(line.content.clone())
    };

    let row = div()
        .min_w_0()
        .bg(background)
        .border_1()
        .border_color(if selected {
            theme.selection_line
        } else {
            background
        })
        .px_2()
        .py_0p5()
        .flex()
        .items_start()
        .gap_2()
        .font_family("Consolas")
        .text_xs()
        .child(
            div()
                .flex_shrink_0()
                .w(px(54.0))
                .text_color(theme.muted)
                .child(line_label),
        )
        .child(
            div()
                .flex_shrink_0()
                .w(px(10.0))
                .text_color(color)
                .child(marker),
        )
        .child(
            div()
                .min_w_0()
                .flex_1()
                .text_color(theme.text)
                .child(line.content.clone()),
        )
        .child(note_target_badge(theme, selected));

    if let Some(task_id) = review_context.task_id {
        let focus_handle = review_context.focus.clone();
        let element_id = review_line_element_id(task_id, &line_identity);
        let target = ReviewDraftTarget {
            task_id,
            line: line_identity,
            selected_text,
        };
        row.cursor_pointer()
            .hover(|style| style.bg(theme.panel).border_color(theme.selection_line))
            .id(element_id)
            .on_click(cx.listener(move |this, _: &gpui::ClickEvent, window, cx| {
                this.dispatch(WorkbenchCommand::SelectReviewTarget(target.clone()), cx);
                window.focus(&focus_handle);
            }))
            .into_any_element()
    } else {
        row.into_any_element()
    }
}

fn review_line_element_id(task_id: TaskId, line: &LineIdentity) -> gpui::ElementId {
    let side = match line.side {
        DiffSide::Old => "old",
        DiffSide::New => "new",
    };
    let suffix = format!(
        "review-line:{}:{}:{}:{}:{}",
        line.path,
        side,
        line.old_line.unwrap_or_default(),
        line.new_line.unwrap_or_default(),
        line.hunk_header
    );
    gpui::ElementId::from((gpui::ElementId::from(task_id.as_uuid()), suffix))
}

fn note_target_badge(theme: RelayTheme, selected: bool) -> gpui::Div {
    div()
        .flex_shrink_0()
        .h(px(18.0))
        .min_w(px(18.0))
        .rounded_sm()
        .border_1()
        .border_color(if selected {
            theme.selection_line
        } else {
            theme.line
        })
        .bg(if selected {
            theme.selection
        } else {
            theme.chrome
        })
        .flex()
        .items_center()
        .justify_center()
        .text_color(if selected { theme.text } else { theme.muted })
        .child("+")
}

fn line_identity(path: &str, hunk_header: &str, line: &DiffLineProjection) -> LineIdentity {
    LineIdentity {
        path: path.to_string(),
        side: match line.kind {
            DiffLineProjectionKind::Deleted => DiffSide::Old,
            DiffLineProjectionKind::Added
            | DiffLineProjectionKind::Context
            | DiffLineProjectionKind::NoNewline => DiffSide::New,
        },
        old_line: line.old_line,
        new_line: line.new_line,
        hunk_header: hunk_header.to_string(),
    }
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

fn review_composer(
    theme: RelayTheme,
    draft: &ReviewDraftState,
    review_draft_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> gpui::Div {
    let focus_handle = review_draft_focus.clone();
    let body_label = if draft.body.is_empty() {
        "Write a review note".to_string()
    } else {
        draft.body.clone()
    };
    let target_label = draft
        .target
        .as_ref()
        .map(|target| format!("{} - {}", target.path(), target.line_label()))
        .unwrap_or_else(|| "No line selected".to_string());
    let can_submit = draft.target.is_some() && !draft.body.trim().is_empty();

    div()
        .rounded_sm()
        .border_1()
        .border_color(if draft.target.is_some() {
            theme.selection_line
        } else {
            theme.line
        })
        .bg(theme.panel)
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.text)
                        .child("New note"),
                )
                .child(review_target_state(theme, draft.target.is_some())),
        )
        .child(
            div()
                .min_w_0()
                .truncate()
                .font_family("Consolas")
                .text_xs()
                .text_color(theme.muted)
                .child(target_label),
        )
        .child(
            div()
                .min_h(px(34.0))
                .rounded_md()
                .border_1()
                .border_color(if draft.body.is_empty() {
                    theme.line
                } else {
                    theme.selection_line
                })
                .bg(theme.chrome)
                .px_3()
                .py_2()
                .flex()
                .items_center()
                .text_sm()
                .text_color(if draft.body.is_empty() {
                    theme.muted
                } else {
                    theme.text
                })
                .track_focus(review_draft_focus)
                .tab_index(0)
                .cursor(CursorStyle::IBeam)
                .key_context("ReviewDraft")
                .focus(|style| style.border_color(theme.selection_line))
                .on_key_down(cx.listener(|this, event, _, cx| {
                    if this.handle_review_draft_key(event, cx) {
                        cx.stop_propagation();
                    }
                }))
                .id("review-draft-input")
                .on_click(cx.listener(move |_, _: &gpui::ClickEvent, window, _| {
                    window.focus(&focus_handle);
                }))
                .child(div().min_w_0().truncate().child(body_label)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted)
                        .child(review_draft_status(draft)),
                )
                .child(if can_submit {
                    save_review_button(theme, cx).into_any_element()
                } else {
                    review_state_badge(theme, "DRAFT", theme.muted).into_any_element()
                }),
        )
}

fn review_target_state(theme: RelayTheme, selected: bool) -> gpui::Div {
    if selected {
        review_state_badge(theme, "LINE", theme.accent)
    } else {
        review_state_badge(theme, "NO LINE", theme.warning)
    }
}

fn review_draft_status(draft: &ReviewDraftState) -> &'static str {
    match (draft.target.is_some(), draft.body.trim().is_empty()) {
        (false, _) => "No target",
        (true, true) => "Empty body",
        (true, false) => "Ready",
    }
}

fn review_state_badge(theme: RelayTheme, label: &'static str, color: gpui::Hsla) -> gpui::Div {
    div()
        .h(px(22.0))
        .px_2()
        .rounded_sm()
        .border_1()
        .border_color(theme.line)
        .bg(theme.chrome_alt)
        .flex()
        .items_center()
        .text_xs()
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(color)
        .child(label)
}

fn save_review_button(theme: RelayTheme, cx: &mut Context<AppShell>) -> impl IntoElement {
    div()
        .h(px(24.0))
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
        .id("save-review-draft")
        .on_click(cx.listener(|this, _: &gpui::ClickEvent, _, cx| {
            this.dispatch(WorkbenchCommand::SubmitReviewDraft, cx);
        }))
        .child("Save")
}

fn review_comment(
    theme: RelayTheme,
    comment: &ReviewCommentProjection,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let path = comment.path.clone();
    div()
        .rounded_sm()
        .border_1()
        .border_color(theme.line)
        .bg(theme.chrome_alt)
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .cursor_pointer()
        .hover(|style| style.bg(theme.panel).border_color(theme.selection_line))
        .id((
            gpui::ElementId::from(comment.id.as_uuid()),
            "review-comment",
        ))
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _, cx| {
            this.dispatch(WorkbenchCommand::OpenChangedFile(path.clone()), cx);
        }))
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
