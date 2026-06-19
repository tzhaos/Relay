use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius},
};

use super::KeyboardShortcut;

type SelectHandler = Box<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;

/// One command row inside a command palette or launcher.
#[derive(IntoElement)]
pub struct CommandRow {
    id: ElementId,
    key: &'static str,
    label: String,
    detail: Option<String>,
    icon: Option<IconName>,
    shortcut: Option<KeyboardShortcut>,
    selected: bool,
    disabled: bool,
    on_select: Option<SelectHandler>,
}

impl CommandRow {
    pub fn new(id: impl Into<ElementId>, key: &'static str, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            key,
            label: label.into(),
            detail: None,
            icon: None,
            shortcut: None,
            selected: false,
            disabled: false,
            on_select: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn shortcut(mut self, shortcut: KeyboardShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for CommandRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let handler = self.on_select;
        let disabled = self.disabled || handler.is_none();
        let key = self.key;

        div()
            .id(self.id)
            .min_h(px(44.0))
            .px_2()
            .py_1()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .when(self.selected, |this| this.bg(theme.selection))
            .when(disabled, |this| this.opacity(0.55))
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(theme.hover))
            })
            .when_some(self.icon, |this, icon| {
                this.child(
                    div()
                        .size(px(24.0))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            Icon::new(icon)
                                .size(IconSize::Small)
                                .color(theme.text_secondary),
                        ),
                )
            })
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(1.0))
                    .child(
                        div()
                            .truncate()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text)
                            .child(self.label),
                    )
                    .when_some(self.detail, |this, detail| {
                        this.child(
                            div()
                                .truncate()
                                .text_size(px(11.0))
                                .text_color(theme.text_muted)
                                .child(detail),
                        )
                    }),
            )
            .when_some(self.shortcut, |this, shortcut| this.child(shortcut))
            .when_some(handler.filter(|_| !disabled), |this, handler| {
                this.on_click(move |_: &ClickEvent, window, cx| {
                    handler(key, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}
