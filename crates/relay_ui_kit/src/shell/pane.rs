use gpui::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, space};

/// Fixed or flexible pane sizing used by [`Pane`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaneWidth {
    Fixed(f32),
    Flex,
}

/// Surface treatment for a workbench pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneSurface {
    Chrome,
    Panel,
    Inset,
    Transparent,
}

/// A stable pane container with an optional header and a body slot.
#[derive(IntoElement)]
pub struct Pane {
    width: PaneWidth,
    surface: PaneSurface,
    header: Option<AnyElement>,
    body: AnyElement,
}

impl Pane {
    pub fn new(width: PaneWidth, body: impl IntoElement) -> Self {
        Self {
            width,
            surface: PaneSurface::Panel,
            header: None,
            body: body.into_any_element(),
        }
    }

    pub fn rail(body: impl IntoElement) -> Self {
        Self::new(PaneWidth::Fixed(space::RAIL_WIDTH), body).surface(PaneSurface::Chrome)
    }

    pub fn center(body: impl IntoElement) -> Self {
        Self::new(PaneWidth::Flex, body)
    }

    pub fn context(body: impl IntoElement) -> Self {
        Self::new(PaneWidth::Fixed(space::CONTEXT_WIDTH), body).surface(PaneSurface::Chrome)
    }

    pub fn surface(mut self, surface: PaneSurface) -> Self {
        self.surface = surface;
        self
    }

    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }
}

impl RenderOnce for Pane {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let bg = match self.surface {
            PaneSurface::Chrome => theme.chrome,
            PaneSurface::Panel => theme.panel,
            PaneSurface::Inset => theme.inset,
            PaneSurface::Transparent => gpui::transparent_black(),
        };
        let root = div()
            .h_full()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(bg)
            .when_some(self.header, |this, header| this.child(header))
            .child(div().flex_1().min_h_0().child(self.body));

        match self.width {
            PaneWidth::Fixed(width) => root.w(px(width)).flex_shrink_0(),
            PaneWidth::Flex => root.flex_1().min_w_0(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pane_width_preserves_fixed_value() {
        assert_eq!(PaneWidth::Fixed(space::RAIL_WIDTH), PaneWidth::Fixed(300.0));
    }
}
