use gpui::{
    AnyElement, App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, space},
};

/// A 40px pane header with a leading title and optional trailing slot.
#[derive(IntoElement)]
pub struct PanelHeader {
    title: String,
    icon: Option<IconName>,
    trailing: Option<AnyElement>,
}

impl PanelHeader {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            trailing: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }
}

impl RenderOnce for PanelHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .h(px(space::PANE_HEADER))
            .px_3()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.chrome)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .min_w_0()
                    .when_some(self.icon, |this, icon| {
                        this.child(
                            Icon::new(icon)
                                .size(IconSize::Small)
                                .color(theme.text_secondary),
                        )
                    })
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(self.title),
                    ),
            )
            .when_some(self.trailing, |this, trailing| {
                this.child(div().flex().items_center().gap_1().child(trailing))
            })
    }
}
