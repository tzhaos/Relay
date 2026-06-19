use gpui::{
    App, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, mono_family, space};

use super::TerminalLine;

/// A dark terminal output surface with optional prompt cursor.
#[derive(IntoElement)]
pub struct TerminalSurface {
    lines: Vec<TerminalLine>,
    prompt: Option<String>,
    connected: bool,
    cursor: bool,
}

impl TerminalSurface {
    pub fn new(lines: Vec<TerminalLine>) -> Self {
        Self {
            lines,
            prompt: None,
            connected: true,
            cursor: true,
        }
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn connected(mut self, connected: bool) -> Self {
        self.connected = connected;
        self
    }

    pub fn cursor(mut self, cursor: bool) -> Self {
        self.cursor = cursor;
        self
    }
}

impl RenderOnce for TerminalSurface {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let has_content = !self.lines.is_empty() || self.prompt.is_some();

        div()
            .flex_1()
            .min_h_0()
            .id("terminal-surface")
            .overflow_y_scroll()
            .bg(theme.terminal_bg)
            .p_3()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .font_family(mono_family())
            .text_size(px(13.0))
            .when(!has_content, |this| {
                this.items_center()
                    .justify_center()
                    .child(empty_terminal_state(theme, self.connected))
            })
            .children(
                self.lines
                    .into_iter()
                    .map(|line| terminal_line(theme, line)),
            )
            .when_some(self.prompt, |this, prompt| {
                this.child(prompt_line(theme, prompt, self.cursor))
            })
    }
}

fn terminal_line(theme: crate::Theme, line: TerminalLine) -> gpui::Div {
    div()
        .min_h(px(18.0))
        .text_color(line.style.color(&theme))
        .child(line.text)
}

fn prompt_line(theme: crate::Theme, prompt: String, cursor: bool) -> gpui::Div {
    div()
        .min_h(px(18.0))
        .flex()
        .items_center()
        .gap_1()
        .child(div().text_color(theme.terminal_dim).child(prompt))
        .when(cursor, |this| {
            this.child(div().w(px(8.0)).h(px(15.0)).bg(theme.terminal_text))
        })
}

fn empty_terminal_state(theme: crate::Theme, connected: bool) -> gpui::Div {
    let title = if connected {
        "No terminal output"
    } else {
        "No terminal session"
    };
    let detail = if connected {
        "The session is attached and waiting for output."
    } else {
        "Open a project or create a terminal session."
    };

    div()
        .max_w(px(360.0))
        .flex()
        .flex_col()
        .items_center()
        .gap_1()
        .px(px(space::LG))
        .text_center()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.terminal_text)
                .child(title),
        )
        .child(div().text_xs().text_color(theme.terminal_dim).child(detail))
}
