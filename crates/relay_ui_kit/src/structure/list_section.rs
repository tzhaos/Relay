use gpui::{
    AnyElement, App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, space};

/// A titled section with an optional trailing action and content slot.
#[derive(IntoElement)]
pub struct ListSection {
    title: String,
    count: Option<usize>,
    trailing: Option<AnyElement>,
    body: Option<AnyElement>,
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
