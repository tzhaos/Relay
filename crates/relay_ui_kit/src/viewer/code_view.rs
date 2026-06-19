use gpui::{
    App, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, Theme, mono_family, radius};

/// A lightweight read-only code/file text surface.
#[derive(IntoElement)]
pub struct CodeView {
    lines: Vec<String>,
    language: Option<String>,
    line_numbers: bool,
}

impl CodeView {
    /// Creates a read-only code view from source text.
    pub fn new(source: impl Into<String>) -> Self {
        let source = source.into();
        Self {
            lines: split_lines(&source),
            language: None,
            line_numbers: true,
        }
    }

    /// Shows a compact language label above the text.
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Toggles line numbers.
    pub fn line_numbers(mut self, line_numbers: bool) -> Self {
        self.line_numbers = line_numbers;
        self
    }
}

impl RenderOnce for CodeView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();

        div()
            .id("code-view")
            .size_full()
            .min_h_0()
            .overflow_y_scroll()
            .bg(theme.panel)
            .p_3()
            .font_family(mono_family())
            .text_size(px(12.0))
            .flex()
            .flex_col()
            .gap(px(1.0))
            .when_some(self.language, |this, language| {
                this.child(
                    div()
                        .mb_2()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text_muted)
                        .child(language),
                )
            })
            .children(
                self.lines
                    .into_iter()
                    .enumerate()
                    .map(move |(index, line)| code_line(theme, index + 1, line, self.line_numbers)),
            )
    }
}

fn code_line(theme: Theme, number: usize, line: String, show_number: bool) -> gpui::Div {
    div()
        .min_h(px(18.0))
        .flex()
        .items_start()
        .gap_3()
        .rounded(px(radius::SM))
        .child(
            div()
                .w(px(34.0))
                .flex_shrink_0()
                .text_right()
                .text_color(if show_number {
                    theme.text_muted
                } else {
                    gpui::transparent_black()
                })
                .child(number.to_string()),
        )
        .child(
            div()
                .min_w_0()
                .text_color(theme.text_secondary)
                .child(if line.is_empty() { " ".into() } else { line }),
        )
}

fn split_lines(source: &str) -> Vec<String> {
    if source.is_empty() {
        return Vec::new();
    }
    source
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_lines_preserves_code_line_count() {
        assert_eq!(split_lines("a\nb\nc").len(), 3);
    }

    #[test]
    fn split_lines_preserves_trailing_blank_line() {
        assert_eq!(split_lines("a\n").len(), 2);
    }
}
