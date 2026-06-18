use gpui::{div, prelude::*, px};
use relay_core::{PreviewTargetProjection, TaskProjection};

use crate::theme::RelayTheme;

pub fn preview_content(theme: RelayTheme, task: Option<&TaskProjection>) -> gpui::Div {
    let mut targets = div().flex().flex_col().gap_2();
    if let Some(task) = task {
        for target in &task.preview_targets {
            targets = targets.child(preview_target(theme, target));
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
                        .text_sm()
                        .text_color(theme.muted)
                        .child(preview_count_label(task)),
                ),
        )
        .child(
            div()
                .flex_1()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(active_preview_summary(theme, task))
                .child(targets)
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

fn preview_target(theme: RelayTheme, target: &PreviewTargetProjection) -> gpui::Div {
    div()
        .rounded_sm()
        .border_1()
        .border_color(theme.line)
        .bg(theme.chrome)
        .px_3()
        .py_2()
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
                .child(target.uri.clone()),
        )
}

fn preview_target_state(theme: RelayTheme, task: Option<&TaskProjection>) -> gpui::Div {
    let has_preview = task.is_some_and(|task| !task.preview_targets.is_empty());
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
        .child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(if has_preview {
                    theme.accent
                } else {
                    theme.muted
                })
                .child(if has_preview { "ATTACHED" } else { "DETACHED" }),
        )
}

fn preview_count_label(task: Option<&TaskProjection>) -> String {
    let count = task.map_or(0, |task| task.preview_target_count);
    match count {
        0 => "no targets".to_string(),
        1 => "1 target".to_string(),
        value => format!("{value} targets"),
    }
}
