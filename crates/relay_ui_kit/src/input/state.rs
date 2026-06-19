use gpui::KeyDownEvent;
use unicode_segmentation::UnicodeSegmentation;

/// The editable model for a [`crate::TextInput`].
#[derive(Debug, Clone, Default)]
pub struct TextInputState {
    value: String,
    cursor: usize,
}

/// What a keystroke did to the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextInputAction {
    Edited,
    Submit,
    Cancel,
    Ignored,
}

impl TextInputState {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.value = text.into();
        self.cursor = self.value.len();
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

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

    pub(crate) fn split(&self) -> (&str, &str) {
        self.value.split_at(self.cursor)
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

    fn prev_boundary(&self, byte: usize) -> usize {
        self.value[..byte]
            .grapheme_indices(true)
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn next_boundary(&self, byte: usize) -> usize {
        self.value[byte..]
            .grapheme_indices(true)
            .nth(1)
            .map(|(i, _)| byte + i)
            .unwrap_or(self.value.len())
    }
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
    }

    #[test]
    fn backspace_removes_grapheme_before_cursor() {
        let mut s = TextInputState::with_text("hi");
        assert_eq!(
            s.handle_key(&key("backspace", None)),
            TextInputAction::Edited
        );
        assert_eq!(s.value(), "h");
    }

    #[test]
    fn arrows_move_cursor_and_insert_lands_mid_string() {
        let mut s = TextInputState::with_text("ac");
        assert_eq!(s.handle_key(&key("left", None)), TextInputAction::Edited);
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
