use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius},
    tone::Tone,
};

use super::TerminalStatusBadge;

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// One row in the terminal/session history panel.
#[derive(IntoElement)]
pub struct TerminalSessionRow {
    id: ElementId,
    title: String,
    subtitle: String,
    status: Tone,
    active: bool,
    on_click: Option<ClickHandler>,
}

impl TerminalSessionRow {
    pub fn new(
        id: impl Into<ElementId>,
        title: impl Into<String>,
        subtitle: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: subtitle.into(),
            status: Tone::Muted,
            active: false,
            on_click: None,
        }
    }

    pub fn status(mut self, status: Tone) -> Self {
        self.status = status;
        self
    }

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

impl RenderOnce for TerminalSessionRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let handler = self.on_click;

        div()
            .id(self.id)
            .min_h(px(48.0))
            .px_2()
            .py_1()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .border_1()
            .border_color(if self.active {
                theme.accent_border
            } else {
                gpui::transparent_black()
            })
            .bg(if self.active {
                theme.accent_bg
            } else {
                gpui::transparent_black()
            })
            .when(!self.active, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(theme.hover))
            })
            .child(
                Icon::new(IconName::Terminal)
                    .size(IconSize::Small)
                    .color(self.status.fg(&theme)),
            )
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(1.0))
                    .child(
                        div()
                            .truncate()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text)
                            .child(self.title),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(px(11.0))
                            .text_color(theme.text_muted)
                            .child(self.subtitle),
                    ),
            )
            .child(TerminalStatusBadge::new(self.status))
            .when_some(handler, |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}
