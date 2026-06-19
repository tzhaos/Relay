//! Interactive surface controls: [`SegmentedControl`], [`SearchField`], and
//! [`PanelHeader`].
//!
//! These are the Orca-style chrome pieces — the Files/Diff/Review segmented bar,
//! the right-pane file filter, and the 40px header strip atop each pane. Like the
//! buttons, they carry generic callbacks and read the theme from `App`, so they
//! drop into the gallery and the real workbench unchanged.

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FocusHandle, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius, space},
};

// ---------------------------------------------------------------------------
// Segmented control — Files / Diff / Review, Terminal / Preview.
// ---------------------------------------------------------------------------

/// One labelled segment in a [`SegmentedControl`]. `key` is the stable string the
/// caller maps back to its own enum in the `on_select` handler.
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

/// A pill-grouped segmented control. The active segment gets a raised panel fill;
/// inactive segments are quiet. Unlike an underline tab bar this matches Orca's
/// rounded segmented group (e.g. the 名称 / 内容 switch).
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

// ---------------------------------------------------------------------------
// Search field — the right-pane file filter.
// ---------------------------------------------------------------------------

type KeyHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) -> bool + 'static>;

/// A focusable search/filter well with a leading magnifier icon. The caller owns
/// the draft text and wires `on_key` to its own keystroke classifier; this
/// component is pure presentation plus focus plumbing.
#[derive(IntoElement)]
pub struct SearchField {
    id: ElementId,
    focus: FocusHandle,
    value: String,
    placeholder: String,
    key_context: &'static str,
    on_key: Option<KeyHandler>,
}

impl SearchField {
    pub fn new(id: impl Into<ElementId>, focus: FocusHandle) -> Self {
        Self {
            id: id.into(),
            focus,
            value: String::new(),
            placeholder: "Search".into(),
            key_context: "SearchField",
            on_key: None,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn key_context(mut self, key_context: &'static str) -> Self {
        self.key_context = key_context;
        self
    }

    pub fn on_key(
        mut self,
        handler: impl Fn(&KeyDownEvent, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.on_key = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SearchField {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let is_empty = self.value.is_empty();
        let display = if is_empty {
            self.placeholder.clone()
        } else {
            self.value.clone()
        };
        let text_color = if is_empty {
            theme.text_muted
        } else {
            theme.text
        };
        let focus_for_click = self.focus.clone();
        let on_key = self.on_key;

        div()
            .id(self.id)
            .h(px(30.0))
            .w_full()
            .flex()
            .items_center()
            .gap_2()
            .px_2()
            .rounded(px(radius::MD))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border)
            .track_focus(&self.focus)
            .tab_index(0)
            .key_context(self.key_context)
            .cursor(gpui::CursorStyle::IBeam)
            .hover(move |style| style.border_color(theme.border_strong))
            .child(
                Icon::new(IconName::Search)
                    .size(IconSize::Small)
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .text_color(text_color)
                    .child(display),
            )
            .when_some(on_key, |this, on_key| {
                this.on_key_down(move |event, window, cx| {
                    if on_key(event, window, cx) {
                        cx.stop_propagation();
                    }
                })
            })
            .on_click(move |_: &ClickEvent, window, _| {
                window.focus(&focus_for_click);
            })
    }
}

// ---------------------------------------------------------------------------
// Panel header — the 40px chrome strip atop each pane.
// ---------------------------------------------------------------------------

/// A 40px pane header: a leading title (optionally icon-prefixed) and an optional
/// trailing control slot for actions / tabs / badges.
#[derive(IntoElement)]
pub struct PanelHeader {
    title: String,
    icon: Option<IconName>,
    trailing: Option<AnyElement>,
}

impl PanelHeader {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            trailing: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }
}

impl RenderOnce for PanelHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .h(px(space::PANE_HEADER))
            .px_3()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.chrome)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .min_w_0()
                    .when_some(self.icon, |this, icon| {
                        this.child(
                            Icon::new(icon)
                                .size(IconSize::Small)
                                .color(theme.text_secondary),
                        )
                    })
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(self.title),
                    ),
            )
            .when_some(self.trailing, |this, trailing| {
                this.child(div().flex().items_center().gap_1().child(trailing))
            })
    }
}
