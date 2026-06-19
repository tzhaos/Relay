use gpui::{
    App, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use similar::{ChangeTag, TextDiff};

use crate::theme::{ActiveTheme, Theme, mono_family, radius};

/// Visual classification for a diff line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Added,
    Removed,
    Context,
}

/// One rendered diff row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub text: String,
}

impl DiffLine {
    pub fn new(kind: DiffLineKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            old_line: None,
            new_line: None,
            text: text.into(),
        }
    }

    pub fn with_numbers(
        kind: DiffLineKind,
        old_line: Option<usize>,
        new_line: Option<usize>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            old_line,
            new_line,
            text: text.into(),
        }
    }
}

/// A contiguous diff hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

impl DiffHunk {
    pub fn new(header: impl Into<String>, lines: Vec<DiffLine>) -> Self {
        Self {
            header: header.into(),
            lines,
        }
    }
}

/// A read-only unified diff view for changed files.
#[derive(IntoElement)]
pub struct DiffView {
    hunks: Vec<DiffHunk>,
}

impl DiffView {
    pub fn new(hunks: Vec<DiffHunk>) -> Self {
        Self { hunks }
    }

    pub fn from_text_diff(old: &str, new: &str) -> Self {
        Self::new(vec![DiffHunk::new("working tree", diff_lines(old, new))])
    }
}

impl RenderOnce for DiffView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();

        div()
            .id("diff-view")
            .size_full()
            .min_h_0()
            .overflow_y_scroll()
            .bg(theme.panel)
            .font_family(mono_family())
            .text_size(px(12.0))
            .flex()
            .flex_col()
            .children(
                self.hunks
                    .into_iter()
                    .map(move |hunk| render_hunk(theme, hunk)),
            )
    }
}

fn render_hunk(theme: Theme, hunk: DiffHunk) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(28.0))
                .px_3()
                .flex()
                .items_center()
                .border_b_1()
                .border_color(theme.border)
                .bg(theme.panel_alt)
                .text_color(theme.text_muted)
                .font_weight(FontWeight::SEMIBOLD)
                .child(hunk.header),
        )
        .children(
            hunk.lines
                .into_iter()
                .map(move |line| render_line(theme, line)),
        )
}

fn render_line(theme: Theme, line: DiffLine) -> gpui::Div {
    let (prefix, fg, bg) = match line.kind {
        DiffLineKind::Added => ("+", theme.accent, theme.accent_bg),
        DiffLineKind::Removed => ("-", theme.danger, theme.panel_alt),
        DiffLineKind::Context => (" ", theme.text_muted, gpui::transparent_black()),
    };

    div()
        .min_h(px(20.0))
        .px_3()
        .flex()
        .items_center()
        .gap_2()
        .bg(bg)
        .child(line_number_cell(theme, line.old_line))
        .child(line_number_cell(theme, line.new_line))
        .child(
            div()
                .w(px(12.0))
                .flex_shrink_0()
                .text_color(fg)
                .font_weight(FontWeight::SEMIBOLD)
                .child(prefix),
        )
        .child(
            div()
                .min_w_0()
                .rounded(px(radius::SM))
                .text_color(theme.text_secondary)
                .child(if line.text.is_empty() {
                    " ".into()
                } else {
                    line.text
                }),
        )
}

fn line_number_cell(theme: Theme, line: Option<usize>) -> gpui::Div {
    div()
        .w(px(34.0))
        .flex_shrink_0()
        .text_right()
        .text_color(theme.text_muted)
        .child(line.map(|line| line.to_string()).unwrap_or_default())
}

fn diff_lines(old: &str, new: &str) -> Vec<DiffLine> {
    let mut old_line = 1;
    let mut new_line = 1;

    TextDiff::from_lines(old, new)
        .iter_all_changes()
        .map(|change| {
            let (kind, current_old, current_new) = match change.tag() {
                ChangeTag::Delete => {
                    let current = old_line;
                    old_line += 1;
                    (DiffLineKind::Removed, Some(current), None)
                }
                ChangeTag::Insert => {
                    let current = new_line;
                    new_line += 1;
                    (DiffLineKind::Added, None, Some(current))
                }
                ChangeTag::Equal => {
                    let current_old = old_line;
                    let current_new = new_line;
                    old_line += 1;
                    new_line += 1;
                    (DiffLineKind::Context, Some(current_old), Some(current_new))
                }
            };
            DiffLine::with_numbers(
                kind,
                current_old,
                current_new,
                change.to_string().trim_end_matches('\n'),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_lines_marks_insertions() {
        let lines = diff_lines("a\n", "a\nb\n");
        assert!(lines.iter().any(|line| line.kind == DiffLineKind::Added));
    }

    #[test]
    fn diff_lines_assigns_new_line_number_to_insertions() {
        let lines = diff_lines("a\n", "a\nb\n");
        let added = lines
            .iter()
            .find(|line| line.kind == DiffLineKind::Added)
            .unwrap();
        assert_eq!(added.new_line, Some(2));
    }
}
