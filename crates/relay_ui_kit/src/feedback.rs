//! Feedback components for loading, progress, errors, and transient notices.

use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, App, ElementId, FontWeight, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, Styled, Transformation, Window, div, percentage,
    prelude::FluentBuilder, px, relative, svg,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    motion::{MotionDirection, MotionExt},
    theme::{ActiveTheme, radius},
    tone::Tone,
};

/// An inline spinner for indeterminate work.
#[derive(IntoElement)]
pub struct LoadingSpinner {
    id: ElementId,
    label: Option<String>,
    tone: Tone,
}

impl LoadingSpinner {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            tone: Tone::Muted,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
}

impl RenderOnce for LoadingSpinner {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = self.tone.fg(&theme);
        let id = self.id.clone();
        let glyph = svg()
            .path(IconName::RefreshCw.path())
            .size(px(14.0))
            .text_color(fg)
            .with_animation(
                (id, "loading-spinner"),
                Animation::new(Duration::from_millis(800)).repeat(),
                |this, delta| this.with_transformation(Transformation::rotate(percentage(delta))),
            );

        div()
            .h(px(24.0))
            .flex()
            .items_center()
            .gap_1()
            .text_color(theme.text_secondary)
            .child(glyph)
            .when_some(self.label, |this, label| {
                this.child(div().text_xs().child(label))
            })
    }
}

/// A determinate horizontal progress bar.
#[derive(IntoElement)]
pub struct ProgressBar {
    id: ElementId,
    value: f32,
    max: f32,
    tone: Tone,
    label: Option<String>,
}

impl ProgressBar {
    pub fn new(id: impl Into<ElementId>, value: f32, max: f32) -> Self {
        Self {
            id: id.into(),
            value,
            max,
            tone: Tone::Accent,
            label: None,
        }
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn ratio(&self) -> f32 {
        progress_ratio(self.value, self.max)
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let ratio = self.ratio();
        let fg = self.tone.fg(&theme);
        let bar = div()
            .id(self.id)
            .h(px(6.0))
            .w_full()
            .rounded_full()
            .bg(theme.panel_alt)
            .border_1()
            .border_color(theme.border)
            .child(div().h_full().rounded_full().bg(fg).w(relative(ratio)));

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_1()
            .when_some(self.label, |this, label| {
                this.child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text_secondary)
                        .child(label),
                )
            })
            .child(bar)
    }
}

/// A subtle placeholder block for loading rows and panes.
#[derive(IntoElement)]
pub struct Skeleton {
    id: ElementId,
    width: f32,
    height: f32,
    animated: bool,
}

impl Skeleton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            width: 160.0,
            height: 12.0,
            animated: true,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }
}

impl RenderOnce for Skeleton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let base = div()
            .id(self.id)
            .w(px(self.width))
            .h(px(self.height))
            .rounded(px(radius::SM))
            .bg(theme.panel_alt)
            .border_1()
            .border_color(theme.border);

        if self.animated {
            base.motion_pulse(0.45, 0.86).into_any_element()
        } else {
            base.into_any_element()
        }
    }
}

/// A compact inline error message for forms and terminal launch failures.
#[derive(IntoElement)]
pub struct InlineError {
    message: String,
    detail: Option<String>,
}

impl InlineError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            detail: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

impl RenderOnce for InlineError {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .flex()
            .items_start()
            .gap_2()
            .text_color(theme.danger)
            .child(
                div().pt(px(1.0)).child(
                    Icon::new(IconName::X)
                        .size(IconSize::Small)
                        .color(theme.danger),
                ),
            )
            .child(
                div()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(1.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.message),
                    )
                    .when_some(self.detail, |this, detail| {
                        this.child(div().text_xs().text_color(theme.text_muted).child(detail))
                    }),
            )
    }
}

/// A full-width operational notice with optional action content.
#[derive(IntoElement)]
pub struct Banner {
    title: String,
    detail: Option<String>,
    tone: Tone,
    action: Option<AnyElement>,
}

impl Banner {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            detail: None,
            tone: Tone::Info,
            action: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl RenderOnce for Banner {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = self.tone.fg(&theme);
        div()
            .w_full()
            .min_h(px(40.0))
            .px_3()
            .py_2()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .border_1()
            .border_color(self.tone.soft_border(&theme))
            .bg(self.tone.soft_bg(&theme))
            .child(
                Icon::new(tone_icon(self.tone))
                    .size(IconSize::Small)
                    .color(fg),
            )
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(1.0))
                    .child(
                        div()
                            .truncate()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text)
                            .child(self.title),
                    )
                    .when_some(self.detail, |this, detail| {
                        this.child(
                            div()
                                .truncate()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child(detail),
                        )
                    }),
            )
            .when_some(self.action, |this, action| this.child(action))
    }
}

/// A compact floating notification body.
#[derive(IntoElement)]
pub struct Toast {
    id: ElementId,
    title: String,
    detail: Option<String>,
    tone: Tone,
    animated: bool,
}

impl Toast {
    pub fn new(id: impl Into<ElementId>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            detail: None,
            tone: Tone::Info,
            animated: true,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }
}

impl RenderOnce for Toast {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = self.tone.fg(&theme);
        let base = div()
            .id(self.id)
            .w(px(320.0))
            .px_3()
            .py_2()
            .flex()
            .items_start()
            .gap_2()
            .rounded(px(radius::LG))
            .border_1()
            .border_color(theme.border_strong)
            .bg(theme.panel)
            .shadow_lg()
            .child(
                div().pt(px(1.0)).child(
                    Icon::new(tone_icon(self.tone))
                        .size(IconSize::Small)
                        .color(fg),
                ),
            )
            .child(
                div()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(1.0))
                    .child(
                        div()
                            .truncate()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text)
                            .child(self.title),
                    )
                    .when_some(self.detail, |this, detail| {
                        this.child(div().text_xs().text_color(theme.text_muted).child(detail))
                    }),
            );

        if self.animated {
            base.motion_slide_in(MotionDirection::FromTop, true)
                .into_any_element()
        } else {
            base.into_any_element()
        }
    }
}

fn tone_icon(tone: Tone) -> IconName {
    match tone {
        Tone::Accent => IconName::Check,
        Tone::Warning => IconName::CircleDot,
        Tone::Danger => IconName::X,
        Tone::Info | Tone::Muted | Tone::Secondary => IconName::CircleDot,
    }
}

fn progress_ratio(value: f32, max: f32) -> f32 {
    if max <= 0.0 {
        return 0.0;
    }

    (value / max).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_ratio_clamps_overflow() {
        assert_eq!(progress_ratio(12.0, 10.0), 1.0);
    }

    #[test]
    fn progress_ratio_handles_zero_max() {
        assert_eq!(progress_ratio(10.0, 0.0), 0.0);
    }
}
