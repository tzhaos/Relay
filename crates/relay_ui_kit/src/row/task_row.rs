use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    display::StatusDot,
    theme::{ActiveTheme, radius, space},
    tone::Tone,
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Metadata for one [`TaskRow`].
pub struct TaskRowData {
    pub title: String,
    pub status_label: String,
    pub status_tone: Tone,
    pub branch: Option<String>,
    pub changed: usize,
    pub review: usize,
}

/// A fixed-height task row for the left rail.
#[derive(IntoElement)]
pub struct TaskRow {
    id: ElementId,
    data: TaskRowData,
    selected: bool,
    on_click: Option<ClickHandler>,
}

impl TaskRow {
    pub fn new(id: impl Into<ElementId>, data: TaskRowData) -> Self {
        Self {
            id: id.into(),
            data,
            selected: false,
            on_click: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TaskRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let data = self.data;
        let meta = task_meta(&data);

        div()
            .id(self.id)
            .h(px(space::TASK_ROW))
            .px_2()
            .py(px(space::SM))
            .flex()
            .flex_col()
            .justify_center()
            .gap_1()
            .rounded(px(radius::MD))
            .border_1()
            .when(self.selected, |this| {
                this.bg(theme.accent_bg).border_color(theme.accent_border)
            })
            .when(!self.selected, |this| {
                this.border_color(gpui::transparent_black())
                    .cursor_pointer()
                    .hover(move |s| s.bg(theme.hover))
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(StatusDot::new(data.status_tone))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(data.title),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(data.status_tone.fg(&theme))
                            .child(data.status_label),
                    ),
            )
            .when(!meta.is_empty(), |this| {
                this.child(
                    div()
                        .pl(px(15.0))
                        .truncate()
                        .text_size(px(11.0))
                        .text_color(theme.text_muted)
                        .child(meta),
                )
            })
            .when_some(self.on_click, |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}

fn task_meta(data: &TaskRowData) -> String {
    let mut parts = Vec::new();
    if let Some(branch) = &data.branch {
        parts.push(branch.clone());
    }
    if data.changed > 0 {
        parts.push(format!("{}±", data.changed));
    }
    if data.review > 0 {
        parts.push(format!("{} review", data.review));
    }
    parts.join("  ·  ")
}
