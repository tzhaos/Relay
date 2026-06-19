//! Structural components: [`Tabs`] (an underline tab bar), [`ListSection`] (a
//! titled group container), and [`KeyValue`] (a metadata row).
//!
//! These differ from the pill-style [`crate::SegmentedControl`]: `Tabs` is the
//! larger underline-style switcher Orca uses for primary panes, while the
//! segmented control is the compact rounded group for in-pane filters.

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, space},
};

// ---------------------------------------------------------------------------
// Tabs — an underline tab bar.
// ---------------------------------------------------------------------------

/// One tab in a [`Tabs`] bar. `key` is the stable string the host maps back to
/// its own enum in `on_select`.
pub struct Tab {
    key: &'static str,
    label: &'static str,
    icon: Option<IconName>,
    count: Option<usize>,
}

impl Tab {
    pub fn new(key: &'static str, label: &'static str) -> Self {
        Self {
            key,
            label,
            icon: None,
            count: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }
}

type SelectHandler = Box<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;

/// An underline tab bar. The active tab gets an accent underline and high-contrast
/// text; inactive tabs are muted. The bar carries a 1px bottom border so it reads
/// as a chrome strip even with few tabs.
#[derive(IntoElement)]
pub struct Tabs {
    id: ElementId,
    tabs: Vec<Tab>,
    active: &'static str,
    on_select: Option<SelectHandler>,
}

impl Tabs {
    pub fn new(id: impl Into<ElementId>, tabs: Vec<Tab>) -> Self {
        Self {
            id: id.into(),
            tabs,
            active: "",
            on_select: None,
        }
    }

    pub fn active(mut self, active: &'static str) -> Self {
        self.active = active;
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let handler = self.on_select.map(std::rc::Rc::new);
        let active = self.active;

        let mut row = div()
            .id(self.id)
            .h(px(36.0))
            .w_full()
            .flex()
            .items_center()
            .gap_1()
            .border_b_1()
            .border_color(theme.border);

        for (index, tab) in self.tabs.into_iter().enumerate() {
            let is_active = tab.key == active;
            let key = tab.key;
            let handler = handler.clone();
            let (fg, underline) = if is_active {
                (theme.text, theme.accent)
            } else {
                (theme.text_muted, gpui::transparent_black())
            };

            let cell = div()
                .id(("tab", index))
                .px_2()
                .flex()
                .items_center()
                .gap_1()
                .text_sm()
                .font_weight(if is_active {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(fg)
                // A 2px underline that sits flush on the bar's bottom border.
                .border_b_2()
                .border_color(underline)
                .when(!is_active, |this| {
                    this.cursor_pointer()
                        .hover(move |s| s.text_color(theme.text_secondary))
                })
                .when_some(tab.icon, |this, icon| {
                    let c = if is_active {
                        theme.accent
                    } else {
                        theme.text_muted
                    };
                    this.child(Icon::new(icon).size(IconSize::Small).color(c))
                })
                .child(tab.label)
                .when_some(tab.count, |this, count| {
                    this.child(
                        div()
                            .text_size(px(11.0))
                            .text_color(theme.text_muted)
                            .child(format!("({count})")),
                    )
                })
                .when_some(handler.filter(|_| !is_active), |this, handler| {
                    this.on_click(move |_: &ClickEvent, window, cx| {
                        handler(key, window, cx);
                        cx.stop_propagation();
                    })
                });
            row = row.child(cell);
        }
        row
    }
}

// ---------------------------------------------------------------------------
// ListSection — a titled group container.
// ---------------------------------------------------------------------------

/// A titled section: an uppercase caption with an optional trailing action, above
/// a content slot. Matches Orca's "PROJECTS" / "TASKS" rail groupings.
#[derive(IntoElement)]
pub struct ListSection {
    title: String,
    count: Option<usize>,
    trailing: Option<gpui::AnyElement>,
    body: Option<gpui::AnyElement>,
}

impl ListSection {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            count: None,
            trailing: None,
            body: None,
        }
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    pub fn child(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }
}

impl RenderOnce for ListSection {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .flex()
            .flex_col()
            .child(
                // Header row.
                div()
                    .h(px(space::ROW_SM))
                    .px_2()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text_muted)
                                    .child(self.title.to_uppercase()),
                            )
                            .when_some(self.count, |this, count| {
                                this.child(
                                    div()
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.text_muted)
                                        .child(count.to_string()),
                                )
                            }),
                    )
                    .when_some(self.trailing, |this, trailing| this.child(trailing)),
            )
            .when_some(self.body, |this, body| this.child(body))
    }
}

// ---------------------------------------------------------------------------
// KeyValue — a metadata row.
// ---------------------------------------------------------------------------

/// A compact key/value metadata row: a muted label on the left, a value on the
/// right. Used in task detail / context footers.
#[derive(IntoElement)]
pub struct KeyValue {
    label: String,
    value: String,
    icon: Option<IconName>,
}

impl KeyValue {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            icon: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl RenderOnce for KeyValue {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .h(px(space::ROW_SM))
            .flex()
            .items_center()
            .gap_2()
            .text_sm()
            .when_some(self.icon, |this, icon| {
                this.child(
                    Icon::new(icon)
                        .size(IconSize::Small)
                        .color(theme.text_muted),
                )
            })
            .child(
                div()
                    .flex_shrink_0()
                    .text_color(theme.text_muted)
                    .child(self.label),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_color(theme.text_secondary)
                    .child(self.value),
            )
    }
}
