use gpui::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder,
};

use crate::{display::Divider, theme::ActiveTheme};

/// Three-column workbench shell with optional title and status bars.
#[derive(IntoElement)]
pub struct AppShell {
    title_bar: Option<AnyElement>,
    left: Option<AnyElement>,
    center: AnyElement,
    right: Option<AnyElement>,
    status_bar: Option<AnyElement>,
}

impl AppShell {
    pub fn new(center: impl IntoElement) -> Self {
        Self {
            title_bar: None,
            left: None,
            center: center.into_any_element(),
            right: None,
            status_bar: None,
        }
    }

    pub fn title_bar(mut self, title_bar: impl IntoElement) -> Self {
        self.title_bar = Some(title_bar.into_any_element());
        self
    }

    pub fn left(mut self, left: impl IntoElement) -> Self {
        self.left = Some(left.into_any_element());
        self
    }

    pub fn right(mut self, right: impl IntoElement) -> Self {
        self.right = Some(right.into_any_element());
        self
    }

    pub fn status_bar(mut self, status_bar: impl IntoElement) -> Self {
        self.status_bar = Some(status_bar.into_any_element());
        self
    }
}

impl RenderOnce for AppShell {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let mut body = div().flex().flex_1().min_h_0();

        if let Some(left) = self.left {
            body = body.child(left).child(Divider::vertical());
        }
        body = body.child(self.center);
        if let Some(right) = self.right {
            body = body.child(Divider::vertical()).child(right);
        }

        div()
            .size_full()
            .bg(theme.app_bg)
            .text_color(theme.text)
            .flex()
            .flex_col()
            .when_some(self.title_bar, |this, title_bar| this.child(title_bar))
            .child(body)
            .when_some(self.status_bar, |this, status_bar| this.child(status_bar))
    }
}
