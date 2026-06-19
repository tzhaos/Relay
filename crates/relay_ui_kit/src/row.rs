//! List rows: [`NavRow`], [`TreeRow`], and [`TaskRow`].
//!
//! These are the Orca left-rail building blocks — a top-level navigation entry,
//! a file/worktree tree node, and a multi-line task card. All three are
//! `RenderOnce` builders with a generic click callback and a selected/active
//! state, so the gallery and the real workbench wire them to different handlers.

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    display::StatusDot,
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius, space},
    tone::Tone,
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

// ---------------------------------------------------------------------------
// NavRow — a top-level navigation entry (任务 / 自动化 / 搜索).
// ---------------------------------------------------------------------------

/// A top-level navigation row: leading icon, label, optional trailing count
/// badge. Selection uses a rounded accent-tinted fill, matching Orca's left nav.
#[derive(IntoElement)]
pub struct NavRow {
    id: ElementId,
    icon: IconName,
    label: String,
    count: Option<usize>,
    selected: bool,
    on_click: Option<ClickHandler>,
}

impl NavRow {
    pub fn new(id: impl Into<ElementId>, icon: IconName, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            icon,
            label: label.into(),
            count: None,
            selected: false,
            on_click: None,
        }
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for NavRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let (fg, icon_color) = if self.selected {
            (theme.text, theme.accent)
        } else {
            (theme.text_secondary, theme.text_muted)
        };

        div()
            .id(self.id)
            .h(px(space::ROW_MD))
            .px_2()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .text_color(fg)
            .when(self.selected, |this| this.bg(theme.selection))
            .when(!self.selected, |this| {
                this.cursor_pointer().hover(move |s| s.bg(theme.hover))
            })
            .child(Icon::new(self.icon).size(IconSize::Small).color(icon_color))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .font_weight(if self.selected {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::MEDIUM
                    })
                    .child(self.label),
            )
            .when_some(self.count, |this, count| {
                this.child(
                    div()
                        .min_w(px(18.0))
                        .h(px(18.0))
                        .px_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(radius::SM))
                        .bg(theme.panel_alt)
                        .text_color(theme.text_muted)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(count.to_string()),
                )
            })
            .when_some(self.on_click, |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}

// ---------------------------------------------------------------------------
// TreeRow — a file/worktree tree node with indent + disclosure.
// ---------------------------------------------------------------------------

/// A file-tree node: optional disclosure chevron, leading icon, label, indented
/// by `depth`. Used for the right-pane file tree and the left-rail worktree tree.
#[derive(IntoElement)]
pub struct TreeRow {
    id: ElementId,
    icon: IconName,
    label: String,
    depth: usize,
    expandable: bool,
    expanded: bool,
    selected: bool,
    on_click: Option<ClickHandler>,
}

impl TreeRow {
    pub fn new(id: impl Into<ElementId>, icon: IconName, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            icon,
            label: label.into(),
            depth: 0,
            expandable: false,
            expanded: false,
            selected: false,
            on_click: None,
        }
    }

    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// Mark this node as a directory; `expanded` drives the chevron direction.
    pub fn expandable(mut self, expanded: bool) -> Self {
        self.expandable = true;
        self.expanded = expanded;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TreeRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = if self.selected {
            theme.text
        } else {
            theme.text_secondary
        };
        // Indent: 14px per level, plus a fixed slot for the disclosure chevron so
        // leaf and parent labels align.
        let indent = px(space::SM + self.depth as f32 * 14.0);
        let chevron = if self.expandable {
            Some(if self.expanded {
                IconName::ChevronDown
            } else {
                IconName::ChevronRight
            })
        } else {
            None
        };

        div()
            .id(self.id)
            .h(px(space::ROW_SM))
            .pr_2()
            .pl(indent)
            .flex()
            .items_center()
            .gap_1()
            .rounded(px(radius::SM))
            .text_color(fg)
            .when(self.selected, |this| this.bg(theme.selection))
            .when(!self.selected, |this| {
                this.cursor_pointer().hover(move |s| s.bg(theme.hover))
            })
            // Disclosure slot — fixed width so labels align whether or not a
            // chevron is present.
            .child(
                div()
                    .w(px(14.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .when_some(chevron, |this, chevron| {
                        this.child(
                            Icon::new(chevron)
                                .size(IconSize::XSmall)
                                .color(theme.text_muted),
                        )
                    }),
            )
            .child(
                Icon::new(self.icon)
                    .size(IconSize::Small)
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .child(self.label),
            )
            .when_some(self.on_click, |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}

// ---------------------------------------------------------------------------
// TaskRow — a multi-line task card for the left rail.
// ---------------------------------------------------------------------------

/// Metadata for one [`TaskRow`]. The row shows a status dot, title, a status
/// label, and a second metadata line (branch/path + changed/review counts).
pub struct TaskRowData {
    pub title: String,
    pub status_label: String,
    pub status_tone: Tone,
    pub branch: Option<String>,
    pub changed: usize,
    pub review: usize,
}

/// A task row: a fixed-height card showing status dot, title, status badge, and a
/// quiet metadata line. Height stays constant between active/inactive states so
/// the list never reflows (per the QA checklist).
#[derive(IntoElement)]
pub struct TaskRow {
    id: ElementId,
    data: TaskRowData,
    selected: bool,
    on_click: Option<ClickHandler>,
}

impl TaskRow {
    pub fn new(id: impl Into<ElementId>, data: TaskRowData) -> Self {
        Self {
            id: id.into(),
            data,
            selected: false,
            on_click: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TaskRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let data = self.data;

        // Second metadata line: branch + changed/review counts, joined quietly.
        let mut meta_parts: Vec<String> = Vec::new();
        if let Some(branch) = &data.branch {
            meta_parts.push(branch.clone());
        }
        if data.changed > 0 {
            meta_parts.push(format!("{}±", data.changed));
        }
        if data.review > 0 {
            meta_parts.push(format!("{} review", data.review));
        }
        let meta = meta_parts.join("  ·  ");

        div()
            .id(self.id)
            .h(px(space::TASK_ROW))
            .px_2()
            .py(px(space::SM))
            .flex()
            .flex_col()
            .justify_center()
            .gap_1()
            .rounded(px(radius::MD))
            .border_1()
            .when(self.selected, |this| {
                this.bg(theme.accent_bg).border_color(theme.accent_border)
            })
            .when(!self.selected, |this| {
                this.border_color(gpui::transparent_black())
                    .cursor_pointer()
                    .hover(move |s| s.bg(theme.hover))
            })
            // Line 1: status dot + title + status label.
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(StatusDot::new(data.status_tone))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(data.title),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(data.status_tone.fg(&theme))
                            .child(data.status_label),
                    ),
            )
            // Line 2: quiet metadata.
            .when(!meta.is_empty(), |this| {
                this.child(
                    div()
                        .pl(px(15.0))
                        .truncate()
                        .text_size(px(11.0))
                        .text_color(theme.text_muted)
                        .child(meta),
                )
            })
            .when_some(self.on_click, |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}
