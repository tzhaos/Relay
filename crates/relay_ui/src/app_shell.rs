use gpui::{
    App, Bounds, Context, IntoElement, Render, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, size,
};

use crate::theme::RelayTheme;

pub struct AppShell {
    theme: RelayTheme,
    tasks: Vec<TaskRow>,
}

#[derive(Clone)]
struct TaskRow {
    title: &'static str,
    status: &'static str,
    meta: &'static str,
}

impl AppShell {
    pub fn open(cx: &mut App) -> anyhow::Result<()> {
        let bounds = Bounds::centered(None, size(px(1320.0), px(820.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(Default::default()),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Self::new()),
        )?;
        cx.activate(true);
        Ok(())
    }

    fn new() -> Self {
        Self {
            theme: RelayTheme::dark(),
            tasks: vec![
                TaskRow {
                    title: "Design GPUI shell",
                    status: "WORKING",
                    meta: "relay-ui / main",
                },
                TaskRow {
                    title: "Codex provider spike",
                    status: "WAITING",
                    meta: "relay-agent / task-2",
                },
                TaskRow {
                    title: "Diff review model",
                    status: "DONE",
                    meta: "relay-diff / task-3",
                },
            ],
        }
    }

    fn header(&self) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(44.0))
            .px_4()
            .border_b_1()
            .border_color(self.theme.line)
            .bg(self.theme.panel)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(self.theme.text)
                            .child("Relay"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(self.theme.muted)
                            .child("Rust-native agent workbench"),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(self.theme.muted)
                    .child("Zed-like surface / Orca-like workflow"),
            )
    }

    fn task_list(&self) -> impl IntoElement {
        let mut list = div().flex().flex_col().gap_2();
        for task in &self.tasks {
            list = list.child(self.task_row(task));
        }

        div()
            .w(px(286.0))
            .h_full()
            .border_r_1()
            .border_color(self.theme.line)
            .bg(self.theme.panel)
            .p_3()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .text_color(self.theme.muted)
                    .child("TASKS / WORKTREES"),
            )
            .child(list)
    }

    fn task_row(&self, task: &TaskRow) -> impl IntoElement {
        let status_color = match task.status {
            "WORKING" => self.theme.accent,
            "WAITING" => self.theme.warning,
            "DONE" => self.theme.muted,
            _ => self.theme.text,
        };

        div()
            .rounded_md()
            .border_1()
            .border_color(self.theme.line)
            .bg(self.theme.panel_alt)
            .p_3()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_color(self.theme.text)
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(task.title),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(div().text_sm().text_color(self.theme.muted).child(task.meta))
                    .child(
                        div()
                            .text_xs()
                            .text_color(status_color)
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(task.status),
                    ),
            )
    }

    fn terminal_pane(&self) -> impl IntoElement {
        div()
            .flex_1()
            .h_full()
            .bg(self.theme.bg)
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .text_color(self.theme.muted)
                    .child("TERMINAL"),
            )
            .child(
                div()
                    .flex_1()
                    .rounded_md()
                    .border_1()
                    .border_color(self.theme.line)
                    .bg(self.theme.panel)
                    .p_4()
                    .text_color(self.theme.text)
                    .child("relay $ claude")
                    .child(div().mt_2().text_color(self.theme.muted).child(
                        "Native CLI agent will run here via PTY.",
                    ))
                    .child(div().mt_4().text_color(self.theme.accent).child(
                        "Step 1 target: GPUI shell + pane layout.",
                    )),
            )
    }

    fn context_pane(&self) -> impl IntoElement {
        div()
            .w(px(360.0))
            .h_full()
            .border_l_1()
            .border_color(self.theme.line)
            .bg(self.theme.panel)
            .p_3()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .text_color(self.theme.muted)
                    .child("FILES / DIFF / REVIEW"),
            )
            .child(self.context_row("Changed files", "3"))
            .child(self.context_row("Review comments", "0"))
            .child(self.context_row("Preview targets", "0"))
            .child(
                div()
                    .mt_4()
                    .rounded_md()
                    .border_1()
                    .border_color(self.theme.line)
                    .bg(self.theme.panel_alt)
                    .p_3()
                    .text_color(self.theme.muted)
                    .child("Diff viewer and review loop attach here in Step 8."),
            )
    }

    fn context_row(&self, label: &'static str, value: &'static str) -> impl IntoElement {
        div()
            .flex()
            .justify_between()
            .items_center()
            .border_b_1()
            .border_color(self.theme.line)
            .py_2()
            .child(div().text_color(self.theme.muted).child(label))
            .child(
                div()
                    .text_color(self.theme.text)
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(value),
            )
    }
}

impl Render for AppShell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(self.theme.bg)
            .flex()
            .flex_col()
            .child(self.header())
            .child(
                div()
                    .flex()
                    .flex_1()
                    .child(self.task_list())
                    .child(self.terminal_pane())
                    .child(self.context_pane()),
            )
    }
}
