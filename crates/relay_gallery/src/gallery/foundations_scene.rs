use gpui::{Context, Entity, IntoElement, ParentElement, Styled, div};
use relay_ui_kit::{
    Button, ButtonVariant, Divider, EmptyState, IconButton, IconName, NavRow, Segment,
    SegmentedControl, Tab, Tabs, TaskRow, TaskRowData, Theme, Tone, TreeRow,
};

use super::{
    GalleryState,
    shared::{dot_label, icon_sample, scene_stack, section, strip},
};
use crate::GalleryApp;

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryApp>,
    theme: Theme,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    scene_stack()
        .child(section(cx, "Buttons", button_samples(host)))
        .child(section(cx, "Icon buttons", icon_button_samples(host)))
        .child(section(
            cx,
            "Status and icons",
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    strip()
                        .child(dot_label(theme, Tone::Accent, "running"))
                        .child(dot_label(theme, Tone::Warning, "waiting"))
                        .child(dot_label(theme, Tone::Danger, "failed"))
                        .child(dot_label(theme, Tone::Muted, "idle")),
                )
                .child(
                    strip()
                        .child(icon_sample(theme, IconName::Terminal))
                        .child(icon_sample(theme, IconName::Folder))
                        .child(icon_sample(theme, IconName::FileText))
                        .child(icon_sample(theme, IconName::FileDiff))
                        .child(icon_sample(theme, IconName::GitBranch))
                        .child(icon_sample(theme, IconName::Bot))
                        .child(icon_sample(theme, IconName::Search))
                        .child(icon_sample(theme, IconName::Zap))
                        .child(icon_sample(theme, IconName::MessageSquareText)),
                ),
        ))
        .child(section(
            cx,
            "Navigation rows",
            div()
                .flex()
                .items_start()
                .gap_4()
                .flex_wrap()
                .child(nav_rows_sample())
                .child(tree_rows_sample())
                .child(task_rows_sample()),
        ))
        .child(section(
            cx,
            "Tabs and empty state",
            tab_samples(state, host),
        ))
}

fn button_samples(host: &Entity<GalleryApp>) -> impl IntoElement {
    strip()
        .child(
            Button::new("btn-primary", "Launch Agent")
                .primary()
                .icon(IconName::Play)
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
            Button::new("btn-secondary", "Refresh")
                .icon(IconName::RefreshCw)
                .on_click({
                    let host = host.clone();
                    move |_event, _window, cx| {
                        host.update(cx, |this, cx| {
                            this.gallery.search_input.clear();
                            cx.notify();
                        });
                    }
                }),
        )
        .child(
            Button::new("btn-ghost", "Archive")
                .ghost()
                .icon(IconName::Archive)
                .on_click({
                    let host = host.clone();
                    move |_event, _window, cx| {
                        host.update(cx, |this, cx| {
                            this.gallery.auto_archive = !this.gallery.auto_archive;
                            cx.notify();
                        });
                    }
                }),
        )
        .child(
            Button::new("btn-disabled", "Disabled")
                .variant(ButtonVariant::Secondary)
                .disabled(true),
        )
}

fn icon_button_samples(host: &Entity<GalleryApp>) -> impl IntoElement {
    strip()
        .child(
            IconButton::new("ib-filter", IconName::ListFilter).on_click({
                let host = host.clone();
                move |_event, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.gallery.seg_tab = "files";
                        cx.notify();
                    });
                }
            }),
        )
        .child(
            IconButton::new("ib-refresh", IconName::RefreshCw).on_click({
                let host = host.clone();
                move |_event, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.gallery.search_input.clear();
                        cx.notify();
                    });
                }
            }),
        )
        .child(
            IconButton::new("ib-settings", IconName::Settings).on_click({
                let host = host.clone();
                move |_event, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.gallery.launcher_choice = "settings";
                        cx.notify();
                    });
                }
            }),
        )
        .child(IconButton::new("ib-active", IconName::PanelLeft).active(true))
}

fn nav_rows_sample() -> impl IntoElement {
    div()
        .w(gpui::px(280.0))
        .flex()
        .flex_col()
        .gap_1()
        .child(
            NavRow::new("nav-tasks", IconName::ListChecks, "Tasks")
                .count(3)
                .selected(true),
        )
        .child(NavRow::new(
            "nav-terminals",
            IconName::Terminal,
            "Terminals",
        ))
        .child(NavRow::new("nav-search", IconName::Search, "Search"))
}

fn tree_rows_sample() -> impl IntoElement {
    div()
        .w(gpui::px(300.0))
        .flex()
        .flex_col()
        .child(
            TreeRow::new("tr-1", IconName::Folder, "crates")
                .expandable(true)
                .depth(0),
        )
        .child(
            TreeRow::new("tr-2", IconName::Folder, "relay_ui_kit")
                .expandable(false)
                .depth(1),
        )
        .child(
            TreeRow::new("tr-3", IconName::FileText, "theme.rs")
                .depth(2)
                .selected(true),
        )
        .child(TreeRow::new("tr-4", IconName::FileText, "icon.rs").depth(2))
}

fn task_rows_sample() -> impl IntoElement {
    div()
        .w(gpui::px(320.0))
        .flex()
        .flex_col()
        .gap_1()
        .child(
            TaskRow::new(
                "task-1",
                TaskRowData {
                    title: "Wire diff pane".into(),
                    status_label: "RUNNING".into(),
                    status_tone: Tone::Accent,
                    branch: Some("relay/diff-pane".into()),
                    changed: 12,
                    review: 0,
                },
            )
            .selected(true),
        )
        .child(TaskRow::new(
            "task-2",
            TaskRowData {
                title: "Refactor terminal session".into(),
                status_label: "WAITING".into(),
                status_tone: Tone::Warning,
                branch: Some("relay/term".into()),
                changed: 3,
                review: 2,
            },
        ))
}

fn tab_samples(state: &GalleryState, host: &Entity<GalleryApp>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            Tabs::new(
                "demo-tabs",
                vec![
                    Tab::new("files", "Files").icon(IconName::FileText),
                    Tab::new("diff", "Diff").icon(IconName::FileDiff).count(12),
                    Tab::new("review", "Review")
                        .icon(IconName::MessageSquareText)
                        .count(3),
                ],
            )
            .active(state.seg_tab)
            .on_select({
                let host = host.clone();
                move |key, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.gallery.seg_tab = key;
                        cx.notify();
                    });
                }
            }),
        )
        .child(
            strip().child(
                SegmentedControl::new(
                    "seg-demo",
                    vec![
                        Segment::new("files", "Files"),
                        Segment::new("diff", "Diff"),
                        Segment::new("review", "Review"),
                    ],
                )
                .active(state.seg_tab)
                .on_select({
                    let host = host.clone();
                    move |key, _window, cx| {
                        host.update(cx, |this, cx| {
                            this.gallery.seg_tab = key;
                            cx.notify();
                        });
                    }
                }),
            ),
        )
        .child(Divider::horizontal())
        .child(
            EmptyState::new("No tasks yet", "Create a task to launch an agent.")
                .icon(IconName::ListChecks),
        )
}
