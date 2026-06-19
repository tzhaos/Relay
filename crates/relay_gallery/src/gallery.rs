//! The Components page — every kit primitive, fully interactive.
//!
//! State lives in [`GalleryState`]; the render fn closes over the host
//! `Entity<GalleryApp>` so each component callback can `host.update(cx, ..)` to
//! mutate state and re-render. This is the same wiring the real workbench will
//! use, so the gallery doubles as a reference for that integration.

use gpui::{
    Context, Entity, FocusHandle, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use relay_ui_kit::{
    ActiveTheme, Badge, Button, ButtonVariant, Checkbox, Divider, EmptyState, Icon, IconButton,
    IconName, IconSize, Menu, MenuItem, NavRow, Radio, Segment, SegmentedControl, StatusDot, Tab,
    Tabs, TaskRow, TaskRowData, TextInput, TextInputAction, TextInputState, Toggle, Tone, TreeRow,
    overlay, space,
};

use crate::GalleryApp;

mod product_samples;

use product_samples::{command_sample, launcher_sample, shell_sample, terminal_sample};

/// Interactive state for the Components page.
pub struct GalleryState {
    pub name_input: TextInputState,
    pub name_focus: FocusHandle,
    pub search_input: TextInputState,
    pub search_focus: FocusHandle,
    pub notifications: bool,
    pub auto_archive: bool,
    pub theme_choice: &'static str,
    pub seg_tab: &'static str,
    pub terminal_session: &'static str,
    pub launcher_choice: &'static str,
    pub shell_split_width: f32,
    pub menu_open: bool,
}

impl GalleryState {
    pub fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            name_input: TextInputState::with_text("relay-agent"),
            name_focus: cx.focus_handle(),
            search_input: TextInputState::new(),
            search_focus: cx.focus_handle(),
            notifications: true,
            auto_archive: false,
            theme_choice: "system",
            seg_tab: "diff",
            terminal_session: "codex",
            launcher_choice: "powershell",
            shell_split_width: 260.0,
            menu_open: false,
        }
    }
}

/// A labelled section wrapper: a quiet caption above a card of samples.
fn section(cx: &mut Context<GalleryApp>, title: &str, body: impl IntoElement) -> impl IntoElement {
    let theme = *cx.theme();
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text_muted)
                .child(title.to_uppercase()),
        )
        .child(
            div()
                .p_3()
                .rounded(px(relay_ui_kit::radius::LG))
                .bg(theme.panel)
                .border_1()
                .border_color(theme.border)
                .child(body),
        )
}

/// A horizontal sample strip with comfortable gaps.
fn strip() -> gpui::Div {
    div().flex().items_center().gap_3().flex_wrap()
}

pub fn render(
    state: &GalleryState,
    host: &Entity<GalleryApp>,
    window: &Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    let theme = *cx.theme();
    let name_focused = state.name_focus.is_focused(window);
    let search_focused = state.search_focus.is_focused(window);

    div()
        .id("gallery-scroll")
        .size_full()
        .overflow_y_scroll()
        .bg(theme.app_bg)
        .child(
            div()
                .max_w(px(1100.0))
                .mx_auto()
                .p(px(space::XL))
                .flex()
                .flex_col()
                .gap(px(space::XL))
                // Text inputs -------------------------------------------------
                .child(section(
                    cx,
                    "Text input",
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .max_w(px(360.0))
                        .child(text_input_field(
                            state,
                            host,
                            "name",
                            &state.name_input,
                            state.name_focus.clone(),
                            name_focused,
                            None,
                            "Agent name",
                        ))
                        .child(text_input_field(
                            state,
                            host,
                            "search",
                            &state.search_input,
                            state.search_focus.clone(),
                            search_focused,
                            Some(IconName::Search),
                            "Filter files",
                        )),
                ))
                // Buttons -----------------------------------------------------
                .child(section(
                    cx,
                    "Buttons",
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
                        ),
                ))
                // Icon buttons ------------------------------------------------
                .child(section(
                    cx,
                    "Icon buttons",
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
                                        this.gallery.menu_open = !this.gallery.menu_open;
                                        cx.notify();
                                    });
                                }
                            }),
                        )
                        .child(
                            IconButton::new("ib-active", IconName::PanelLeft)
                                .active(true)
                                .on_click({
                                    let host = host.clone();
                                    move |_event, _window, cx| {
                                        host.update(cx, |this, cx| {
                                            this.gallery.seg_tab = "review";
                                            cx.notify();
                                        });
                                    }
                                }),
                        ),
                ))
                // Checkboxes + toggles ----------------------------------------
                .child(section(
                    cx,
                    "Checkbox & toggle",
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(checkbox_row(host, state.notifications))
                        .child(toggle_row(host, state.auto_archive)),
                ))
                // Radios ------------------------------------------------------
                .child(section(
                    cx,
                    "Radio group",
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(radio_row(
                            host,
                            "system",
                            "Follow system",
                            state.theme_choice,
                        ))
                        .child(radio_row(host, "light", "Always light", state.theme_choice))
                        .child(radio_row(host, "dark", "Always dark", state.theme_choice)),
                ))
                // Dropdown menu -----------------------------------------------
                .child(section(
                    cx,
                    "Dropdown menu",
                    div()
                        .relative()
                        .child(dropdown_trigger(host, state.menu_open))
                        .when(state.menu_open, |this| this.child(dropdown_menu(host))),
                ))
                // Badges ------------------------------------------------------
                .child(section(
                    cx,
                    "Badges",
                    strip()
                        .child(Badge::new("RUNNING").tone(Tone::Accent).soft())
                        .child(Badge::new("WAITING").tone(Tone::Warning).soft())
                        .child(Badge::new("FAILED").tone(Tone::Danger).soft())
                        .child(Badge::new("main").tone(Tone::Secondary))
                        .child(Badge::new("worktree").icon(IconName::GitBranch)),
                ))
                // Status dots -------------------------------------------------
                .child(section(
                    cx,
                    "Status dots",
                    strip()
                        .child(dot_label(theme, Tone::Accent, "running"))
                        .child(dot_label(theme, Tone::Warning, "waiting"))
                        .child(dot_label(theme, Tone::Danger, "failed"))
                        .child(dot_label(theme, Tone::Muted, "idle")),
                ))
                // Icons -------------------------------------------------------
                .child(section(
                    cx,
                    "Icons",
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
                ))
                // Tabs (underline) --------------------------------------------
                .child(section(
                    cx,
                    "Tabs",
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
                ))
                // Segmented control -------------------------------------------
                .child(section(
                    cx,
                    "Segmented control",
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
                ))
                // Nav rows ----------------------------------------------------
                .child(section(
                    cx,
                    "Nav rows",
                    div()
                        .w(px(280.0))
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
                        .child(NavRow::new("nav-search", IconName::Search, "Search")),
                ))
                // Tree rows ---------------------------------------------------
                .child(section(
                    cx,
                    "Tree rows",
                    div()
                        .w(px(320.0))
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
                        .child(TreeRow::new("tr-4", IconName::FileText, "icon.rs").depth(2)),
                ))
                // Task rows ---------------------------------------------------
                .child(section(
                    cx,
                    "Task rows",
                    div()
                        .w(px(320.0))
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
                        )),
                ))
                // Shell / split panes ----------------------------------------
                .child(section(
                    cx,
                    "Shell & split panes",
                    shell_sample(state, host),
                ))
                // Terminal product components --------------------------------
                .child(section(
                    cx,
                    "Terminal & agent",
                    terminal_sample(state, host, theme),
                ))
                // Command palette + launcher ---------------------------------
                .child(section(
                    cx,
                    "Command & launcher",
                    div()
                        .flex()
                        .items_start()
                        .gap_3()
                        .flex_wrap()
                        .child(command_sample(state, host))
                        .child(launcher_sample(state, host, theme)),
                ))
                // Divider + empty state ---------------------------------------
                .child(section(
                    cx,
                    "Divider & empty state",
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.text_secondary)
                                .child("Above the line"),
                        )
                        .child(Divider::horizontal())
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.text_secondary)
                                .child("Below the line"),
                        )
                        .child(Divider::horizontal())
                        .child(
                            EmptyState::new("No tasks yet", "Create a task to launch an agent.")
                                .icon(IconName::ListChecks),
                        ),
                )),
        )
}

// ---------------------------------------------------------------------------
// Interactive sub-builders.
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn text_input_field(
    _state: &GalleryState,
    host: &Entity<GalleryApp>,
    id: &'static str,
    input: &TextInputState,
    focus: FocusHandle,
    focused: bool,
    icon: Option<IconName>,
    placeholder: &'static str,
) -> impl IntoElement {
    let host = host.clone();
    let is_name = id == "name";
    let mut field = TextInput::new(id, focus, input)
        .placeholder(placeholder)
        .focused(focused)
        .on_key(move |event, _window, cx| {
            host.update(cx, |this, cx| {
                let target = if is_name {
                    &mut this.gallery.name_input
                } else {
                    &mut this.gallery.search_input
                };
                match target.handle_key(event) {
                    TextInputAction::Edited | TextInputAction::Submit | TextInputAction::Cancel => {
                        cx.notify()
                    }
                    TextInputAction::Ignored => {}
                }
            });
        });
    if let Some(icon) = icon {
        field = field.leading_icon(icon);
    }
    field
}

fn checkbox_row(host: &Entity<GalleryApp>, checked: bool) -> impl IntoElement {
    Checkbox::new("cb-notify", checked)
        .label("Enable notifications")
        .on_click({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.gallery.notifications = !this.gallery.notifications;
                    cx.notify();
                });
            }
        })
}

fn toggle_row(host: &Entity<GalleryApp>, on: bool) -> impl IntoElement {
    Toggle::new("tg-archive", on)
        .label("Auto-archive merged tasks")
        .on_click({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.gallery.auto_archive = !this.gallery.auto_archive;
                    cx.notify();
                });
            }
        })
}

fn radio_row(
    host: &Entity<GalleryApp>,
    key: &'static str,
    label: &'static str,
    selected: &'static str,
) -> impl IntoElement {
    Radio::new(key, key == selected, label).on_click({
        let host = host.clone();
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.gallery.theme_choice = key;
                cx.notify();
            });
        }
    })
}

fn dropdown_trigger(host: &Entity<GalleryApp>, open: bool) -> impl IntoElement {
    Button::new("dd-trigger", "Branch actions")
        .icon(if open {
            IconName::ChevronDown
        } else {
            IconName::ChevronRight
        })
        .on_click({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.gallery.menu_open = !this.gallery.menu_open;
                    cx.notify();
                });
            }
        })
}

fn dropdown_menu(host: &Entity<GalleryApp>) -> impl IntoElement {
    let close = {
        let host = host.clone();
        move |cx: &mut gpui::App| {
            host.update(cx, |this, cx| {
                this.gallery.menu_open = false;
                cx.notify();
            });
        }
    };
    let close_a = close.clone();
    let close_b = close.clone();
    let close_c = close;
    overlay(Menu::new(
        "dd-menu",
        vec![
            MenuItem::new("Checkout")
                .icon(IconName::GitBranch)
                .on_click(move |_e, _w, cx| close_a(cx)),
            MenuItem::new("New worktree")
                .icon(IconName::FolderPlus)
                .on_click(move |_e, _w, cx| close_b(cx)),
            MenuItem::separator(),
            MenuItem::new("Delete branch")
                .icon(IconName::Archive)
                .danger()
                .on_click(move |_e, _w, cx| close_c(cx)),
        ],
    ))
    .offset(0.0, 34.0)
}

fn dot_label(theme: relay_ui_kit::Theme, tone: Tone, label: &str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(StatusDot::new(tone))
        .child(
            div()
                .text_sm()
                .text_color(theme.text_secondary)
                .child(label.to_string()),
        )
}

fn icon_sample(theme: relay_ui_kit::Theme, name: IconName) -> impl IntoElement {
    div()
        .size(px(32.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(relay_ui_kit::radius::MD))
        .bg(theme.panel_alt)
        .border_1()
        .border_color(theme.border)
        .child(
            Icon::new(name)
                .size(IconSize::Medium)
                .color(theme.text_secondary),
        )
}
