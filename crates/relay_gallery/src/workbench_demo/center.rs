use gpui::{Entity, IntoElement, ParentElement, Styled, Window, div, prelude::FluentBuilder, px};
use relay_ui_kit::{
    AgentQuickLaunch, Button, FileKind, FileView, IconButton, IconName, LauncherItem,
    LauncherItemKind, LauncherMenu, MarkdownView, Pane, PaneSurface, PaneWidth, PanelHeader,
    Segment, SegmentedControl, TerminalStatusBadge, TerminalSurface, TerminalTab, TerminalToolbar,
    Theme,
    theme::{self},
};

use super::{
    WorkbenchState,
    data::{
        DEMO_SESSIONS, DemoTask, active_session, active_task, prompt_prefix, session_index_for_key,
        session_lines,
    },
};
use crate::GalleryApp;

pub(super) fn center_pane(
    state: &WorkbenchState,
    host: &Entity<GalleryApp>,
    theme: Theme,
) -> impl IntoElement {
    let active = active_task(state);
    let route = state.route;
    let header = PanelHeader::new(active.title)
        .icon(IconName::Terminal)
        .trailing(route_switch(host, route));

    let body = div()
        .size_full()
        .flex()
        .flex_col()
        .when(route == "terminal", |this| {
            this.child(terminal_body(state, host, theme))
        })
        .when(route == "preview", |this| this.child(preview_body(active)));

    Pane::new(PaneWidth::Flex, body)
        .surface(PaneSurface::Panel)
        .header(header)
}

fn route_switch(host: &Entity<GalleryApp>, route: &'static str) -> impl IntoElement {
    SegmentedControl::new(
        "route",
        vec![
            Segment::new("terminal", "Terminal"),
            Segment::new("preview", "Preview"),
        ],
    )
    .active(route)
    .on_select({
        let host = host.clone();
        move |key, _window, cx| {
            host.update(cx, |this, cx| {
                this.workbench.route = key;
                cx.notify();
            });
        }
    })
}

fn terminal_body(
    state: &WorkbenchState,
    host: &Entity<GalleryApp>,
    theme: Theme,
) -> impl IntoElement {
    let session = active_session(state);

    div()
        .size_full()
        .min_h_0()
        .flex()
        .flex_col()
        .child(terminal_toolbar(state, host))
        .child(agent_quick_launches(host, theme))
        .child(
            div()
                .h(px(30.0))
                .px_3()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .border_b_1()
                .border_color(theme.border)
                .bg(theme.panel_alt)
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .font_family(theme::mono_family())
                        .text_size(px(12.0))
                        .text_color(theme.text_secondary)
                        .child(session.cwd),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .font_family(theme::mono_family())
                        .text_size(px(12.0))
                        .text_color(theme.text_muted)
                        .child(session.command),
                )
                .child(TerminalStatusBadge::new(session.tone)),
        )
        .child(
            TerminalSurface::new(session_lines(session))
                .prompt(format!("{} {}", prompt_prefix(session), session.cwd))
                .connected(true),
        )
}

fn terminal_toolbar(state: &WorkbenchState, host: &Entity<GalleryApp>) -> impl IntoElement {
    let mut toolbar = TerminalToolbar::new();
    for (index, session) in DEMO_SESSIONS.iter().enumerate() {
        let host = host.clone();
        toolbar = toolbar.tab(
            TerminalTab::new(("terminal-tab", index), session.label)
                .active(index == state.active_session)
                .status(session.tone)
                .on_click(move |_event, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.workbench.active_session = index;
                        this.workbench.launcher_open = false;
                        cx.notify();
                    });
                }),
        );
    }

    toolbar.actions(terminal_actions(state, host))
}

fn terminal_actions(state: &WorkbenchState, host: &Entity<GalleryApp>) -> impl IntoElement {
    div()
        .relative()
        .flex()
        .items_center()
        .gap_1()
        .child(
            Button::new("terminal-new", "New")
                .icon(IconName::Plus)
                .on_click({
                    let host = host.clone();
                    move |_event, _window, cx| {
                        host.update(cx, |this, cx| {
                            this.workbench.launcher_open = !this.workbench.launcher_open;
                            cx.notify();
                        });
                    }
                }),
        )
        .child(
            IconButton::new("terminal-refresh", IconName::RefreshCw).on_click({
                let host = host.clone();
                move |_event, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.workbench.launcher_open = false;
                        cx.notify();
                    });
                }
            }),
        )
        .when(state.launcher_open, |this| {
            this.child(
                div()
                    .absolute()
                    .top(px(34.0))
                    .right(px(0.0))
                    .child(launcher_menu(host)),
            )
        })
}

fn launcher_menu(host: &Entity<GalleryApp>) -> impl IntoElement {
    LauncherMenu::new(
        "terminal-launcher",
        vec![
            LauncherItem::new("powershell", "New PowerShell", IconName::Terminal)
                .detail("Open a shell in the current workspace")
                .kind(LauncherItemKind::Terminal),
            LauncherItem::new("codex", "Codex Agent", IconName::Bot)
                .detail("Attach Codex to the active task terminal")
                .kind(LauncherItemKind::Agent),
            LauncherItem::new("claude", "Claude Agent", IconName::Bot)
                .detail("Attach Claude to the active task terminal")
                .kind(LauncherItemKind::Agent),
        ],
    )
    .on_select({
        let host = host.clone();
        move |key, _window, cx| {
            host.update(cx, |this, cx| {
                this.workbench.active_session = session_index_for_key(key);
                this.workbench.route = "terminal";
                this.workbench.launcher_open = false;
                cx.notify();
            });
        }
    })
}

fn agent_quick_launches(host: &Entity<GalleryApp>, theme: Theme) -> impl IntoElement {
    let launch = |key: &'static str, host: &Entity<GalleryApp>| {
        let host = host.clone();
        move |_event: &gpui::ClickEvent, _window: &mut Window, cx: &mut gpui::App| {
            host.update(cx, |this, cx| {
                this.workbench.active_session = session_index_for_key(key);
                this.workbench.route = "terminal";
                this.workbench.launcher_open = false;
                cx.notify();
            });
        }
    };

    div()
        .h(px(40.0))
        .px_2()
        .border_b_1()
        .border_color(theme.border)
        .bg(theme.chrome)
        .flex()
        .items_center()
        .gap_2()
        .child(
            AgentQuickLaunch::new("quick-codex", "Codex", "codex").on_click(launch("codex", host)),
        )
        .child(
            AgentQuickLaunch::new("quick-claude", "Claude", "claude")
                .on_click(launch("claude", host)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_xs()
                .text_color(theme.text_muted)
                .child("Shortcuts select a session in this gallery sample."),
        )
}

fn preview_body(active: &DemoTask) -> impl IntoElement {
    div().size_full().p_2().child(FileView::new(
        format!("docs/tasks/{}.md", active.branch.replace('/', "-")),
        FileKind::Markdown,
        MarkdownView::new(task_preview(active)),
    ))
}

fn task_preview(active: &DemoTask) -> String {
    format!(
        r#"# {}

Branch: `{}`

Worktree: `{}`

- Status: {}
- Changed files: {}
- Review notes: {}

```text
terminal session: {}
```
"#,
        active.title,
        active.branch,
        active.worktree,
        active.status,
        active.changed,
        active.review,
        active.session_key
    )
}
