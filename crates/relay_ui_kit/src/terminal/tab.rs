use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    display::StatusDot,
    theme::{ActiveTheme, radius},
    tone::Tone,
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A terminal tab in a session tab strip.
#[derive(IntoElement)]
pub struct TerminalTab {
    id: ElementId,
    label: String,
    active: bool,
    status: Tone,
    on_click: Option<ClickHandler>,
}

impl TerminalTab {
    pub fn new(id: impl Into<ElementId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            active: false,
            status: Tone::Muted,
            on_click: None,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn status(mut self, status: Tone) -> Self {
        self.status = status;
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

impl RenderOnce for TerminalTab {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let handler = self.on_click;

        div()
            .id(self.id)
            .h(px(30.0))
            .max_w(px(190.0))
            .px_2()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .border_1()
            .border_color(if self.active {
                theme.accent_border
            } else {
                theme.border
            })
            .bg(if self.active {
                theme.panel
            } else {
                theme.panel_alt
            })
            .text_color(if self.active {
                theme.text
            } else {
                theme.text_secondary
            })
            .when(!self.active, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(theme.hover))
            })
            .child(StatusDot::new(self.status).size(6.0))
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
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
