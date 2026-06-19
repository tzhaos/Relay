use gpui::{
    App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, space},
    tone::Tone,
};

/// One compact key/value item in a [`StatusBar`].
pub struct StatusItem {
    label: String,
    value: String,
    tone: Tone,
    icon: Option<IconName>,
}

impl StatusItem {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            tone: Tone::Secondary,
            icon: None,
        }
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// A bottom status strip with left and right item groups.
#[derive(IntoElement)]
pub struct StatusBar {
    left: Vec<StatusItem>,
    right: Vec<StatusItem>,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            left: Vec::new(),
            right: Vec::new(),
        }
    }

    pub fn left(mut self, item: StatusItem) -> Self {
        self.left.push(item);
        self
    }

    pub fn right(mut self, item: StatusItem) -> Self {
        self.right.push(item);
        self
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for StatusBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .h(px(space::STATUS_BAR))
            .flex_shrink_0()
            .px_3()
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.chrome)
            .flex()
            .items_center()
            .justify_between()
            .text_xs()
            .child(status_group(theme, self.left))
            .child(status_group(theme, self.right))
    }
}

fn status_group(theme: crate::Theme, items: Vec<StatusItem>) -> gpui::Div {
    div()
        .min_w_0()
        .flex()
        .items_center()
        .gap_3()
        .children(items.into_iter().map(move |item| status_item(theme, item)))
}

fn status_item(theme: crate::Theme, item: StatusItem) -> gpui::Div {
    div()
        .min_w_0()
        .flex()
        .items_center()
        .gap_1()
        .when_some(item.icon, |this, icon| {
            this.child(
                Icon::new(icon)
                    .size(IconSize::XSmall)
                    .color(item.tone.fg(&theme)),
            )
        })
        .child(
            div()
                .flex_shrink_0()
                .text_color(theme.text_muted)
                .child(item.label),
        )
        .child(
            div()
                .min_w_0()
                .truncate()
                .text_color(item.tone.fg(&theme))
                .child(item.value),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_bar_starts_empty() {
        let bar = StatusBar::new();
        assert!(bar.left.is_empty());
    }
}
