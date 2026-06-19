use gpui::{
    AnyElement, App, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    Styled, Window, WindowControlArea, div, prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, radius, space};

/// A client-side title bar for windows opened without native decorations.
#[derive(IntoElement)]
pub struct TitleBar {
    app_name: String,
    project_label: Option<String>,
    center: Option<AnyElement>,
    actions: Option<AnyElement>,
    show_window_controls: bool,
}

impl TitleBar {
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            project_label: None,
            center: None,
            actions: None,
            show_window_controls: true,
        }
    }

    pub fn project(mut self, project_label: impl Into<String>) -> Self {
        self.project_label = Some(project_label.into());
        self
    }

    pub fn center(mut self, center: impl IntoElement) -> Self {
        self.center = Some(center.into_any_element());
        self
    }

    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions = Some(actions.into_any_element());
        self
    }

    pub fn window_controls(mut self, show: bool) -> Self {
        self.show_window_controls = show;
        self
    }
}

impl RenderOnce for TitleBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();

        div()
            .h(px(space::TITLE_BAR))
            .flex_shrink_0()
            .w_full()
            .flex()
            .items_center()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.chrome)
            .child(
                div()
                    .w(px(space::RAIL_WIDTH))
                    .h_full()
                    .min_w_0()
                    .flex_shrink_0()
                    .px_3()
                    .flex()
                    .items_center()
                    .gap_2()
                    .window_control_area(WindowControlArea::Drag)
                    .child(
                        div()
                            .size(px(22.0))
                            .rounded(px(radius::MD))
                            .bg(theme.accent)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme.on_accent)
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .child("R"),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text)
                                    .child(self.app_name),
                            )
                            .when_some(self.project_label, |this, project_label| {
                                this.child(
                                    div()
                                        .min_w_0()
                                        .truncate()
                                        .text_sm()
                                        .text_color(theme.text_muted)
                                        .child(project_label),
                                )
                            }),
                    ),
            )
            .child(
                div()
                    .h_full()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .window_control_area(WindowControlArea::Drag)
                    .when_some(self.center, |this, center| this.child(center)),
            )
            .child(
                div()
                    .h_full()
                    .flex_shrink_0()
                    .pl_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when_some(self.actions, |this, actions| this.child(actions))
                    .when(self.show_window_controls, |this| {
                        this.child(WindowControls::new())
                    }),
            )
    }
}

/// Native hit-test-backed window controls for client-decorated windows.
#[derive(IntoElement)]
pub struct WindowControls;

impl WindowControls {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowControls {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for WindowControls {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let maximize_label = if window.is_maximized() { "[]" } else { "[ ]" };

        div()
            .h_full()
            .flex()
            .items_center()
            .child(window_control_button(
                theme,
                WindowControlArea::Min,
                "-",
                false,
            ))
            .child(window_control_button(
                theme,
                WindowControlArea::Max,
                maximize_label,
                false,
            ))
            .child(window_control_button(
                theme,
                WindowControlArea::Close,
                "x",
                true,
            ))
    }
}

fn window_control_button(
    theme: crate::Theme,
    area: WindowControlArea,
    label: &'static str,
    danger: bool,
) -> gpui::Div {
    div()
        .w(px(44.0))
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(13.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.text_muted)
        .window_control_area(area)
        .hover(move |style| {
            if danger {
                style.bg(theme.danger).text_color(gpui::white())
            } else {
                style.bg(theme.hover).text_color(theme.text)
            }
        })
        .child(label)
}
