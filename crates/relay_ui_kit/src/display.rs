//! Display primitives: [`Badge`], [`StatusDot`], [`Divider`], and [`EmptyState`].
//!
//! These are pure `RenderOnce` builders with no interactivity — they read the
//! active [`Theme`] and a [`Tone`] and emit a styled element. They replace the
//! scattered `*_state_badge` / `status_dot` / `empty_state` copies that used to
//! live in each pane.

use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius, space},
    tone::Tone,
};

// ---------------------------------------------------------------------------
// Badge — a compact label chip.
// ---------------------------------------------------------------------------

/// Fill treatment for a [`Badge`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeStyle {
    /// Soft tinted fill in the tone's color family.
    Soft,
    /// Quiet neutral fill with a 1px border (the default for metadata chips).
    Outline,
}

/// A small label chip. `tone` drives the foreground color; `style` selects a
/// soft tinted fill or a quiet outlined fill.
#[derive(IntoElement)]
pub struct Badge {
    label: String,
    tone: Tone,
    style: BadgeStyle,
    icon: Option<IconName>,
}

impl Badge {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            tone: Tone::Muted,
            style: BadgeStyle::Outline,
            icon: None,
        }
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn soft(mut self) -> Self {
        self.style = BadgeStyle::Soft;
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = self.tone.fg(&theme);
        let (bg, border) = match self.style {
            BadgeStyle::Soft => (self.tone.soft_bg(&theme), self.tone.soft_border(&theme)),
            BadgeStyle::Outline => (theme.panel_alt, theme.border),
        };

        div()
            .h(px(20.0))
            .px(px(space::SM))
            .rounded(px(radius::SM))
            .border_1()
            .border_color(border)
            .bg(bg)
            .flex()
            .items_center()
            .gap_1()
            .text_color(fg)
            .text_size(px(11.0))
            .font_weight(FontWeight::MEDIUM)
            .when_some(self.icon, |this, icon| {
                this.child(Icon::new(icon).size(IconSize::XSmall).color(fg))
            })
            .child(self.label)
    }
}

// ---------------------------------------------------------------------------
// Status dot — a small circular state indicator.
// ---------------------------------------------------------------------------

/// A small circular status indicator (agent / task / connection state).
#[derive(IntoElement)]
pub struct StatusDot {
    tone: Tone,
    size: f32,
}

impl StatusDot {
    pub fn new(tone: Tone) -> Self {
        Self { tone, size: 7.0 }
    }

    pub fn size(mut self, px: f32) -> Self {
        self.size = px;
        self
    }
}

impl RenderOnce for StatusDot {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self.tone.fg(cx.theme());
        div().size(px(self.size)).rounded_full().bg(color)
    }
}

// ---------------------------------------------------------------------------
// Divider — a 1px hairline.
// ---------------------------------------------------------------------------

/// A 1px hairline divider. Defaults to horizontal; call [`Divider::vertical`]
/// for a vertical rule.
#[derive(IntoElement)]
pub struct Divider {
    vertical: bool,
}

impl Divider {
    pub fn horizontal() -> Self {
        Self { vertical: false }
    }

    pub fn vertical() -> Self {
        Self { vertical: true }
    }
}

impl RenderOnce for Divider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = cx.theme().border;
        if self.vertical {
            div().w(px(1.0)).h_full().bg(color)
        } else {
            div().h(px(1.0)).w_full().bg(color)
        }
    }
}

// ---------------------------------------------------------------------------
// Empty state — compact and operational.
// ---------------------------------------------------------------------------

/// A compact operational empty state. `title` states the situation; `detail`
/// gives an actionable next step rather than a literal "no X stored" string.
#[derive(IntoElement)]
pub struct EmptyState {
    title: String,
    detail: String,
    icon: Option<IconName>,
}

impl EmptyState {
    pub fn new(title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            detail: detail.into(),
            icon: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl RenderOnce for EmptyState {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap_1()
            .px(px(space::LG))
            .py(px(space::XL))
            .text_center()
            .when_some(self.icon, |this, icon| {
                this.child(
                    div().mb_1().child(
                        Icon::new(icon)
                            .size(IconSize::Large)
                            .color(theme.text_muted),
                    ),
                )
            })
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text_secondary)
                    .child(self.title),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child(self.detail),
            )
    }
}
