use gpui::{
    AnyElement, App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    Badge,
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius, space},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Text,
    Code,
    Markdown,
    Diff,
}

impl FileKind {
    fn label(self) -> &'static str {
        match self {
            Self::Text => "TEXT",
            Self::Code => "CODE",
            Self::Markdown => "MD",
            Self::Diff => "DIFF",
        }
    }

    fn icon(self) -> IconName {
        match self {
            Self::Diff => IconName::FileDiff,
            Self::Text | Self::Code | Self::Markdown => IconName::FileText,
        }
    }
}

#[derive(IntoElement)]
pub struct FileView {
    path: String,
    kind: FileKind,
    detail: Option<String>,
    body: AnyElement,
}

impl FileView {
    pub fn new(path: impl Into<String>, kind: FileKind, body: impl IntoElement) -> Self {
        Self {
            path: path.into(),
            kind,
            detail: None,
            body: body.into_any_element(),
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

impl RenderOnce for FileView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();

        div()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .rounded(px(radius::LG))
            .border_1()
            .border_color(theme.border)
            .bg(theme.panel)
            .overflow_hidden()
            .child(
                div()
                    .h(px(space::PANE_HEADER))
                    .px_3()
                    .flex()
                    .items_center()
                    .gap_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .bg(theme.chrome)
                    .child(
                        Icon::new(self.kind.icon())
                            .size(IconSize::Small)
                            .color(theme.text_secondary),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .truncate()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(self.path),
                    )
                    .when_some(self.detail, |this, detail| {
                        this.child(
                            div()
                                .max_w(px(220.0))
                                .truncate()
                                .text_size(px(11.0))
                                .text_color(theme.text_muted)
                                .child(detail),
                        )
                    })
                    .child(Badge::new(self.kind.label()).soft()),
            )
            .child(div().flex_1().min_h_0().child(self.body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_kind_labels_are_compact() {
        assert_eq!(FileKind::Markdown.label(), "MD");
    }
}
