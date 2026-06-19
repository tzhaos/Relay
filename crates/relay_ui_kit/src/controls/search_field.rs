use gpui::{
    App, ClickEvent, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius},
};

type KeyHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) -> bool + 'static>;

/// A focusable search/filter well with a leading magnifier icon.
#[derive(IntoElement)]
pub struct SearchField {
    id: ElementId,
    focus: FocusHandle,
    value: String,
    placeholder: String,
    key_context: &'static str,
    on_key: Option<KeyHandler>,
}

impl SearchField {
    pub fn new(id: impl Into<ElementId>, focus: FocusHandle) -> Self {
        Self {
            id: id.into(),
            focus,
            value: String::new(),
            placeholder: "Search".into(),
            key_context: "SearchField",
            on_key: None,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn key_context(mut self, key_context: &'static str) -> Self {
        self.key_context = key_context;
        self
    }

    pub fn on_key(
        mut self,
        handler: impl Fn(&KeyDownEvent, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.on_key = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SearchField {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let is_empty = self.value.is_empty();
        let display = if is_empty {
            self.placeholder.clone()
        } else {
            self.value.clone()
        };
        let text_color = if is_empty {
            theme.text_muted
        } else {
            theme.text
        };
        let focus_for_click = self.focus.clone();
        let on_key = self.on_key;

        div()
            .id(self.id)
            .h(px(30.0))
            .w_full()
            .flex()
            .items_center()
            .gap_2()
            .px_2()
            .rounded(px(radius::MD))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border)
            .track_focus(&self.focus)
            .tab_index(0)
            .key_context(self.key_context)
            .cursor(gpui::CursorStyle::IBeam)
            .hover(move |style| style.border_color(theme.border_strong))
            .child(
                Icon::new(IconName::Search)
                    .size(IconSize::Small)
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .text_color(text_color)
                    .child(display),
            )
            .when_some(on_key, |this, on_key| {
                this.on_key_down(move |event, window, cx| {
                    if on_key(event, window, cx) {
                        cx.stop_propagation();
                    }
                })
            })
            .on_click(move |_: &ClickEvent, window, _| {
                window.focus(&focus_for_click);
            })
    }
}
