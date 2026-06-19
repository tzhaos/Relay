use gpui::{Context, Entity, IntoElement, ParentElement, Styled, Window, div, px};
use relay_ui_kit::{Button, IconName, TaskRow, TaskRowData, Theme, Tone, TreeRow};

use super::{
    GalleryState,
    shared::{section, text_input_field},
    viewer_samples::viewer_sample,
};
use crate::GalleryApp;

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryApp>,
    window: &Window,
    theme: Theme,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    div()
        .h(px(650.0))
        .flex()
        .gap_3()
        .child(
            div()
                .w(px(320.0))
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap_3()
                .child(section(cx, "Files", file_tree_sample(state, host, window)))
                .child(section(
                    cx,
                    "Review state",
                    review_state_sample(host, theme),
                )),
        )
        .child(div().flex_1().min_w_0().child(section(
            cx,
            "Preview and diff",
            viewer_sample(state, host),
        )))
}

fn file_tree_sample(
    state: &GalleryState,
    host: &Entity<GalleryApp>,
    window: &Window,
) -> impl IntoElement {
    let search_focused = state.search_focus.is_focused(window);

    div()
        .h(px(510.0))
        .flex()
        .flex_col()
        .gap_2()
        .child(text_input_field(
            host,
            "review-filter",
            &state.search_input,
            state.search_focus.clone(),
            search_focused,
            Some(IconName::Funnel),
            "Filter files",
        ))
        .child(
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .gap(px(1.0))
                .child(TreeRow::new("review-crates", IconName::Folder, "crates").expandable(true))
                .child(TreeRow::new("review-kit", IconName::Folder, "relay_ui_kit").depth(1))
                .child(
                    TreeRow::new("review-viewer", IconName::FileDiff, "viewer/diff_view.rs")
                        .depth(2)
                        .selected(true),
                )
                .child(TreeRow::new("review-md", IconName::FileText, "DESIGN.md"))
                .child(TreeRow::new(
                    "review-cargo",
                    IconName::FileText,
                    "Cargo.toml",
                )),
        )
}

fn review_state_sample(host: &Entity<GalleryApp>, theme: Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            TaskRow::new(
                "review-task",
                TaskRowData {
                    title: "Review viewer components".into(),
                    status_label: "REVIEW".into(),
                    status_tone: Tone::Warning,
                    branch: Some("ui/viewers".into()),
                    changed: 8,
                    review: 3,
                },
            )
            .selected(true),
        )
        .child(
            Button::new("review-open-terminal", "Open Terminal")
                .icon(IconName::Terminal)
                .on_click({
                    let host = host.clone();
                    move |_event, _window, cx| {
                        host.update(cx, |this, cx| {
                            this.gallery.terminal_session = "codex";
                            cx.notify();
                        });
                    }
                }),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child("Review samples use real viewer components, not static text blocks."),
        )
}
