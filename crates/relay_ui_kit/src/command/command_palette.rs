use gpui::{
    AnyElement, App, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    display::EmptyState,
    icon::IconName,
    theme::{ActiveTheme, radius, space},
};

/// A floating command palette panel.
#[derive(IntoElement)]
pub struct CommandPalette {
    title: String,
    query: Option<AnyElement>,
    rows: Vec<AnyElement>,
    footer: Option<AnyElement>,
}

impl CommandPalette {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            query: None,
            rows: Vec::new(),
            footer: None,
        }
    }

    pub fn query(mut self, query: impl IntoElement) -> Self {
        self.query = Some(query.into_any_element());
        self
    }

    pub fn row(mut self, row: impl IntoElement) -> Self {
        self.rows.push(row.into_any_element());
        self
    }

    pub fn rows(mut self, rows: impl IntoIterator<Item = AnyElement>) -> Self {
        self.rows.extend(rows);
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }
}

impl RenderOnce for CommandPalette {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let has_rows = !self.rows.is_empty();

        div()
            .w(px(560.0))
            .max_h(px(620.0))
            .rounded(px(radius::LG))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border_strong)
            .shadow_lg()
            .occlude()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(space::PANE_HEADER))
                    .px_3()
                    .flex()
                    .items_center()
                    .border_b_1()
                    .border_color(theme.border)
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text)
                    .child(self.title),
            )
            .when_some(self.query, |this, query| {
                this.child(
                    div()
                        .p_2()
                        .border_b_1()
                        .border_color(theme.border)
                        .child(query),
                )
            })
            .child(
                div()
                    .id("command-palette-list")
                    .flex_1()
                    .min_h(px(0.0))
                    .p(px(space::XS))
                    .overflow_y_scroll()
                    .when(has_rows, |this| this.children(self.rows))
                    .when(!has_rows, |this| {
                        this.child(
                            EmptyState::new("No commands", "Try a different query.")
                                .icon(IconName::Search),
                        )
                    }),
            )
            .when_some(self.footer, |this, footer| {
                this.child(
                    div()
                        .px_3()
                        .py_2()
                        .border_t_1()
                        .border_color(theme.border)
                        .child(footer),
                )
            })
    }
}
