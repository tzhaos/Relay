use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius},
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Compact shortcut for launching a CLI agent into a terminal.
#[derive(IntoElement)]
pub struct AgentQuickLaunch {
    id: ElementId,
    label: String,
    command: String,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl AgentQuickLaunch {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<String>,
        command: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            command: command.into(),
            disabled: false,
            on_click: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

impl RenderOnce for AgentQuickLaunch {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let handler = self.on_click;
        let disabled = self.disabled || handler.is_none();

        div()
            .id(self.id)
            .h(px(30.0))
            .px_2()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .border_1()
            .border_color(theme.accent_border)
            .bg(theme.accent_bg)
            .text_color(theme.accent)
            .when(disabled, |this| this.opacity(0.5))
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(theme.selection))
            })
            .child(
                Icon::new(IconName::Bot)
                    .size(IconSize::Small)
                    .color(theme.accent),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.label),
            )
            .child(
                div()
                    .max_w(px(140.0))
                    .truncate()
                    .text_size(px(11.0))
                    .text_color(theme.text_muted)
                    .child(self.command),
            )
            .when_some(handler.filter(|_| !disabled), |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}
