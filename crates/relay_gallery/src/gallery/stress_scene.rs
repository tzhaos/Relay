use gpui::{Context, Entity, IntoElement, ParentElement, Styled, div, px};
use relay_ui_kit::{
    Button, ButtonVariant, CodeView, FileKind, FileView, IconName, TaskRow, TaskRowData,
    TerminalLine, TerminalLineStyle, TerminalSurface, Theme, Tone, TreeRow,
};

use super::{
    GalleryState,
    shared::{scene_stack, section, strip},
};
use crate::GalleryApp;

pub(super) fn render(
    _state: &GalleryState,
    _host: &Entity<GalleryApp>,
    theme: Theme,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    scene_stack()
        .child(section(
            cx,
            "Long text",
            div()
                .flex()
                .items_start()
                .gap_3()
                .flex_wrap()
                .child(long_task_rows())
                .child(long_file_tree()),
        ))
        .child(section(
            cx,
            "Disabled and quiet states",
            strip()
                .child(
                    Button::new("stress-disabled-primary", "Launch Agent")
                        .primary()
                        .icon(IconName::Play)
                        .disabled(true),
                )
                .child(
                    Button::new("stress-disabled-secondary", "Archive")
                        .variant(ButtonVariant::Secondary)
                        .icon(IconName::Archive)
                        .disabled(true),
                )
                .child(
                    Button::new("stress-disabled-ghost", "Refresh")
                        .ghost()
                        .icon(IconName::RefreshCw)
                        .disabled(true),
                ),
        ))
        .child(section(
            cx,
            "Terminal scrollback",
            div()
                .h(px(280.0))
                .border_1()
                .border_color(theme.border)
                .rounded(px(relay_ui_kit::radius::LG))
                .overflow_hidden()
                .child(TerminalSurface::new(stress_terminal_lines()).prompt("relay>")),
        ))
        .child(section(
            cx,
            "Code overflow",
            div().h(px(260.0)).child(
                FileView::new(
                    "crates/relay_gallery/src/gallery/stress_scene.rs",
                    FileKind::Code,
                    CodeView::new(STRESS_CODE).language("rust"),
                )
                .detail("long line"),
            ),
        ))
}

fn long_task_rows() -> impl IntoElement {
    div()
        .w(px(420.0))
        .flex()
        .flex_col()
        .gap_1()
        .child(TaskRow::new(
            "stress-task-long",
            TaskRowData {
                title: "Repair terminal focus after switching between a Codex session and a plain shell in a nested worktree".into(),
                status_label: "RUNNING".into(),
                status_tone: Tone::Accent,
                branch: Some("feature/terminal-focus-after-agent-switching".into()),
                changed: 128,
                review: 12,
            },
        ).selected(true))
        .child(TaskRow::new(
            "stress-task-muted",
            TaskRowData {
                title: "Check long review note delivery state".into(),
                status_label: "WAITING".into(),
                status_tone: Tone::Warning,
                branch: Some("review/very-long-review-delivery-state".into()),
                changed: 42,
                review: 9,
            },
        ))
}

fn long_file_tree() -> impl IntoElement {
    div()
        .w(px(420.0))
        .flex()
        .flex_col()
        .child(
            TreeRow::new("stress-tree-root", IconName::Folder, "crates")
                .expandable(true)
                .depth(0),
        )
        .child(
            TreeRow::new(
                "stress-tree-deep",
                IconName::Folder,
                "relay_ui_kit/src/terminal/session/history/very/deep/path",
            )
            .depth(1),
        )
        .child(
            TreeRow::new(
                "stress-tree-file",
                IconName::FileText,
                "terminal_session_history_projection_with_extremely_long_name.rs",
            )
            .depth(2)
            .selected(true),
        )
        .child(TreeRow::new(
            "stress-tree-diff",
            IconName::FileDiff,
            "workbench/context/diff/review/comment_delivery.rs",
        ))
}

fn stress_terminal_lines() -> Vec<TerminalLine> {
    (0..14)
        .map(|index| {
            let style = match index % 4 {
                0 => TerminalLineStyle::Input,
                1 => TerminalLineStyle::Output,
                2 => TerminalLineStyle::Success,
                _ => TerminalLineStyle::Muted,
            };
            TerminalLine::new(format!(
                "line {index:02}: terminal scrollback keeps row height stable while content changes"
            ))
            .style(style)
        })
        .collect()
}

const STRESS_CODE: &str = r#"pub fn absurdly_long_terminal_command_preview() {
    let command = "codex --worktree F:/Workspace/Relay/.worktrees/terminal-focus-after-agent-switching --agent codex --review-diff --keep-terminal-focus --really-long-argument-name";
    println!("{command}");
}
"#;
