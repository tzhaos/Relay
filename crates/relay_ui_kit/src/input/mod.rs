//! Keyboard-driven single-line text input.

mod state;
mod text_input;

pub use state::{TextInputAction, TextInputState};
pub use text_input::TextInput;
