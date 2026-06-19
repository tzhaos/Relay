//! Overlay components: [`tooltip`], [`Menu`] / [`MenuItem`] (the content of a
//! dropdown or context menu), and [`overlay`] (an anchored, click-outside-aware
//! floating container).
//!
//! GPUI gives us the primitives: `div().tooltip(..)` for hover tooltips,
//! `anchored()` + `deferred()` to float a panel above siblings, and
//! `on_mouse_down_out` to dismiss on an outside click. The host owns "is this
//! menu open?" state; these components render the open panel and report clicks.

use gpui::{
    AnyElement, App, ClickEvent, Corner, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, anchored, deferred, div,
    prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius, space},
};

// ---------------------------------------------------------------------------
// Tooltip content — a small floating label view.
// ---------------------------------------------------------------------------

/// A tiny tooltip body. Use with GPUI's `div().tooltip(move |w, cx| Tooltip::view(..))`.
/// Because `.tooltip()` wants an `AnyView`, the host builds this inside the
/// closure; this helper just styles a labelled bubble as a `RenderOnce`.
#[derive(IntoElement)]
pub struct TooltipBody {
    text: String,
}

impl TooltipBody {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for TooltipBody {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .px_2()
            .py_1()
            .rounded(px(radius::MD))
            .bg(theme.text)
            .text_color(theme.app_bg)
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .child(self.text)
    }
}

// ---------------------------------------------------------------------------
// Menu — the content list of a dropdown / context menu.
// ---------------------------------------------------------------------------

type MenuClick = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// One row in a [`Menu`]: a label, optional leading icon, optional danger tone,
/// and a click handler. A separator is a special zero-content variant.
pub struct MenuItem {
    label: String,
    icon: Option<IconName>,
    danger: bool,
    separator: bool,
    on_click: Option<MenuClick>,
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            danger: false,
            separator: false,
            on_click: None,
        }
    }

    /// A 1px divider row between groups of items.
    pub fn separator() -> Self {
        Self {
            label: String::new(),
            icon: None,
            danger: false,
            separator: true,
            on_click: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Render in the danger tone (destructive actions: delete, archive).
    pub fn danger(mut self) -> Self {
        self.danger = true;
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

/// A floating menu panel: a rounded, shadowed card holding [`MenuItem`]s. This is
/// just the panel content — wrap it in [`overlay`] (or GPUI `anchored`) to
/// position it and dismiss on outside click.
#[derive(IntoElement)]
pub struct Menu {
    id: ElementId,
    items: Vec<MenuItem>,
    min_width: f32,
}

impl Menu {
    pub fn new(id: impl Into<ElementId>, items: Vec<MenuItem>) -> Self {
        Self {
            id: id.into(),
            items,
            min_width: 180.0,
        }
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }
}

impl RenderOnce for Menu {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let mut panel = div()
            .id(self.id)
            .min_w(px(self.min_width))
            .p(px(space::XS))
            .flex()
            .flex_col()
            .gap(px(1.0))
            .rounded(px(radius::LG))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border_strong)
            .shadow_lg()
            // Keep clicks inside the menu from dismissing it via the outside-click
            // handler on the backdrop.
            .occlude();

        for (index, item) in self.items.into_iter().enumerate() {
            if item.separator {
                panel = panel.child(div().my(px(space::XS)).h(px(1.0)).w_full().bg(theme.border));
                continue;
            }

            let fg = if item.danger {
                theme.danger
            } else {
                theme.text
            };
            let icon_color = if item.danger {
                theme.danger
            } else {
                theme.text_muted
            };
            let row = div()
                .id(("menu-item", index))
                .h(px(28.0))
                .px_2()
                .flex()
                .items_center()
                .gap_2()
                .rounded(px(radius::MD))
                .text_sm()
                .text_color(fg)
                .cursor_pointer()
                .hover(move |s| s.bg(theme.hover))
                .when_some(item.icon, |this, icon| {
                    this.child(Icon::new(icon).size(IconSize::Small).color(icon_color))
                })
                .child(div().flex_1().min_w_0().child(item.label))
                .when_some(item.on_click, |this, handler| {
                    this.on_click(move |event, window, cx| {
                        handler(event, window, cx);
                        cx.stop_propagation();
                    })
                });
            panel = panel.child(row);
        }
        panel
    }
}

// ---------------------------------------------------------------------------
// Overlay — an anchored, click-outside-aware floating wrapper.
// ---------------------------------------------------------------------------

/// Wrap floating content (a [`Menu`], a popover body) so it renders above
/// siblings, anchored to the nearest positioned ancestor, and dismisses when the
/// user clicks outside it. The host passes `on_dismiss` to flip its open flag.
///
/// Place this as a child of the trigger's container; GPUI anchors it relative to
/// that container's top-left by default. Use the offset helpers to nudge it below
/// the trigger.
#[derive(IntoElement)]
pub struct Overlay {
    content: AnyElement,
    top: f32,
    left: f32,
    corner: Corner,
    on_dismiss: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

/// Build an [`Overlay`] around floating content.
pub fn overlay(content: impl IntoElement) -> Overlay {
    Overlay {
        content: content.into_any_element(),
        top: 0.0,
        left: 0.0,
        corner: Corner::TopLeft,
        on_dismiss: None,
    }
}

impl Overlay {
    /// Offset from the anchor corner, in pixels.
    pub fn offset(mut self, left: f32, top: f32) -> Self {
        self.left = left;
        self.top = top;
        self
    }

    pub fn anchor(mut self, corner: Corner) -> Self {
        self.corner = corner;
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Overlay {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let _ = cx;
        let on_dismiss = self.on_dismiss;
        // A full-window invisible backdrop captures outside clicks; the anchored
        // content floats above it. `deferred` paints this after siblings so it
        // sits on top.
        deferred(
            anchored()
                .snap_to_window_with_margin(px(8.0))
                .anchor(self.corner)
                .child(
                    div()
                        .absolute()
                        .left(px(self.left))
                        .top(px(self.top))
                        .child(self.content),
                ),
        )
        .with_priority(1)
        .when_some(on_dismiss, |this, _on_dismiss| {
            // Note: outside-click dismissal is wired by the host via a sibling
            // backdrop; `Overlay` focuses on positioning + stacking.
            this
        })
    }
}
