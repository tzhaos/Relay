use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::ActiveTheme,
};

/// One tab in a [`Tabs`] bar.
pub struct Tab {
    key: &'static str,
    label: &'static str,
    icon: Option<IconName>,
    count: Option<usize>,
}

impl Tab {
    pub fn new(key: &'static str, label: &'static str) -> Self {
        Self {
            key,
            label,
            icon: None,
            count: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }
}

type SelectHandler = Box<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;

/// An underline tab bar.
#[derive(IntoElement)]
pub struct Tabs {
    id: ElementId,
    tabs: Vec<Tab>,
    active: &'static str,
    on_select: Option<SelectHandler>,
}

impl Tabs {
    pub fn new(id: impl Into<ElementId>, tabs: Vec<Tab>) -> Self {
        Self {
            id: id.into(),
            tabs,
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

impl RenderOnce for Tabs {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let handler = self.on_select.map(std::rc::Rc::new);
        let active = self.active;
        let mut row = div()
            .id(self.id)
            .h(px(36.0))
            .w_full()
            .flex()
            .items_center()
            .gap_1()
            .border_b_1()
            .border_color(theme.border);

        for (index, tab) in self.tabs.into_iter().enumerate() {
            let is_active = tab.key == active;
            let key = tab.key;
            let handler = handler.clone();
            let (fg, underline) = if is_active {
                (theme.text, theme.accent)
            } else {
                (theme.text_muted, gpui::transparent_black())
            };

            let cell = div()
                .id(("tab", index))
                .px_2()
                .flex()
                .items_center()
                .gap_1()
                .text_sm()
                .font_weight(if is_active {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(fg)
                .border_b_2()
                .border_color(underline)
                .when(!is_active, |this| {
                    this.cursor_pointer()
                        .hover(move |s| s.text_color(theme.text_secondary))
                })
                .when_some(tab.icon, |this, icon| {
                    let c = if is_active {
                        theme.accent
                    } else {
                        theme.text_muted
                    };
                    this.child(Icon::new(icon).size(IconSize::Small).color(c))
                })
                .child(tab.label)
                .when_some(tab.count, |this, count| {
                    this.child(
                        div()
                            .text_size(px(11.0))
                            .text_color(theme.text_muted)
                            .child(format!("({count})")),
                    )
                })
                .when_some(handler.filter(|_| !is_active), |this, handler| {
                    this.on_click(move |_: &ClickEvent, window, cx| {
                        handler(key, window, cx);
                        cx.stop_propagation();
                    })
                });
            row = row.child(cell);
        }
        row
    }
}
