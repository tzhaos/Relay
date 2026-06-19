use gpui::{
    AnyElement, App, AppContext, DragMoveEvent, Empty, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, space};

/// Split direction for [`SplitPane`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone)]
struct DraggedSplitPane {
    id: &'static str,
}

type ResizeHandler = std::rc::Rc<dyn Fn(f32, &mut Window, &mut App) + 'static>;

/// A two-pane layout with a draggable divider.
#[derive(IntoElement)]
pub struct SplitPane {
    id: &'static str,
    axis: SplitAxis,
    first: AnyElement,
    second: AnyElement,
    first_size: f32,
    min_first: f32,
    min_second: f32,
    on_resize: Option<ResizeHandler>,
}

impl SplitPane {
    pub fn new(id: &'static str, first: impl IntoElement, second: impl IntoElement) -> Self {
        Self {
            id,
            axis: SplitAxis::Horizontal,
            first: first.into_any_element(),
            second: second.into_any_element(),
            first_size: space::RAIL_WIDTH,
            min_first: 220.0,
            min_second: 420.0,
            on_resize: None,
        }
    }

    pub fn axis(mut self, axis: SplitAxis) -> Self {
        self.axis = axis;
        self
    }

    pub fn first_size(mut self, first_size: f32) -> Self {
        self.first_size = first_size;
        self
    }

    pub fn min_sizes(mut self, first: f32, second: f32) -> Self {
        self.min_first = first;
        self.min_second = second;
        self
    }

    pub fn on_resize(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_resize = Some(std::rc::Rc::new(handler));
        self
    }
}

impl RenderOnce for SplitPane {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let handler = self.on_resize;
        let id = self.id;
        let axis = self.axis;
        let min_first = self.min_first;
        let min_second = self.min_second;
        let handle = split_handle(id, axis, handler.clone());

        let first = match axis {
            SplitAxis::Horizontal => div()
                .w(px(self.first_size))
                .h_full()
                .min_h_0()
                .flex_shrink_0()
                .child(self.first),
            SplitAxis::Vertical => div()
                .h(px(self.first_size))
                .w_full()
                .min_w_0()
                .flex_shrink_0()
                .child(self.first),
        };
        let second = div().flex_1().min_w_0().min_h_0().child(self.second);

        div()
            .id(id)
            .size_full()
            .min_w_0()
            .min_h_0()
            .flex()
            .when(axis == SplitAxis::Vertical, |this| this.flex_col())
            .bg(theme.app_bg)
            .child(first)
            .child(handle)
            .child(second)
            .when_some(handler, |this, handler| {
                this.on_drag_move::<DraggedSplitPane>(move |event, window, cx| {
                    if event.drag(cx).id != id {
                        return;
                    }
                    let next = split_size_from_drag(event, axis, min_first, min_second);
                    handler(next, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}

fn split_size_from_drag(
    event: &DragMoveEvent<DraggedSplitPane>,
    axis: SplitAxis,
    min_first: f32,
    min_second: f32,
) -> f32 {
    let total = match axis {
        SplitAxis::Horizontal => f32::from(event.bounds.size.width),
        SplitAxis::Vertical => f32::from(event.bounds.size.height),
    };
    let raw = match axis {
        SplitAxis::Horizontal => f32::from(event.event.position.x - event.bounds.left()),
        SplitAxis::Vertical => f32::from(event.event.position.y - event.bounds.top()),
    };
    clamp_split_size(raw, total, min_first, min_second)
}

fn split_handle(id: &'static str, axis: SplitAxis, handler: Option<ResizeHandler>) -> gpui::Div {
    let divider = match axis {
        SplitAxis::Horizontal => div().w(px(1.0)).h_full(),
        SplitAxis::Vertical => div().h(px(1.0)).w_full(),
    };
    let interactive = match axis {
        SplitAxis::Horizontal => div()
            .id(gpui::SharedString::from(format!("split-handle-{id}")))
            .absolute()
            .left(px(-3.0))
            .top_0()
            .w(px(7.0))
            .h_full()
            .cursor_col_resize(),
        SplitAxis::Vertical => div()
            .id(gpui::SharedString::from(format!("split-handle-{id}")))
            .absolute()
            .top(px(-3.0))
            .left_0()
            .h(px(7.0))
            .w_full()
            .cursor_row_resize(),
    };

    divider
        .relative()
        .flex_shrink_0()
        .bg(gpui::transparent_black())
        .child(
            interactive
                .when_some(handler, |this, _handler| {
                    this.on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .on_drag(DraggedSplitPane { id }, |_, _, _window, cx| {
                        cx.new(|_| Empty)
                    })
                    .on_drop::<DraggedSplitPane>(|_, _, cx| {
                        cx.stop_propagation();
                    })
                })
                .hover(move |style| style.bg(gpui::black().opacity(0.08))),
        )
}

fn clamp_split_size(raw: f32, total: f32, min_first: f32, min_second: f32) -> f32 {
    if total <= min_first + min_second {
        return min_first.min(total.max(0.0));
    }
    raw.clamp(min_first, total - min_second)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_size_clamps_to_minimums() {
        assert_eq!(clamp_split_size(10.0, 1000.0, 220.0, 420.0), 220.0);
    }

    #[test]
    fn split_size_clamps_to_secondary_minimum() {
        assert_eq!(clamp_split_size(900.0, 1000.0, 220.0, 420.0), 580.0);
    }
}
