use gpui::Hsla;

/// Visual treatment for one terminal output row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalLineStyle {
    Prompt,
    Input,
    Output,
    Muted,
    Success,
    Error,
}

impl TerminalLineStyle {
    pub(crate) fn color(self, theme: &crate::Theme) -> Hsla {
        match self {
            TerminalLineStyle::Prompt | TerminalLineStyle::Muted => theme.terminal_dim,
            TerminalLineStyle::Input | TerminalLineStyle::Output => theme.terminal_text,
            TerminalLineStyle::Success => theme.accent,
            TerminalLineStyle::Error => theme.danger,
        }
    }
}

/// A single row of text inside [`crate::TerminalSurface`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalLine {
    pub text: String,
    pub style: TerminalLineStyle,
}

impl TerminalLine {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TerminalLineStyle::Output,
        }
    }

    pub fn style(mut self, style: TerminalLineStyle) -> Self {
        self.style = style;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_line_defaults_to_output() {
        let line = TerminalLine::new("cargo build");
        assert_eq!(line.style, TerminalLineStyle::Output);
    }

    #[test]
    fn terminal_line_style_builder_sets_style() {
        let line = TerminalLine::new("error").style(TerminalLineStyle::Error);
        assert_eq!(line.style, TerminalLineStyle::Error);
    }
}
