use gpui::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, space};

/// Toolbar above a terminal surface: tabs on the left, actions on the right.
#[derive(IntoElement)]
pub struct TerminalToolbar {
    tabs: Vec<AnyElement>,
    actions: Option<AnyElement>,
}

impl TerminalToolbar {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            actions: None,
        }
    }

    pub fn tab(mut self, tab: impl IntoElement) -> Self {
        self.tabs.push(tab.into_any_element());
        self
    }

    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions = Some(actions.into_any_element());
        self
    }
}

impl Default for TerminalToolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for TerminalToolbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .h(px(space::PANE_HEADER))
            .flex_shrink_0()
            .px_2()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.chrome)
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .child(
                div()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap_1()
                    .children(self.tabs),
            )
            .when_some(self.actions, |this, actions| {
                this.child(
                    div()
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(actions),
                )
            })
    }
}
