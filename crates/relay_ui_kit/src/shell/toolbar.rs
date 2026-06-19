use gpui::{
    AnyElement, App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::ActiveTheme,
};

/// A compact toolbar for title bars or pane headers.
#[derive(IntoElement)]
pub struct TopToolbar {
    leading: Option<AnyElement>,
    center: Option<AnyElement>,
    trailing: Option<AnyElement>,
}

impl TopToolbar {
    pub fn new() -> Self {
        Self {
            leading: None,
            center: None,
            trailing: None,
        }
    }

    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self
    }

    pub fn center(mut self, center: impl IntoElement) -> Self {
        self.center = Some(center.into_any_element());
        self
    }

    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }
}

impl Default for TopToolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for TopToolbar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .h_full()
            .min_w_0()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .when_some(self.leading, |this, leading| {
                this.child(div().min_w_0().flex().items_center().gap_2().child(leading))
            })
            .when_some(self.center, |this, center| {
                this.child(
                    div()
                        .min_w_0()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(center),
                )
            })
            .when_some(self.trailing, |this, trailing| {
                this.child(
                    div()
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(trailing),
                )
            })
    }
}

/// A project/worktree breadcrumb for title bar context.
#[derive(IntoElement)]
pub struct WorkspaceBreadcrumb {
    items: Vec<String>,
    active: bool,
}

impl WorkspaceBreadcrumb {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            active: true,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

impl RenderOnce for WorkspaceBreadcrumb {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = if self.active {
            theme.text_muted
        } else {
            theme.text_muted
        };
        let last = self.items.len().saturating_sub(1);
        let mut row = div()
            .h(px(24.0))
            .max_w(px(560.0))
            .flex()
            .items_center()
            .gap_1()
            .text_sm()
            .text_color(fg);

        for (index, item) in self.items.into_iter().enumerate() {
            row = row.child(
                div()
                    .min_w_0()
                    .truncate()
                    .font_weight(if index == last {
                        FontWeight::MEDIUM
                    } else {
                        FontWeight::MEDIUM
                    })
                    .text_color(if index == last && self.active {
                        theme.text_secondary
                    } else {
                        fg
                    })
                    .child(item),
            );
            if index != last {
                row = row.child(
                    Icon::new(IconName::ChevronRight)
                        .size(IconSize::XSmall)
                        .color(theme.text_muted),
                );
            }
        }

        row
    }
}

/// A small action toolbar for pane headers.
#[derive(IntoElement)]
pub struct PaneToolbar {
    actions: Vec<AnyElement>,
}

impl PaneToolbar {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}

impl Default for PaneToolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for PaneToolbar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div().flex().items_center().gap_1().children(self.actions)
    }
}
