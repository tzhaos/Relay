use gpui::{
    Context, InteractiveElement, IntoElement, StatefulInteractiveElement, div, prelude::*, px,
};
use relay_core::{PreviewTargetProjection, TaskId, TaskProjection};

use crate::{
    app_shell::AppShell,
    components::{self, ButtonEmphasis, Tone},
    theme::{RelayTheme, mono_family, spacing},
    workbench::WorkbenchCommand,
};

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
        .bg(theme.app_bg)
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(spacing::PANE_HEADER))
                .px_4()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(theme.border)
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.text)
                        .child("Preview"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_sm()
                        .text_color(theme.text_muted)
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
                .child(preview_target_state(theme, task)),
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
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.panel)
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .min_w_0()
                .truncate()
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(theme.text)
                .child(title),
        )
        .child(
            div()
                .min_w_0()
                .truncate()
                .font_family(mono_family())
                .text_xs()
                .text_color(theme.text_muted)
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
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.panel)
        .px_3()
        .py_2()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .cursor_pointer()
        .hover(|style| style.bg(theme.hover).border_color(theme.border_strong))
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
                        .min_w_0()
                        .truncate()
                        .text_sm()
                        .text_color(theme.text)
                        .child(target.label.clone()),
                )
                .child(
                    div()
                        .font_family(mono_family())
                        .text_xs()
                        .text_color(theme.text_muted)
                        .truncate()
                        .child(target.uri.clone()),
                ),
        )
        .child(components::badge(theme, "OPEN", Tone::Secondary))
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
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.panel)
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
                        .font_family(mono_family())
                        .text_xs()
                        .text_color(theme.text_muted)
                        .truncate()
                        .child(worktree_label),
                ),
        )
        .children(attach_preview_action(theme, task, cx))
}

fn preview_target_state(theme: RelayTheme, task: Option<&TaskProjection>) -> gpui::Div {
    let has_preview = task.is_some_and(|task| !task.preview_targets.is_empty());
    let can_attach = task.is_some_and(|task| task.worktree_path.is_some());
    let (label, tone) = if has_preview {
        ("ATTACHED", Tone::Accent)
    } else if can_attach {
        ("READY", Tone::Warning)
    } else {
        ("DETACHED", Tone::Muted)
    };
    div()
        .border_b_1()
        .border_color(theme.border)
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
        .child(components::badge(theme, label, tone))
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

    let task_id = task.id;
    Some(components::button(
        theme,
        "Attach",
        ButtonEmphasis::Primary,
        (gpui::ElementId::from(task_id.as_uuid()), "attach-preview"),
        cx,
        move |this, cx| {
            this.dispatch(WorkbenchCommand::AttachWorktreePreview(task_id), cx);
        },
    ))
}

fn is_worktree_preview(target: &PreviewTargetProjection) -> bool {
    target.label == "Worktree" && target.uri.starts_with("file://")
}
