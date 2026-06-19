use gpui::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius, space},
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A file/worktree tree node with indentation and optional disclosure chevron.
#[derive(IntoElement)]
pub struct TreeRow {
    id: ElementId,
    icon: IconName,
    label: String,
    depth: usize,
    expandable: bool,
    expanded: bool,
    selected: bool,
    on_click: Option<ClickHandler>,
}

impl TreeRow {
    pub fn new(id: impl Into<ElementId>, icon: IconName, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            icon,
            label: label.into(),
            depth: 0,
            expandable: false,
            expanded: false,
            selected: false,
            on_click: None,
        }
    }

    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    pub fn expandable(mut self, expanded: bool) -> Self {
        self.expandable = true;
        self.expanded = expanded;
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

impl RenderOnce for TreeRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = if self.selected {
            theme.text
        } else {
            theme.text_secondary
        };
        let indent = px(space::SM + self.depth as f32 * 14.0);
        let chevron = if self.expandable {
            Some(if self.expanded {
                IconName::ChevronDown
            } else {
                IconName::ChevronRight
            })
        } else {
            None
        };

        div()
            .id(self.id)
            .h(px(space::ROW_SM))
            .pr_2()
            .pl(indent)
            .flex()
            .items_center()
            .gap_1()
            .rounded(px(radius::SM))
            .text_color(fg)
            .when(self.selected, |this| this.bg(theme.selection))
            .when(!self.selected, |this| {
                this.cursor_pointer().hover(move |s| s.bg(theme.hover))
            })
            .child(
                div()
                    .w(px(14.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .when_some(chevron, |this, chevron| {
                        this.child(
                            Icon::new(chevron)
                                .size(IconSize::XSmall)
                                .color(theme.text_muted),
                        )
                    }),
            )
            .child(
                Icon::new(self.icon)
                    .size(IconSize::Small)
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .child(self.label),
            )
            .when_some(self.on_click, |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}
