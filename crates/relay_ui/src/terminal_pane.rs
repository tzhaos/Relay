use gpui::{IntoElement, div, prelude::*};
use relay_core::TerminalSessionId;

use crate::theme::RelayTheme;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalPaneProjection {
    pub session_id: Option<TerminalSessionId>,
    pub cwd: String,
    pub title: Option<String>,
    pub scrollback: String,
    pub exited: bool,
}

impl TerminalPaneProjection {
    pub fn detached() -> Self {
        Self {
            session_id: None,
            cwd: String::new(),
            title: None,
            scrollback: String::new(),
            exited: false,
        }
    }
}

pub fn terminal_pane(theme: RelayTheme, projection: &TerminalPaneProjection) -> impl IntoElement {
    let status = if projection.exited {
        "EXITED"
    } else if projection.session_id.is_some() {
        "ATTACHED"
    } else {
        "DETACHED"
    };

    let title = projection
        .title
        .clone()
        .unwrap_or_else(|| "Relay terminal".to_string());
    let cwd = if projection.cwd.is_empty() {
        "No worktree attached".to_string()
    } else {
        projection.cwd.clone()
    };
    let scrollback = if projection.scrollback.is_empty() {
        "relay $ waiting for terminal runtime...".to_string()
    } else {
        projection.scrollback.clone()
    };

    div()
        .flex_1()
        .h_full()
        .bg(theme.bg)
        .p_4()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(div().text_sm().text_color(theme.muted).child("TERMINAL"))
                .child(
                    div()
                        .text_xs()
                        .text_color(if projection.exited {
                            theme.muted
                        } else {
                            theme.accent
                        })
                        .font_weight(gpui::FontWeight::BOLD)
                        .child(status),
                ),
        )
        .child(
            div()
                .flex_1()
                .rounded_md()
                .border_1()
                .border_color(theme.line)
                .bg(theme.panel)
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(div().text_color(theme.text).child(title))
                        .child(div().text_sm().text_color(theme.muted).child(cwd)),
                )
                .child(
                    div()
                        .flex_1()
                        .font_family("Consolas")
                        .text_color(theme.text)
                        .bg(theme.panel_alt)
                        .border_1()
                        .border_color(theme.line)
                        .rounded_md()
                        .p_3()
                        .child(scrollback),
                ),
        )
}
