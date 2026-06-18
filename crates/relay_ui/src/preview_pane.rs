use gpui::{
    Context, InteractiveElement, IntoElement, StatefulInteractiveElement, div, prelude::*, px,
};
use relay_core::{PreviewTargetProjection, TaskId, TaskProjection};

use crate::{app_shell::AppShell, theme::RelayTheme, workbench::WorkbenchCommand};

pub fn preview_content(
    theme: RelayTheme,
    task: Option<&TaskProjection>,
    cx: &mut Context<AppShell>,
) -> gpui::Div {
    let mut targets = div().flex().flex_col().gap_2();
    if let Some(task) = task {
        for target in &task.preview_targets {
            targets = targets.child(preview_target(theme, task.id, target, cx));
        }
    }

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
                .child(div().text_color(theme.text).child("Preview"))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_sm()
                        .text_color(theme.muted)
                        .child(preview_count_label(task)),
                ),
        )
        .child(
            div()
                .id("preview-scroll")
                .flex_1()
                .min_h_0()
                .p_4()
                .overflow_y_scroll()
                .overflow_x_hidden()
                .flex()
                .flex_col()
                .gap_3()
                .child(active_preview_summary(theme, task))
                .child(
                    if task.is_some_and(|task| task.preview_targets.is_empty()) {
                        empty_preview_state(theme, task, cx).into_any_element()
                    } else {
                        targets.into_any_element()
                    },
                )
                .child(preview_target_state(theme, task, cx)),
        )
}

fn active_preview_summary(theme: RelayTheme, task: Option<&TaskProjection>) -> gpui::Div {
    let title = task
        .map(|task| task.title.clone())
        .unwrap_or_else(|| "No active task".to_string());
    let target = task
        .and_then(|task| task.preview_targets.first())
        .map(|target| target.uri.clone())
        .unwrap_or_else(|| "No preview target attached".to_string());

    div()
        .rounded_sm()
        .border_1()
        .border_color(theme.line)
        .bg(theme.chrome_alt)
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_color(theme.text).child(title))
        .child(
            div()
                .font_family("Consolas")
                .text_sm()
                .text_color(theme.muted)
                .child(target),
        )
}

fn preview_target(
    theme: RelayTheme,
    task_id: TaskId,
    target: &PreviewTargetProjection,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let target_id = target.id;
    div()
        .rounded_sm()
        .border_1()
        .border_color(theme.line)
        .bg(theme.chrome)
        .px_3()
        .py_2()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .cursor_pointer()
        .hover(|style| style.bg(theme.panel).border_color(theme.selection_line))
        .id((gpui::ElementId::from(target_id.as_uuid()), "preview-target"))
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _, cx| {
            this.dispatch(
                WorkbenchCommand::OpenPreviewTarget { task_id, target_id },
                cx,
            );
        }))
        .child(
            div()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.text)
                        .child(target.label.clone()),
                )
                .child(
                    div()
                        .font_family("Consolas")
                        .text_xs()
                        .text_color(theme.muted)
                        .truncate()
                        .child(target.uri.clone()),
                ),
        )
        .child(open_preview_badge(theme))
}

fn empty_preview_state(
    theme: RelayTheme,
    task: Option<&TaskProjection>,
    cx: &mut Context<AppShell>,
) -> gpui::Div {
    let worktree_label = task
        .and_then(|task| task.worktree_path.clone())
        .unwrap_or_else(|| "No worktree".to_string());

    div()
        .rounded_sm()
        .border_1()
        .border_color(theme.line)
        .bg(theme.chrome)
        .px_3()
        .py_2()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            div()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_1()
                .child(div().text_sm().text_color(theme.text).child("Worktree"))
                .child(
                    div()
                        .font_family("Consolas")
                        .text_xs()
                        .text_color(theme.muted)
                        .truncate()
                        .child(worktree_label),
                ),
        )
        .children(attach_preview_action(theme, task, cx))
}

fn preview_target_state(
    theme: RelayTheme,
    task: Option<&TaskProjection>,
    _cx: &mut Context<AppShell>,
) -> gpui::Div {
    let has_preview = task.is_some_and(|task| !task.preview_targets.is_empty());
    let can_attach = task.is_some_and(|task| task.worktree_path.is_some());
    div()
        .border_b_1()
        .border_color(theme.line)
        .py_2()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_sm()
                .text_color(theme.text)
                .child("Preview target"),
        )
        .child(if has_preview {
            preview_state_badge(theme, "ATTACHED", theme.accent).into_any_element()
        } else if can_attach {
            preview_state_badge(theme, "READY", theme.warning).into_any_element()
        } else {
            preview_state_badge(theme, "DETACHED", theme.muted).into_any_element()
        })
}

fn preview_count_label(task: Option<&TaskProjection>) -> String {
    let count = task.map_or(0, |task| task.preview_target_count);
    match count {
        0 => "no targets".to_string(),
        1 => "1 target".to_string(),
        value => format!("{value} targets"),
    }
}

fn attach_preview_action(
    theme: RelayTheme,
    task: Option<&TaskProjection>,
    cx: &mut Context<AppShell>,
) -> Option<gpui::AnyElement> {
    let task = task?;
    if task.worktree_path.is_none() || task.preview_targets.iter().any(is_worktree_preview) {
        return None;
    }

    Some(attach_worktree_button(theme, task.id, cx).into_any_element())
}

fn is_worktree_preview(target: &PreviewTargetProjection) -> bool {
    target.label == "Worktree" && target.uri.starts_with("file://")
}

fn attach_worktree_button(
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
        .id((gpui::ElementId::from(task_id.as_uuid()), "attach-preview"))
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _, cx| {
            this.dispatch(WorkbenchCommand::AttachWorktreePreview(task_id), cx);
        }))
        .child("Attach")
}

fn open_preview_badge(theme: RelayTheme) -> gpui::Div {
    div()
        .flex_shrink_0()
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
        .child("Open")
}

fn preview_state_badge(theme: RelayTheme, label: &'static str, color: gpui::Hsla) -> gpui::Div {
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
}
