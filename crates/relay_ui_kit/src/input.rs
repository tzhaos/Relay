//! Keyboard-driven text input: [`TextInputState`] (the editable model the host
//! owns) and [`TextInput`] (the `RenderOnce` view that draws it with a caret).
//!
//! A fully native input would implement `EntityInputHandler` (IME composition,
//! mouse caret placement, UTF-16 selection). That is heavier than the kit needs:
//! the workbench composers are short single-line fields. So this is a pragmatic,
//! genuinely-editable model — insert / backspace / delete / cursor motion /
//! home / end — driven by `on_key_down`. Chinese and other IME text still arrives
//! through `key_char` (`with_simulated_ime`), so CJK input works.
//!
//! The host holds a [`TextInputState`], feeds each `KeyDownEvent` to
//! [`TextInputState::handle_key`], and renders a [`TextInput`] bound to a
//! [`FocusHandle`]. The component is stateless; all mutation lives in the model.

use gpui::{
    App, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius},
};

/// The editable model for a [`TextInput`]. The host owns one of these per field.
///
/// `cursor` is a byte offset into `value` that always sits on a UTF-8 char
/// boundary (every mutation keeps it there). Editing is grapheme-aware so
/// backspacing an emoji or combining sequence removes the whole cluster.
#[derive(Debug, Clone, Default)]
pub struct TextInputState {
    value: String,
    cursor: usize,
}

/// What a keystroke did to the model, so the host can react (e.g. submit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextInputAction {
    /// Text or cursor changed; re-render and propagate the new value.
    Edited,
    /// Enter pressed.
    Submit,
    /// Escape pressed.
    Cancel,
    /// The key was not handled by the input (let it bubble).
    Ignored,
}

impl TextInputState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed the model with initial text, cursor at the end.
    pub fn with_text(text: impl Into<String>) -> Self {
        let value = text.into();
        let cursor = value.len();
        Self { value, cursor }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Replace all text and move the cursor to the end.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.value = text.into();
        self.cursor = self.value.len();
    }

    /// Clear the field.
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    /// Feed a key event to the model. Returns what happened so the host can
    /// decide whether to submit, propagate the filter text, etc.
    pub fn handle_key(&mut self, event: &KeyDownEvent) -> TextInputAction {
        let keystroke = event.keystroke.clone().with_simulated_ime();
        let mods = keystroke.modifiers;

        match keystroke.key.as_str() {
            "enter" => TextInputAction::Submit,
            "escape" => TextInputAction::Cancel,
            "backspace" => {
                if self.delete_grapheme_before() {
                    TextInputAction::Edited
                } else {
                    TextInputAction::Ignored
                }
            }
            "delete" => {
                if self.delete_grapheme_after() {
                    TextInputAction::Edited
                } else {
                    TextInputAction::Ignored
                }
            }
            "left" => {
                if self.move_left() {
                    TextInputAction::Edited
                } else {
                    TextInputAction::Ignored
                }
            }
            "right" => {
                if self.move_right() {
                    TextInputAction::Edited
                } else {
                    TextInputAction::Ignored
                }
            }
            "home" => {
                if self.cursor != 0 {
                    self.cursor = 0;
                    TextInputAction::Edited
                } else {
                    TextInputAction::Ignored
                }
            }
            "end" => {
                if self.cursor != self.value.len() {
                    self.cursor = self.value.len();
                    TextInputAction::Edited
                } else {
                    TextInputAction::Ignored
                }
            }
            // Printable text (including IME output). Reject control-modifier
            // combos and control characters so shortcuts don't insert glyphs.
            _ if !mods.control && !mods.alt && !mods.platform && !mods.function => {
                match keystroke
                    .key_char
                    .as_ref()
                    .filter(|text| text.chars().all(|c| !c.is_control()))
                {
                    Some(text) => {
                        self.insert(text);
                        TextInputAction::Edited
                    }
                    None => TextInputAction::Ignored,
                }
            }
            _ => TextInputAction::Ignored,
        }
    }

    fn insert(&mut self, text: &str) {
        self.value.insert_str(self.cursor, text);
        self.cursor += text.len();
    }

    fn delete_grapheme_before(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        let prev = self.prev_boundary(self.cursor);
        self.value.replace_range(prev..self.cursor, "");
        self.cursor = prev;
        true
    }

    fn delete_grapheme_after(&mut self) -> bool {
        if self.cursor >= self.value.len() {
            return false;
        }
        let next = self.next_boundary(self.cursor);
        self.value.replace_range(self.cursor..next, "");
        true
    }

    fn move_left(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor = self.prev_boundary(self.cursor);
        true
    }

    fn move_right(&mut self) -> bool {
        if self.cursor >= self.value.len() {
            return false;
        }
        self.cursor = self.next_boundary(self.cursor);
        true
    }

    /// Byte offset of the grapheme boundary immediately before `byte`.
    fn prev_boundary(&self, byte: usize) -> usize {
        self.value[..byte]
            .grapheme_indices(true)
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Byte offset of the grapheme boundary immediately after `byte`.
    fn next_boundary(&self, byte: usize) -> usize {
        self.value[byte..]
            .grapheme_indices(true)
            .nth(1)
            .map(|(i, _)| byte + i)
            .unwrap_or(self.value.len())
    }

    /// Split the value at the cursor for rendering (before, after).
    fn split(&self) -> (&str, &str) {
        self.value.split_at(self.cursor)
    }
}

/// A single-line text input. Stateless: it renders a [`TextInputState`] plus a
/// caret, and tracks focus through a host-owned [`FocusHandle`]. Wire `on_key`
/// to push events into the model.
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
    on_key: Option<Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>>,
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

    /// Whether this field currently holds focus (drives the caret + accent ring).
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

        // The text run: before-cursor, caret (only when focused), after-cursor.
        // When empty and unfocused, show the placeholder instead.
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

/// A 2px-wide caret bar matching the line height.
fn caret(color: gpui::Hsla) -> impl IntoElement {
    div().w(px(1.5)).h(px(16.0)).bg(color)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(name: &str, ch: Option<&str>) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: gpui::Keystroke {
                key: name.to_string(),
                key_char: ch.map(|c| c.to_string()),
                ..Default::default()
            },
            is_held: false,
        }
    }

    #[test]
    fn typing_inserts_at_cursor() {
        let mut s = TextInputState::new();
        assert_eq!(s.handle_key(&key("h", Some("h"))), TextInputAction::Edited);
        s.handle_key(&key("i", Some("i")));
        assert_eq!(s.value(), "hi");
        assert_eq!(s.cursor(), 2);
    }

    #[test]
    fn backspace_removes_grapheme_before_cursor() {
        let mut s = TextInputState::with_text("hi");
        assert_eq!(
            s.handle_key(&key("backspace", None)),
            TextInputAction::Edited
        );
        assert_eq!(s.value(), "h");
        // Backspace at start is a no-op.
        s.handle_key(&key("backspace", None));
        assert_eq!(
            s.handle_key(&key("backspace", None)),
            TextInputAction::Ignored
        );
    }

    #[test]
    fn arrows_move_cursor_and_insert_lands_mid_string() {
        let mut s = TextInputState::with_text("ac");
        // Move left once: cursor between a and c.
        assert_eq!(s.handle_key(&key("left", None)), TextInputAction::Edited);
        assert_eq!(s.cursor(), 1);
        s.handle_key(&key("b", Some("b")));
        assert_eq!(s.value(), "abc");
    }

    #[test]
    fn enter_and_escape_report_intents() {
        let mut s = TextInputState::with_text("x");
        assert_eq!(s.handle_key(&key("enter", None)), TextInputAction::Submit);
        assert_eq!(s.handle_key(&key("escape", None)), TextInputAction::Cancel);
    }

    #[test]
    fn home_end_jump_to_edges() {
        let mut s = TextInputState::with_text("abc");
        assert_eq!(s.handle_key(&key("home", None)), TextInputAction::Edited);
        assert_eq!(s.cursor(), 0);
        assert_eq!(s.handle_key(&key("end", None)), TextInputAction::Edited);
        assert_eq!(s.cursor(), 3);
    }

    #[test]
    fn cjk_input_via_key_char() {
        let mut s = TextInputState::new();
        s.handle_key(&key("中", Some("中")));
        s.handle_key(&key("文", Some("文")));
        assert_eq!(s.value(), "中文");
        // Backspace removes a whole multi-byte char.
        s.handle_key(&key("backspace", None));
        assert_eq!(s.value(), "中");
    }

    #[test]
    fn ctrl_combo_is_ignored() {
        let mut s = TextInputState::new();
        let mut k = key("a", Some("a"));
        k.keystroke.modifiers.control = true;
        assert_eq!(s.handle_key(&k), TextInputAction::Ignored);
        assert!(s.is_empty());
    }
}
