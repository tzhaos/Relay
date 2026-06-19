use gpui::{
    Context, CursorStyle, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    StatefulInteractiveElement, div, prelude::*, px,
};
use relay_core::{TaskStatus, TerminalSessionId};

use crate::{
    app_shell::AppShell,
    components::{self, ButtonEmphasis, Tone},
    preview_pane::preview_content,
    theme::{RelayTheme, mono_family, spacing},
    workbench::{PaneRoute, WorkbenchCommand, WorkspaceViewModel},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalPaneProjection {
    pub session_id: Option<TerminalSessionId>,
    pub cwd: String,
    pub title: Option<String>,
    pub scrollback: String,
    pub exited: bool,
    pub connected: bool,
}

impl TerminalPaneProjection {
    pub fn detached() -> Self {
        Self {
            session_id: None,
            cwd: String::new(),
            title: None,
            scrollback: String::new(),
            exited: false,
            connected: false,
        }
    }
}

pub fn terminal_pane(
    theme: RelayTheme,
    view_model: &WorkspaceViewModel,
    projection: &TerminalPaneProjection,
    terminal_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let status = if projection.exited {
        "EXITED"
    } else if projection.connected {
        "SESSION"
    } else if projection.session_id.is_some() {
        "OFFLINE"
    } else {
        "DETACHED"
    };
    let status_tone = if projection.exited {
        Tone::Muted
    } else if projection.connected {
        Tone::Accent
    } else {
        Tone::Warning
    };

    let cwd = if projection.cwd.is_empty() {
        "No worktree attached".to_string()
    } else {
        projection.cwd.clone()
    };
    let launchable_task = view_model.active_task().filter(|task| {
        task.status == TaskStatus::ReadyForAgent
            && task.agent.is_none()
            && projection.connected
            && !projection.exited
    });
    let workspace_terminal_session_id = projection.session_id.filter(|session_id| {
        projection.connected
            && !projection.exited
            && Some(*session_id) == view_model.workspace_terminal_session_id
            && launchable_task.is_none()
    });

    div()
        .flex_1()
        .min_w_0()
        .h_full()
        .bg(theme.app_bg)
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(spacing::PANE_HEADER))
                .px_3()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(theme.border)
                .bg(theme.chrome)
                .child(
                    div()
                        .min_w_0()
                        .flex()
                        .items_end()
                        .h(px(spacing::PANE_HEADER))
                        .child(route_tab(
                            theme,
                            view_model.pane_route,
                            PaneRoute::Terminal,
                            "Terminal",
                            terminal_focus,
                            cx,
                        ))
                        .child(route_tab(
                            theme,
                            view_model.pane_route,
                            PaneRoute::Preview,
                            "Preview",
                            terminal_focus,
                            cx,
                        )),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .max_w(px(420.0))
                                .truncate()
                                .text_sm()
                                .font_family(mono_family())
                                .text_color(theme.text_muted)
                                .child(cwd.clone()),
                        )
                        .children(launchable_task.map(|task| {
                            let task_id = task.id;
                            components::button(
                                theme,
                                "Launch Agent",
                                ButtonEmphasis::Primary,
                                task_id.as_uuid(),
                                cx,
                                move |this, cx| {
                                    this.dispatch(WorkbenchCommand::LaunchAgent(task_id), cx);
                                },
                            )
                        }))
                        .children(workspace_terminal_session_id.map(|session_id| {
                            components::button(
                                theme,
                                "Start Agent",
                                ButtonEmphasis::Secondary,
                                (
                                    gpui::ElementId::from(session_id.as_uuid()),
                                    "launch-terminal-agent",
                                ),
                                cx,
                                move |this, cx| {
                                    this.dispatch(
                                        WorkbenchCommand::LaunchAgentTerminal(session_id),
                                        cx,
                                    );
                                },
                            )
                        }))
                        .child(components::badge(theme, status, status_tone)),
                ),
        )
        .child(match view_model.pane_route {
            PaneRoute::Terminal => {
                terminal_content(theme, projection, terminal_focus, cx).into_any_element()
            }
            PaneRoute::Preview => {
                preview_content(theme, view_model.active_task(), cx).into_any_element()
            }
        })
}

fn terminal_content(
    theme: RelayTheme,
    projection: &TerminalPaneProjection,
    terminal_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let body = if projection.scrollback.is_empty() {
        if projection.connected {
            "Terminal session is connected.".to_string()
        } else if projection.session_id.is_some() {
            "Terminal session is not running.".to_string()
        } else {
            "No terminal session attached.".to_string()
        }
    } else {
        projection.scrollback.clone()
    };
    let session_id = projection.session_id;
    let can_write = projection.connected && !projection.exited;
    let focus_handle = terminal_focus.clone();

    div()
        .flex_1()
        .min_h_0()
        .bg(theme.terminal_bg)
        .flex()
        .flex_col()
        .min_w_0()
        .border_1()
        .border_color(theme.terminal_bg)
        .track_focus(terminal_focus)
        .tab_index(0)
        .cursor(CursorStyle::IBeam)
        .key_context("Terminal")
        .focus(|style| style.border_color(theme.accent_border))
        .on_key_down(cx.listener(move |this, event, _, cx| {
            if can_write
                && let Some(session_id) = session_id
                && let Some(bytes) = terminal_input_bytes(event)
            {
                this.dispatch(WorkbenchCommand::WriteTerminal(session_id, bytes), cx);
                cx.stop_propagation();
            }
        }))
        .id("terminal-input")
        .on_click(cx.listener(move |_, _: &gpui::ClickEvent, window, _| {
            window.focus(&focus_handle);
        }))
        .child(
            div()
                .id("terminal-scrollback")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .overflow_x_hidden()
                .font_family(mono_family())
                .text_color(theme.terminal_text)
                .bg(theme.terminal_bg)
                .p_4()
                .child(body),
        )
}

fn terminal_input_bytes(event: &KeyDownEvent) -> Option<Vec<u8>> {
    terminal_input_bytes_for_keystroke(event.keystroke.clone())
}

fn terminal_input_bytes_for_keystroke(keystroke: gpui::Keystroke) -> Option<Vec<u8>> {
    let keystroke = keystroke.with_simulated_ime();
    if keystroke.modifiers.platform || keystroke.modifiers.function {
        return None;
    }
    if keystroke.modifiers.control {
        return control_key_sequence(&keystroke.key);
    }

    let mut bytes = match keystroke.key.as_str() {
        "enter" => b"\r".to_vec(),
        "backspace" => b"\x7f".to_vec(),
        "tab" => b"\t".to_vec(),
        "escape" => b"\x1b".to_vec(),
        "up" => b"\x1b[A".to_vec(),
        "down" => b"\x1b[B".to_vec(),
        "right" => b"\x1b[C".to_vec(),
        "left" => b"\x1b[D".to_vec(),
        "home" => b"\x1b[H".to_vec(),
        "end" => b"\x1b[F".to_vec(),
        "delete" => b"\x1b[3~".to_vec(),
        "pageup" => b"\x1b[5~".to_vec(),
        "pagedown" => b"\x1b[6~".to_vec(),
        _ => printable_key_sequence(&keystroke)?,
    };
    if keystroke.modifiers.alt {
        bytes.insert(0, b'\x1b');
    }
    Some(bytes)
}

fn printable_key_sequence(keystroke: &gpui::Keystroke) -> Option<Vec<u8>> {
    let text = keystroke
        .key_char
        .as_deref()
        .or_else(|| printable_key_fallback(&keystroke.key))?;
    if text.chars().any(char::is_control) {
        return None;
    }
    Some(text.as_bytes().to_vec())
}

fn printable_key_fallback(key: &str) -> Option<&str> {
    if key.chars().count() == 1 {
        Some(key)
    } else {
        None
    }
}

fn control_key_sequence(key: &str) -> Option<Vec<u8>> {
    let byte = match key {
        "@" | "space" => 0x00,
        "[" | "escape" => 0x1b,
        "\\" => 0x1c,
        "]" => 0x1d,
        "^" => 0x1e,
        "_" => 0x1f,
        "?" => 0x7f,
        key if key.len() == 1 => {
            let byte = key.as_bytes()[0].to_ascii_lowercase();
            if byte.is_ascii_lowercase() {
                byte & 0x1f
            } else {
                return None;
            }
        }
        _ => return None,
    };
    Some(vec![byte])
}

/// A single Terminal/Preview route tab. Kept as a dedicated function (rather
/// than the shared `segmented_tabs`) because switching to the Terminal route
/// must also focus the terminal input, which needs the `window`.
fn route_tab(
    theme: RelayTheme,
    active_route: PaneRoute,
    route: PaneRoute,
    label: &'static str,
    terminal_focus: &FocusHandle,
    cx: &mut Context<AppShell>,
) -> impl IntoElement {
    let focus_handle = terminal_focus.clone();
    let is_active = active_route == route;
    div()
        .h_full()
        .px_3()
        .flex()
        .items_center()
        .text_sm()
        .font_weight(if is_active {
            gpui::FontWeight::SEMIBOLD
        } else {
            gpui::FontWeight::MEDIUM
        })
        .text_color(if is_active {
            theme.accent
        } else {
            theme.text_muted
        })
        .cursor_pointer()
        .hover(move |style| {
            style.text_color(if is_active {
                theme.accent
            } else {
                theme.text_secondary
            })
        })
        .border_b_2()
        .border_color(if is_active {
            theme.accent
        } else {
            gpui::transparent_black()
        })
        .id(("pane-route", route.index()))
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, window, cx| {
            this.dispatch(WorkbenchCommand::SetPaneRoute(route), cx);
            if route == PaneRoute::Terminal {
                window.focus(&focus_handle);
            }
        }))
        .child(label)
}

impl PaneRoute {
    fn index(self) -> usize {
        match self {
            Self::Terminal => 0,
            Self::Preview => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui::{KeyDownEvent, Keystroke};

    use super::*;

    #[test]
    fn terminal_input_bytes_should_encode_printable_text() {
        assert_eq!(bytes_for("a"), Some(b"a".to_vec()));
        assert_eq!(bytes_for("shift-a"), Some(b"A".to_vec()));
        assert_eq!(bytes_for("space"), Some(b" ".to_vec()));
    }

    #[test]
    fn terminal_input_bytes_should_encode_terminal_control_keys() {
        assert_eq!(bytes_for("enter"), Some(b"\r".to_vec()));
        assert_eq!(bytes_for("backspace"), Some(b"\x7f".to_vec()));
        assert_eq!(bytes_for("up"), Some(b"\x1b[A".to_vec()));
        assert_eq!(bytes_for("delete"), Some(b"\x1b[3~".to_vec()));
    }

    #[test]
    fn terminal_input_bytes_should_encode_ctrl_keys() {
        assert_eq!(bytes_for("ctrl-c"), Some(vec![0x03]));
        assert_eq!(bytes_for("ctrl-d"), Some(vec![0x04]));
        assert_eq!(bytes_for("ctrl-l"), Some(vec![0x0c]));
    }

    #[test]
    fn terminal_input_bytes_should_ignore_platform_shortcuts() {
        assert_eq!(bytes_for("cmd-c"), None);
    }

    fn bytes_for(source: &str) -> Option<Vec<u8>> {
        terminal_input_bytes(&KeyDownEvent {
            keystroke: Keystroke::parse(source).expect("keystroke should parse"),
            is_held: false,
        })
    }
}
