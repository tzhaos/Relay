use gpui::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, radius};

/// One labelled segment in a [`SegmentedControl`].
pub struct Segment {
    pub key: &'static str,
    pub label: &'static str,
}

impl Segment {
    pub fn new(key: &'static str, label: &'static str) -> Self {
        Self { key, label }
    }
}

type SelectHandler = Box<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;

/// A pill-grouped segmented control.
#[derive(IntoElement)]
pub struct SegmentedControl {
    id: ElementId,
    segments: Vec<Segment>,
    active: &'static str,
    on_select: Option<SelectHandler>,
}

impl SegmentedControl {
    pub fn new(id: impl Into<ElementId>, segments: Vec<Segment>) -> Self {
        Self {
            id: id.into(),
            segments,
            active: "",
            on_select: None,
        }
    }

    pub fn active(mut self, active: &'static str) -> Self {
        self.active = active;
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SegmentedControl {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let handler = self.on_select.map(std::rc::Rc::new);
        let active = self.active;
        let mut row = div()
            .id(self.id)
            .h(px(28.0))
            .p(px(2.0))
            .flex()
            .items_center()
            .gap(px(2.0))
            .rounded(px(radius::MD))
            .bg(theme.inset)
            .border_1()
            .border_color(theme.border);

        for (index, segment) in self.segments.into_iter().enumerate() {
            let is_active = segment.key == active;
            let key = segment.key;
            let handler = handler.clone();
            let cell = div()
                .id(("segment", index))
                .h_full()
                .px_3()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(radius::SM))
                .text_xs()
                .font_weight(if is_active {
                    gpui::FontWeight::SEMIBOLD
                } else {
                    gpui::FontWeight::MEDIUM
                })
                .when(is_active, |this| {
                    this.bg(theme.panel).text_color(theme.text)
                })
                .when(!is_active, |this| {
                    this.text_color(theme.text_muted)
                        .cursor_pointer()
                        .hover(move |style| style.text_color(theme.text_secondary))
                })
                .when_some(handler.filter(|_| !is_active), |this, handler| {
                    this.on_click(move |_: &ClickEvent, window, cx| {
                        handler(key, window, cx);
                        cx.stop_propagation();
                    })
                })
                .child(segment.label);
            row = row.child(cell);
        }
        row
    }
}
