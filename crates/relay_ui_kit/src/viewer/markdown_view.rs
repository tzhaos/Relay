use gpui::{
    App, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::theme::{ActiveTheme, Theme, mono_family, radius};

#[derive(Debug, Clone, PartialEq, Eq)]
enum MarkdownBlock {
    Heading(u8, String),
    Paragraph(String),
    ListItem(String),
    Code(String),
    Quote(String),
    Rule,
}

/// A compact read-only Markdown preview for context panes.
#[derive(IntoElement)]
pub struct MarkdownView {
    blocks: Vec<MarkdownBlock>,
}

impl MarkdownView {
    /// Parses Markdown into a lightweight block model for display.
    pub fn new(source: impl AsRef<str>) -> Self {
        Self {
            blocks: parse_markdown(source.as_ref()),
        }
    }
}

impl RenderOnce for MarkdownView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();

        div()
            .id("markdown-view")
            .size_full()
            .min_h_0()
            .overflow_y_scroll()
            .bg(theme.panel)
            .p_3()
            .flex()
            .flex_col()
            .gap_2()
            .children(
                self.blocks
                    .into_iter()
                    .map(move |block| render_block(theme, block)),
            )
    }
}

fn render_block(theme: Theme, block: MarkdownBlock) -> gpui::Div {
    match block {
        MarkdownBlock::Heading(level, text) => div()
            .text_size(px(if level <= 1 { 16.0 } else { 14.0 }))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.text)
            .child(text),
        MarkdownBlock::Paragraph(text) => {
            div().text_sm().text_color(theme.text_secondary).child(text)
        }
        MarkdownBlock::ListItem(text) => div()
            .flex()
            .gap_2()
            .text_sm()
            .text_color(theme.text_secondary)
            .child(
                div()
                    .flex_shrink_0()
                    .text_color(theme.text_muted)
                    .child("-"),
            )
            .child(div().min_w_0().child(text)),
        MarkdownBlock::Code(text) => div()
            .p_2()
            .rounded(px(radius::MD))
            .bg(theme.inset)
            .font_family(mono_family())
            .text_size(px(12.0))
            .text_color(theme.text_secondary)
            .child(text),
        MarkdownBlock::Quote(text) => div()
            .pl_2()
            .border_l_1()
            .border_color(theme.border_strong)
            .text_sm()
            .text_color(theme.text_muted)
            .child(text),
        MarkdownBlock::Rule => div().h(px(1.0)).bg(theme.border),
    }
}

fn parse_markdown(source: &str) -> Vec<MarkdownBlock> {
    let parser = Parser::new_ext(source, Options::ENABLE_STRIKETHROUGH);
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut mode = BlockMode::Paragraph;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current.clear();
                mode = BlockMode::Heading(level_to_u8(level));
            }
            Event::Start(Tag::Item) => {
                current.clear();
                mode = BlockMode::ListItem;
            }
            Event::Start(Tag::BlockQuote(_)) => {
                current.clear();
                mode = BlockMode::Quote;
            }
            Event::Start(Tag::CodeBlock(_)) => {
                current.clear();
                mode = BlockMode::Code;
            }
            Event::Text(text) | Event::Code(text) => current.push_str(&text),
            Event::SoftBreak | Event::HardBreak => {
                if matches!(mode, BlockMode::Code) {
                    current.push('\n');
                } else {
                    current.push(' ');
                }
            }
            Event::Rule => blocks.push(MarkdownBlock::Rule),
            Event::End(TagEnd::Heading(_))
            | Event::End(TagEnd::Paragraph)
            | Event::End(TagEnd::Item)
            | Event::End(TagEnd::BlockQuote(_))
            | Event::End(TagEnd::CodeBlock) => {
                push_current(&mut blocks, mode, &mut current);
                mode = BlockMode::Paragraph;
            }
            _ => {}
        }
    }

    push_current(&mut blocks, mode, &mut current);
    blocks
}

#[derive(Clone, Copy)]
enum BlockMode {
    Heading(u8),
    Paragraph,
    ListItem,
    Code,
    Quote,
}

fn push_current(blocks: &mut Vec<MarkdownBlock>, mode: BlockMode, current: &mut String) {
    let text = current.trim();
    if text.is_empty() {
        current.clear();
        return;
    }

    blocks.push(match mode {
        BlockMode::Heading(level) => MarkdownBlock::Heading(level, text.to_string()),
        BlockMode::Paragraph => MarkdownBlock::Paragraph(text.to_string()),
        BlockMode::ListItem => MarkdownBlock::ListItem(text.to_string()),
        BlockMode::Code => MarkdownBlock::Code(text.to_string()),
        BlockMode::Quote => MarkdownBlock::Quote(text.to_string()),
    });
    current.clear();
}

fn level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_markdown_extracts_heading_and_list_item() {
        let blocks = parse_markdown("# Relay\n\n- Terminal context");
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn parse_markdown_preserves_code_block_newlines() {
        let blocks = parse_markdown("```rust\nfn main() {}\nprintln!();\n```");
        assert!(matches!(&blocks[0], MarkdownBlock::Code(text) if text.contains('\n')));
    }
}
