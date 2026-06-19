use gpui::{App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

use crate::theme::{ActiveTheme, radius};

/// A visual keyboard shortcut hint, rendered as compact keycaps.
#[derive(Clone, Debug, PartialEq, Eq, IntoElement)]
pub struct KeyboardShortcut {
    keys: Vec<String>,
}

impl KeyboardShortcut {
    pub fn new(keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            keys: keys.into_iter().map(Into::into).collect(),
        }
    }

    pub fn keys(&self) -> &[String] {
        &self.keys
    }
}

impl RenderOnce for KeyboardShortcut {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .flex()
            .items_center()
            .gap(px(2.0))
            .children(self.keys.into_iter().map(|key| {
                div()
                    .min_w(px(18.0))
                    .h(px(18.0))
                    .px_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(radius::SM))
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.panel_alt)
                    .text_color(theme.text_muted)
                    .text_size(px(10.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(key)
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_preserves_key_order() {
        let shortcut = KeyboardShortcut::new(["Ctrl", "K"]);
        assert_eq!(shortcut.keys(), ["Ctrl", "K"]);
    }
}
