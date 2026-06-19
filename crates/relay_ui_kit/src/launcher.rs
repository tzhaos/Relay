//! Terminal and agent launcher menu.
//!
//! Relay's primary workflow is opening real terminals and launching CLI agents
//! inside them. This component renders the quick launcher without owning any
//! runtime behavior; hosts map selected keys to their command layer.

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius, space},
};

/// Category shown on the right edge of a launcher row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherItemKind {
    Terminal,
    Agent,
    Action,
}

impl LauncherItemKind {
    fn label(self) -> &'static str {
        match self {
            LauncherItemKind::Terminal => "TERM",
            LauncherItemKind::Agent => "AGENT",
            LauncherItemKind::Action => "ACTION",
        }
    }
}

/// One selectable command in a [`LauncherMenu`].
pub struct LauncherItem {
    key: &'static str,
    label: String,
    detail: Option<String>,
    icon: IconName,
    kind: LauncherItemKind,
    disabled: bool,
}

impl LauncherItem {
    pub fn new(key: &'static str, label: impl Into<String>, icon: IconName) -> Self {
        Self {
            key,
            label: label.into(),
            detail: None,
            icon,
            kind: LauncherItemKind::Action,
            disabled: false,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn kind(mut self, kind: LauncherItemKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

type SelectHandler = Box<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;

/// A compact launcher panel for terminal profiles and CLI agents.
#[derive(IntoElement)]
pub struct LauncherMenu {
    id: ElementId,
    items: Vec<LauncherItem>,
    on_select: Option<SelectHandler>,
}

impl LauncherMenu {
    pub fn new(id: impl Into<ElementId>, items: Vec<LauncherItem>) -> Self {
        Self {
            id: id.into(),
            items,
            on_select: None,
        }
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for LauncherMenu {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let handler = self.on_select.map(std::rc::Rc::new);
        let mut panel = div()
            .id(self.id)
            .w(px(320.0))
            .p(px(space::XS))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .rounded(px(radius::LG))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border_strong)
            .shadow_lg()
            .occlude();

        for (index, item) in self.items.into_iter().enumerate() {
            let handler = handler.clone();
            let key = item.key;
            let disabled = item.disabled || handler.is_none();
            let kind_label = item.kind.label();
            let kind_color = match item.kind {
                LauncherItemKind::Terminal => theme.text_secondary,
                LauncherItemKind::Agent => theme.accent,
                LauncherItemKind::Action => theme.text_muted,
            };

            let row = div()
                .id(("launcher-item", index))
                .min_h(px(42.0))
                .px_2()
                .py_1()
                .flex()
                .items_center()
                .gap_2()
                .rounded(px(radius::MD))
                .when(disabled, |this| this.opacity(0.5))
                .when(!disabled, |this| {
                    this.cursor_pointer()
                        .hover(move |style| style.bg(theme.hover))
                })
                .child(
                    div()
                        .size(px(26.0))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(radius::MD))
                        .bg(theme.panel_alt)
                        .border_1()
                        .border_color(theme.border)
                        .child(Icon::new(item.icon).size(IconSize::Small).color(kind_color)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(px(1.0))
                        .child(
                            div()
                                .truncate()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text)
                                .child(item.label),
                        )
                        .when_some(item.detail, |this, detail| {
                            this.child(
                                div()
                                    .truncate()
                                    .text_size(px(11.0))
                                    .text_color(theme.text_muted)
                                    .child(detail),
                            )
                        }),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .text_size(px(10.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(kind_color)
                        .child(kind_label),
                )
                .when_some(handler.filter(|_| !disabled), |this, handler| {
                    this.on_click(move |_: &ClickEvent, window, cx| {
                        handler(key, window, cx);
                        cx.stop_propagation();
                    })
                });
            panel = panel.child(row);
        }

        panel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_item_defaults_to_action() {
        let item = LauncherItem::new("settings", "Agent Settings", IconName::Settings);
        assert_eq!(item.kind, LauncherItemKind::Action);
    }

    #[test]
    fn launcher_kind_labels_are_stable() {
        assert_eq!(LauncherItemKind::Agent.label(), "AGENT");
    }
}
