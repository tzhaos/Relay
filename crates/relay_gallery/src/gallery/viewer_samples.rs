use gpui::{Entity, IntoElement, ParentElement, Styled, div, prelude::FluentBuilder, px};
use relay_ui_kit::{
    CodeView, DiffView, FileKind, FileView, MarkdownView, Segment, SegmentedControl,
};

use super::GalleryState;
use crate::GalleryApp;

pub(super) fn viewer_sample(state: &GalleryState, host: &Entity<GalleryApp>) -> impl IntoElement {
    let active = state.viewer_tab;

    div()
        .h(px(340.0))
        .flex()
        .flex_col()
        .gap_2()
        .child(
            SegmentedControl::new(
                "viewer-kind",
                vec![
                    Segment::new("code", "Code"),
                    Segment::new("markdown", "Markdown"),
                    Segment::new("diff", "Diff"),
                ],
            )
            .active(active)
            .on_select({
                let host = host.clone();
                move |key, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.gallery.viewer_tab = key;
                        cx.notify();
                    });
                }
            }),
        )
        .child(
            div()
                .flex_1()
                .min_h_0()
                .when(active == "code", |this| {
                    this.child(
                        FileView::new(
                            "crates/relay_terminal/src/session.rs",
                            FileKind::Code,
                            CodeView::new(CODE_SAMPLE).language("rust"),
                        )
                        .detail("read-only"),
                    )
                })
                .when(active == "markdown", |this| {
                    this.child(
                        FileView::new(
                            "docs/rfc-004-ui-workbench.md",
                            FileKind::Markdown,
                            MarkdownView::new(MARKDOWN_SAMPLE),
                        )
                        .detail("preview"),
                    )
                })
                .when(active == "diff", |this| {
                    this.child(
                        FileView::new(
                            "crates/relay_ui_kit/src/viewer/diff_view.rs",
                            FileKind::Diff,
                            DiffView::from_text_diff(DIFF_OLD, DIFF_NEW),
                        )
                        .detail("+4 -1"),
                    )
                }),
        )
}

const CODE_SAMPLE: &str = r#"pub struct TerminalSession {
    pub id: TerminalSessionId,
    pub cwd: PathBuf,
    pub command: String,
}

impl TerminalSession {
    pub fn attach_agent(&mut self, agent: AgentKind) {
        self.command = agent.command_name().to_string();
    }
}"#;

const MARKDOWN_SAMPLE: &str = r#"# Workbench Context

Relay keeps Terminal as the primary surface and uses file viewers as supporting context.

- Files are read-only in this layer.
- Markdown previews are lightweight.
- Diff review stays close to the active agent task.

```text
terminal -> context -> review -> agent
```
"#;

const DIFF_OLD: &str = r#"pub fn preview_body(active: &DemoTask, theme: Theme) -> impl IntoElement {
    empty_preview(active, theme)
}
"#;

const DIFF_NEW: &str = r#"pub fn preview_body(active: &DemoTask, _theme: Theme) -> impl IntoElement {
    FileView::new(
        format!("docs/tasks/{}.md", active.branch),
        FileKind::Markdown,
        MarkdownView::new(task_preview(active)),
    )
}
"#;
