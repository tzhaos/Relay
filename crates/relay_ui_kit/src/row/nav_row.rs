use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius, space},
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A top-level navigation row with a leading icon and optional count.
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
