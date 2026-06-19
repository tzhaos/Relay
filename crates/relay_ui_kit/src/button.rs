//! Button components: a labelled [`Button`] and a square [`IconButton`].
//!
//! Both are `RenderOnce` builders with a generic click callback
//! (`Fn(&ClickEvent, &mut Window, &mut App)`), so they carry no dependency on any
//! concrete view — the gallery and the real workbench wire the same component to
//! different handlers.

use gpui::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius},
};

/// Visual emphasis for a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Filled accent — the single primary action in a surface.
    Primary,
    /// Outlined neutral — standard actionable controls.
    Secondary,
    /// Borderless text — low-stakes secondary actions.
    Ghost,
}

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A compact labelled button with an optional leading icon.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: String,
    variant: ButtonVariant,
    icon: Option<IconName>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: ButtonVariant::Secondary,
            icon: None,
            disabled: false,
            on_click: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shorthand for the single primary action in a surface.
    pub fn primary(mut self) -> Self {
        self.variant = ButtonVariant::Primary;
        self
    }

    /// Shorthand for a borderless, low-stakes action.
    pub fn ghost(mut self) -> Self {
        self.variant = ButtonVariant::Ghost;
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
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

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        // (bg, border, fg, hover_bg, hover_border)
        let (bg, border, fg, hover_bg, hover_border) = match self.variant {
            ButtonVariant::Primary => (
                theme.accent,
                theme.accent,
                theme.on_accent,
                theme.accent,
                theme.accent,
            ),
            ButtonVariant::Secondary => (
                theme.panel,
                theme.border_strong,
                theme.text,
                theme.hover,
                theme.border_strong,
            ),
            ButtonVariant::Ghost => (
                gpui::transparent_black(),
                gpui::transparent_black(),
                theme.text_secondary,
                theme.hover,
                gpui::transparent_black(),
            ),
        };

        let icon_color = fg;
        let handler = self.on_click;
        let disabled = self.disabled || handler.is_none();

        div()
            .id(self.id)
            .h(px(28.0))
            .px_2()
            .rounded(px(radius::MD))
            .border_1()
            .border_color(border)
            .bg(bg)
            .flex()
            .items_center()
            .justify_center()
            .gap_1()
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(fg)
            .when(disabled, |this| this.opacity(0.5))
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(hover_bg).border_color(hover_border))
            })
            .when_some(self.icon, |this, icon| {
                this.child(Icon::new(icon).size(IconSize::Small).color(icon_color))
            })
            .child(self.label)
            .when_some(handler.filter(|_| !disabled), |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}

/// A square, borderless icon button — toolbar actions, row affordances.
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: IconName,
    size: IconSize,
    active: bool,
    on_click: Option<ClickHandler>,
}

impl IconButton {
    pub fn new(id: impl Into<ElementId>, icon: IconName) -> Self {
        Self {
            id: id.into(),
            icon,
            size: IconSize::Small,
            active: false,
            on_click: None,
        }
    }

    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    /// Render in the active/selected state (accent foreground).
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
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

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = if self.active {
            theme.accent
        } else {
            theme.text_muted
        };
        let handler = self.on_click;

        div()
            .id(self.id)
            .size(px(26.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(radius::MD))
            .text_color(fg)
            .cursor_pointer()
            .hover(move |style| style.bg(theme.hover).text_color(theme.text))
            .child(Icon::new(self.icon).size(self.size).color(fg))
            .when_some(handler, |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}
