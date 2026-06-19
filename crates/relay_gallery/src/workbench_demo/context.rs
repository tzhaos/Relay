use gpui::{Entity, IntoElement, ParentElement, Styled, Window, div, prelude::FluentBuilder, px};
use relay_ui_kit::{
    Badge, DiffView, FileKind, FileView, IconButton, IconName, Pane, PaneSurface, PaneWidth,
    PanelHeader, StatusDot, Tab, Tabs, TerminalSessionRow, TextInput, TextInputAction, Theme, Tone,
    TreeRow,
};

use super::{
    WorkbenchState,
    data::{DEMO_FILES, active_session},
};
use crate::GalleryApp;

pub(super) fn right_context(
    state: &WorkbenchState,
    host: &Entity<GalleryApp>,
    window: &Window,
    theme: Theme,
) -> impl IntoElement {
    let tab = state.context_tab;
    let body = div()
        .size_full()
        .flex()
        .flex_col()
        .child(
            div().px_2().pt_2().child(
                Tabs::new(
                    "ctx-tabs",
                    vec![
                        Tab::new("files", "Files").icon(IconName::FileText),
                        Tab::new("diff", "Diff").icon(IconName::FileDiff).count(12),
                        Tab::new("review", "Review")
                            .icon(IconName::MessageSquareText)
                            .count(3),
                    ],
                )
                .active(tab)
                .on_select({
                    let host = host.clone();
                    move |key, _window, cx| {
                        host.update(cx, |this, cx| {
                            this.workbench.context_tab = key;
                            cx.notify();
                        });
                    }
                }),
            ),
        )
        .when(tab == "files", |this| {
            this.child(files_tab(state, host, window))
        })
        .when(tab == "diff", |this| this.child(diff_tab()))
        .when(tab == "review", |this| {
            this.child(review_tab(state, host, theme))
        });

    Pane::new(PaneWidth::Flex, body)
        .surface(PaneSurface::Chrome)
        .header(
            PanelHeader::new("Context")
                .icon(IconName::FileText)
                .trailing(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            IconButton::new("ctx-refresh", IconName::RefreshCw).on_click({
                                let host = host.clone();
                                move |_event, _window, cx| {
                                    host.update(cx, |this, cx| {
                                        this.workbench.filter.clear();
                                        cx.notify();
                                    });
                                }
                            }),
                        )
                        .child(IconButton::new("ctx-more", IconName::Ellipsis).on_click({
                            let host = host.clone();
                            move |_event, _window, cx| {
                                host.update(cx, |this, cx| {
                                    this.workbench.context_tab = "review";
                                    cx.notify();
                                });
                            }
                        })),
                ),
        )
}

fn files_tab(
    state: &WorkbenchState,
    host: &Entity<GalleryApp>,
    window: &Window,
) -> impl IntoElement {
    let host_for_key = host.clone();
    let filter_focused = state.filter_focus.is_focused(window);
    let filter_text = state.filter.value().to_lowercase();
    let files = DEMO_FILES
        .iter()
        .filter(|file| filter_text.is_empty() || file.name.to_lowercase().contains(&filter_text))
        .enumerate()
        .map(|(index, file)| {
            let mut row = TreeRow::new(("file", index), file.icon, file.name).depth(file.depth);
            if file.expandable {
                row = row.expandable(true);
            }
            if file.name == "workbench_demo.rs" {
                row = row.selected(true);
            }
            row.into_any_element()
        });

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .child(
            div().px_2().py_2().child(
                TextInput::new("file-filter", state.filter_focus.clone(), &state.filter)
                    .placeholder("Filter files")
                    .leading_icon(IconName::Funnel)
                    .focused(filter_focused)
                    .on_key(move |event, _window, cx| {
                        host_for_key.update(cx, |this, cx| {
                            match this.workbench.filter.handle_key(event) {
                                TextInputAction::Cancel => {
                                    this.workbench.filter.clear();
                                    cx.notify();
                                }
                                TextInputAction::Edited | TextInputAction::Submit => cx.notify(),
                                TextInputAction::Ignored => {}
                            }
                        });
                    }),
            ),
        )
        .child(
            div()
                .flex_1()
                .min_h_0()
                .px_2()
                .pb_2()
                .flex()
                .flex_col()
                .gap(px(1.0))
                .children(files),
        )
}

fn diff_tab() -> impl IntoElement {
    div().flex_1().min_h_0().p_2().child(
        FileView::new(
            "crates/relay_ui_kit/src/shell/split_pane.rs",
            FileKind::Diff,
            DiffView::from_text_diff(DIFF_OLD, DIFF_NEW),
        )
        .detail("+4 -1"),
    )
}

fn review_tab(state: &WorkbenchState, host: &Entity<GalleryApp>, theme: Theme) -> impl IntoElement {
    div()
        .flex_1()
        .min_h_0()
        .p_2()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            TerminalSessionRow::new(
                "review-session",
                active_session(state).label,
                active_session(state).subtitle,
            )
            .status(active_session(state).tone)
            .active(true)
            .on_click({
                let host = host.clone();
                move |_event, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.workbench.route = "terminal";
                        this.workbench.context_tab = "files";
                        cx.notify();
                    });
                }
            }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(StatusDot::new(Tone::Warning))
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(theme.text)
                        .child("3 comments pending delivery"),
                )
                .child(Badge::new("DRAFT").tone(Tone::Warning).soft()),
        )
        .child(
            div()
                .p_2()
                .rounded(px(relay_ui_kit::radius::MD))
                .bg(theme.panel)
                .border_1()
                .border_color(theme.border)
                .text_xs()
                .text_color(theme.text_secondary)
                .child("workbench_demo.rs: terminal launcher state should stay host-owned."),
        )
}

const DIFF_OLD: &str = r#"let center_and_context = SplitPane::new("center-context-split", center, right)
    .first_size(720.0)
    .min_sizes(560.0, 320.0);
"#;

const DIFF_NEW: &str = r#"let center_and_context = SplitPane::new("center-context-split", center, right)
    .first_size(state.terminal_width)
    .min_sizes(560.0, 320.0)
    .on_resize({
        let host = host.clone();
        move |next, _window, cx| {
            host.update(cx, |this, cx| {
                this.workbench.terminal_width = next;
                cx.notify();
            });
        }
    });
"#;
