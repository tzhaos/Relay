use gpui::{
    App, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius},
};

use super::TextInputState;

type KeyHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;

/// A single-line text input view. The host owns the editable state.
#[derive(IntoElement)]
pub struct TextInput {
    id: ElementId,
    focus: FocusHandle,
    before: String,
    after: String,
    is_empty: bool,
    placeholder: String,
    leading_icon: Option<IconName>,
    focused: bool,
    key_context: &'static str,
    on_key: Option<KeyHandler>,
}

impl TextInput {
    pub fn new(id: impl Into<ElementId>, focus: FocusHandle, state: &TextInputState) -> Self {
        let (before, after) = state.split();
        Self {
            id: id.into(),
            focus,
            before: before.to_string(),
            after: after.to_string(),
            is_empty: state.is_empty(),
            placeholder: String::new(),
            leading_icon: None,
            focused: false,
            key_context: "TextInput",
            on_key: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn key_context(mut self, key_context: &'static str) -> Self {
        self.key_context = key_context;
        self
    }

    pub fn on_key(
        mut self,
        handler: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_key = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TextInput {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let border = if self.focused {
            theme.accent
        } else {
            theme.border_strong
        };
        let focus_for_click = self.focus.clone();
        let on_key = self.on_key;
        let show_placeholder = self.is_empty && !self.focused;

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
            .border_color(border)
            .track_focus(&self.focus)
            .tab_index(0)
            .key_context(self.key_context)
            .cursor(gpui::CursorStyle::IBeam)
            .when(!self.focused, |this| {
                this.hover(move |s| s.border_color(theme.border_strong))
            })
            .when_some(self.leading_icon, |this, icon| {
                this.child(
                    Icon::new(icon)
                        .size(IconSize::Small)
                        .color(theme.text_muted),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .text_sm()
                    .when(show_placeholder, |this| {
                        this.text_color(theme.text_muted).child(self.placeholder)
                    })
                    .when(!show_placeholder, |this| {
                        this.text_color(theme.text)
                            .child(self.before)
                            .when(self.focused, |this| this.child(caret(theme.accent)))
                            .child(self.after)
                    }),
            )
            .when_some(on_key, |this, on_key| {
                this.on_key_down(move |event, window, cx| {
                    on_key(event, window, cx);
                    cx.stop_propagation();
                })
            })
            .on_click(move |_, window, _| {
                window.focus(&focus_for_click);
            })
    }
}

fn caret(color: gpui::Hsla) -> impl IntoElement {
    div().w(px(1.5)).h(px(16.0)).bg(color)
}
