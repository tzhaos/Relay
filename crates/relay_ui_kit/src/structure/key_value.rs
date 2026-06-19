use gpui::{
    App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, space},
};

/// A compact key/value metadata row.
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
