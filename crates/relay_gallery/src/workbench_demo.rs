//! The Orca three-column workbench shell — fully interactive, assembled from
//! `relay_ui_kit` components. This is the "approval sample": it shows the target
//! layout, density, and behavior before the real `relay_ui` is reworked onto the
//! kit.
//!
//! Layout (per `DESIGN.md`): left rail (nav + project tree + tasks) / center
//! terminal / right context (tabs + file tree). Everything is live: click a task
//! to activate it, switch Files/Diff/Review, switch Terminal/Preview, type in the
//! filter to narrow the file tree, open a task's context menu.

use gpui::{
    Context, Entity, FocusHandle, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use relay_ui_kit::{
    ActiveTheme, Badge, Divider, IconButton, NavRow, PanelHeader, Segment, SegmentedControl,
    StatusDot, Tab, Tabs, TaskRow, TaskRowData, TextInput, TextInputAction, TextInputState, Tone,
    TreeRow,
    icon::IconName,
    theme::{self, space},
};

use crate::GalleryApp;

/// Interactive state for the Workbench page.
pub struct WorkbenchState {
    pub active_task: usize,
    pub context_tab: &'static str,
    pub route: &'static str,
    pub filter: TextInputState,
    pub filter_focus: FocusHandle,
    pub menu_task: Option<usize>,
}

impl WorkbenchState {
    pub fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            active_task: 0,
            context_tab: "files",
            route: "terminal",
            filter: TextInputState::new(),
            filter_focus: cx.focus_handle(),
            menu_task: None,
        }
    }
}

struct DemoTask {
    title: &'static str,
    status: &'static str,
    tone: Tone,
    branch: &'static str,
    changed: usize,
    review: usize,
}

fn demo_tasks() -> Vec<DemoTask> {
    vec![
        DemoTask {
            title: "Rework UI onto kit",
            status: "Running",
            tone: Tone::Accent,
            branch: "ui/kit-migration",
            changed: 12,
            review: 0,
        },
        DemoTask {
            title: "Persist window state",
            status: "Review",
            tone: Tone::Warning,
            branch: "feat/window-state",
            changed: 4,
            review: 3,
        },
        DemoTask {
            title: "Terminal scrollback buffer",
            status: "Idle",
            tone: Tone::Muted,
            branch: "fix/scrollback",
            changed: 0,
            review: 0,
        },
        DemoTask {
            title: "Agent retry on timeout",
            status: "Failed",
            tone: Tone::Danger,
            branch: "fix/agent-retry",
            changed: 7,
            review: 1,
        },
    ]
}

/// Files shown in the right-pane tree (path depth, icon, name). The filter
/// narrows this list by substring.
fn demo_files() -> Vec<(usize, IconName, &'static str, bool)> {
    vec![
        (0, IconName::Folder, "crates", true),
        (1, IconName::Folder, "relay_ui_kit", true),
        (2, IconName::FileText, "theme.rs", false),
        (2, IconName::FileText, "icon.rs", false),
        (2, IconName::FileText, "button.rs", false),
        (2, IconName::FileText, "input.rs", false),
        (2, IconName::FileText, "overlay.rs", false),
        (2, IconName::FileText, "row.rs", false),
        (1, IconName::Folder, "relay_gallery", true),
        (2, IconName::FileText, "main.rs", false),
        (2, IconName::FileText, "gallery.rs", false),
        (2, IconName::FileText, "workbench_demo.rs", false),
        (0, IconName::FileText, "Cargo.toml", false),
    ]
}

pub fn render(
    state: &WorkbenchState,
    host: &Entity<GalleryApp>,
    window: &Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    let theme = *cx.theme();
    div()
        .size_full()
        .flex()
        .child(left_rail(state, host, cx))
        .child(Divider::vertical())
        .child(center_terminal(state, host, cx))
        .child(Divider::vertical())
        .child(right_context(state, host, window, cx))
        .bg(theme.app_bg)
}

// ---------------------------------------------------------------------------
// Left rail — nav + project tree + tasks.
// ---------------------------------------------------------------------------

fn left_rail(
    state: &WorkbenchState,
    host: &Entity<GalleryApp>,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    let theme = *cx.theme();
    div()
        .w(px(space::RAIL_WIDTH))
        .flex_shrink_0()
        .h_full()
        .flex()
        .flex_col()
        .bg(theme.chrome)
        // Nav section.
        .child(
            div()
                .px_2()
                .pt_2()
                .pb_1()
                .flex()
                .flex_col()
                .gap(px(1.0))
                .child(
                    NavRow::new("nav-tasks", IconName::ListChecks, "Tasks")
                        .selected(true)
                        .count(4),
                )
                .child(NavRow::new("nav-auto", IconName::Zap, "Automation"))
                .child(NavRow::new("nav-search", IconName::Search, "Search")),
        )
        .child(Divider::horizontal())
        // Project + worktree tree.
        .child(
            div()
                .px_2()
                .py_2()
                .flex()
                .flex_col()
                .gap(px(1.0))
                .child(section_label("PROJECT", cx))
                .child(TreeRow::new("proj-relay", IconName::Folder, "relay").expandable(true))
                .child(
                    TreeRow::new("wt-main", IconName::GitBranch, "main")
                        .depth(1)
                        .selected(true),
                )
                .child(TreeRow::new("wt-kit", IconName::GitBranch, "ui/kit-migration").depth(1)),
        )
        .child(Divider::horizontal())
        // Tasks header.
        .child(
            div()
                .px_3()
                .h(px(space::PANE_HEADER))
                .flex()
                .items_center()
                .justify_between()
                .child(section_label("TASKS", cx))
                .child(IconButton::new("add-task", IconName::Plus)),
        )
        // Task rows — clicking activates.
        .child(
            div()
                .flex_1()
                .min_h_0()
                .px_2()
                .flex()
                .flex_col()
                .gap_1()
                .children(demo_tasks().into_iter().enumerate().map(|(i, task)| {
                    let host = host.clone();
                    TaskRow::new(
                        ("task", i),
                        TaskRowData {
                            title: task.title.into(),
                            status_label: task.status.into(),
                            status_tone: task.tone,
                            branch: Some(task.branch.into()),
                            changed: task.changed,
                            review: task.review,
                        },
                    )
                    .selected(i == state.active_task)
                    .on_click(move |_event, _window, cx| {
                        host.update(cx, |this, cx| {
                            this.workbench.active_task = i;
                            cx.notify();
                        });
                    })
                    .into_any_element()
                })),
        )
}

fn section_label(label: &'static str, cx: &mut Context<GalleryApp>) -> impl IntoElement {
    let theme = *cx.theme();
    div()
        .text_size(px(11.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme.text_muted)
        .child(label)
}

// ---------------------------------------------------------------------------
// Center — terminal / preview, with the active task's branch in the header.
// ---------------------------------------------------------------------------

fn center_terminal(
    state: &WorkbenchState,
    host: &Entity<GalleryApp>,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    let theme = *cx.theme();
    let tasks = demo_tasks();
    let active = &tasks[state.active_task.min(tasks.len() - 1)];
    let route = state.route;

    div()
        .flex_1()
        .min_w_0()
        .h_full()
        .flex()
        .flex_col()
        .child(
            PanelHeader::new(active.branch.to_string())
                .icon(IconName::Terminal)
                .trailing(
                    SegmentedControl::new(
                        "route",
                        vec![
                            Segment::new("terminal", "Terminal"),
                            Segment::new("preview", "Preview"),
                        ],
                    )
                    .active(route)
                    .on_select({
                        let host = host.clone();
                        move |key, _window, cx| {
                            host.update(cx, |this, cx| {
                                this.workbench.route = key;
                                cx.notify();
                            });
                        }
                    }),
                ),
        )
        .when(route == "terminal", |this| {
            this.child(terminal_body(active, theme))
        })
        .when(route == "preview", |this| {
            this.child(preview_body(active, theme))
        })
}

fn terminal_body(active: &DemoTask, theme: relay_ui_kit::Theme) -> impl IntoElement {
    div()
        .flex_1()
        .min_h_0()
        .bg(theme.terminal_bg)
        .p_3()
        .flex()
        .flex_col()
        .gap_1()
        .font_family(theme::mono_family())
        .text_size(px(13.0))
        .child(term_line(
            theme.terminal_dim,
            format!("relay@kit ~/relay ({})", active.branch),
        ))
        .child(term_line(
            theme.terminal_text,
            "$ cargo run -p relay_app".into(),
        ))
        .child(term_line(
            theme.terminal_dim,
            "   Compiling relay_ui_kit v0.1.0".into(),
        ))
        .child(term_line(
            theme.terminal_dim,
            "   Compiling relay_ui v0.1.0".into(),
        ))
        .child(term_line(
            theme.terminal_text,
            "    Finished dev [unoptimized] in 3.41s".into(),
        ))
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .text_color(theme.terminal_dim)
                        .child("relay@kit ~/relay $"),
                )
                .child(div().w(px(8.0)).h(px(15.0)).bg(theme.terminal_text)),
        )
}

fn preview_body(active: &DemoTask, theme: relay_ui_kit::Theme) -> impl IntoElement {
    div()
        .flex_1()
        .min_h_0()
        .bg(theme.panel)
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text_secondary)
                .child(format!("Preview for {}", active.branch)),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child("No dev server attached. Start one to see a live preview."),
        )
}

fn term_line(color: gpui::Hsla, text: String) -> impl IntoElement {
    div().text_color(color).child(text)
}

// ---------------------------------------------------------------------------
// Right context — tabs + filter + file tree.
// ---------------------------------------------------------------------------

fn right_context(
    state: &WorkbenchState,
    host: &Entity<GalleryApp>,
    window: &Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    let theme = *cx.theme();
    let tab = state.context_tab;
    let filter_focused = state.filter_focus.is_focused(window);
    let filter_text = state.filter.value().to_string();

    div()
        .w(px(space::CONTEXT_WIDTH))
        .flex_shrink_0()
        .h_full()
        .flex()
        .flex_col()
        .bg(theme.chrome)
        .child(
            PanelHeader::new("relay").icon(IconName::FileText).trailing(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(IconButton::new("ctx-refresh", IconName::RefreshCw))
                    .child(IconButton::new("ctx-more", IconName::Ellipsis)),
            ),
        )
        // Tabs: Files / Diff / Review.
        .child(
            div().px_2().pt_1().child(
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
        // Tab body.
        .when(tab == "files", |this| {
            this.child(files_tab(state, host, filter_focused, &filter_text))
        })
        .when(tab == "diff", |this| this.child(diff_tab(theme)))
        .when(tab == "review", |this| this.child(review_tab(theme)))
}

fn files_tab(
    state: &WorkbenchState,
    host: &Entity<GalleryApp>,
    filter_focused: bool,
    filter_text: &str,
) -> impl IntoElement {
    let host_for_key = host.clone();
    let needle = filter_text.to_lowercase();
    let files = demo_files()
        .into_iter()
        .filter(|(_, _, name, _)| needle.is_empty() || name.to_lowercase().contains(&needle))
        .enumerate()
        .map(|(i, (depth, icon, name, expandable))| {
            let mut row = TreeRow::new(("file", i), icon, name).depth(depth);
            if expandable {
                row = row.expandable(true);
            }
            if name == "icon.rs" {
                row = row.selected(true);
            }
            row.into_any_element()
        });

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        // Filter field.
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
        // File tree.
        .child(
            div()
                .flex_1()
                .min_h_0()
                .px_2()
                .flex()
                .flex_col()
                .gap(px(1.0))
                .children(files),
        )
}

fn diff_tab(theme: relay_ui_kit::Theme) -> impl IntoElement {
    let hunk = |sign: &'static str, color, text: &'static str| {
        div()
            .flex()
            .gap_2()
            .font_family(theme::mono_family())
            .text_size(px(12.0))
            .child(
                div()
                    .w(px(12.0))
                    .flex_shrink_0()
                    .text_color(color)
                    .child(sign),
            )
            .child(div().text_color(theme.text_secondary).child(text))
    };
    div()
        .flex_1()
        .min_h_0()
        .id("ctx-file-tree")
        .overflow_y_scroll()
        .p_3()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text_muted)
                .child("crates/relay_ui_kit/src/theme.rs"),
        )
        .child(hunk("+", theme.accent, "pub accent: Hsla,"))
        .child(hunk("+", theme.accent, "pub on_accent: Hsla,"))
        .child(hunk("-", theme.danger, "pub legacy_accent: Hsla,"))
        .child(hunk(" ", theme.text_muted, "pub border: Hsla,"))
}

fn review_tab(theme: relay_ui_kit::Theme) -> impl IntoElement {
    div()
        .flex_1()
        .min_h_0()
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
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
                .child("theme.rs:42 — consider documenting the on_accent contrast target."),
        )
}
