//! Selection controls: [`Checkbox`], [`Toggle`], and [`Radio`].
//!
//! All three are stateless `RenderOnce` builders — the host owns the boolean /
//! selected state and flips it in the click handler, then re-renders. They read
//! the active theme and carry a generic `on_click` callback, so they drop into
//! the gallery and the real workbench unchanged.

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius},
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

// ---------------------------------------------------------------------------
// Checkbox — a square check with an optional trailing label.
// ---------------------------------------------------------------------------

/// A labelled checkbox. The host owns `checked` and toggles it in `on_click`.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    checked: bool,
    label: Option<String>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        Self {
            id: id.into(),
            checked,
            label: None,
            disabled: false,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

impl RenderOnce for Checkbox {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let (box_bg, box_border) = if self.checked {
            (theme.accent, theme.accent)
        } else {
            (theme.panel, theme.border_strong)
        };
        let disabled = self.disabled;
        let handler = self.on_click.filter(|_| !disabled);

        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_2()
            .when(disabled, |this| this.opacity(0.5))
            .when(!disabled, |this| this.cursor_pointer())
            .child(
                div()
                    .size(px(16.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(radius::SM))
                    .border_1()
                    .border_color(box_border)
                    .bg(box_bg)
                    .when(self.checked, |this| {
                        this.child(
                            Icon::new(IconName::Check)
                                .size(IconSize::XSmall)
                                .color(theme.on_accent),
                        )
                    }),
            )
            .when_some(self.label, |this, label| {
                this.child(div().text_sm().text_color(theme.text).child(label))
            })
            .when_some(handler, |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}

// ---------------------------------------------------------------------------
// Toggle — a sliding on/off switch.
// ---------------------------------------------------------------------------

/// A sliding on/off switch. The host owns `on` and flips it in `on_click`.
#[derive(IntoElement)]
pub struct Toggle {
    id: ElementId,
    on: bool,
    label: Option<String>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl Toggle {
    pub fn new(id: impl Into<ElementId>, on: bool) -> Self {
        Self {
            id: id.into(),
            on,
            label: None,
            disabled: false,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

impl RenderOnce for Toggle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let track_bg = if self.on {
            theme.accent
        } else {
            theme.border_strong
        };
        let disabled = self.disabled;
        let handler = self.on_click.filter(|_| !disabled);

        // The knob sits at one of two ends; we express that as flex justify.
        let track = div()
            .w(px(32.0))
            .h(px(18.0))
            .flex_shrink_0()
            .rounded(px(9.0))
            .bg(track_bg)
            .p(px(2.0))
            .flex()
            .items_center()
            .when(self.on, |this| this.justify_end())
            .when(!self.on, |this| this.justify_start())
            .child(div().size(px(14.0)).rounded(px(7.0)).bg(theme.panel));

        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_2()
            .when(disabled, |this| this.opacity(0.5))
            .when(!disabled, |this| this.cursor_pointer())
            .child(track)
            .when_some(self.label, |this, label| {
                this.child(div().text_sm().text_color(theme.text).child(label))
            })
            .when_some(handler, |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}

// ---------------------------------------------------------------------------
// Radio — a circular single-choice indicator with a label.
// ---------------------------------------------------------------------------

/// A labelled radio option. The host renders a group of these and tracks which
/// is `selected`; clicking one sets the group's value.
#[derive(IntoElement)]
pub struct Radio {
    id: ElementId,
    selected: bool,
    label: String,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl Radio {
    pub fn new(id: impl Into<ElementId>, selected: bool, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            selected,
            label: label.into(),
            disabled: false,
            on_click: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

impl RenderOnce for Radio {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let border = if self.selected {
            theme.accent
        } else {
            theme.border_strong
        };
        let disabled = self.disabled;
        let handler = self.on_click.filter(|_| !disabled);

        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_2()
            .when(disabled, |this| this.opacity(0.5))
            .when(!disabled, |this| this.cursor_pointer())
            .child(
                div()
                    .size(px(16.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(border)
                    .bg(theme.panel)
                    .when(self.selected, |this| {
                        this.child(div().size(px(8.0)).rounded(px(4.0)).bg(theme.accent))
                    }),
            )
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text)
                    .child(self.label),
            )
            .when_some(handler, |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}
