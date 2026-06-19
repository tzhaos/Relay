use gpui::{Context, Entity, FocusHandle, FontWeight, IntoElement, ParentElement, Styled, div, px};
use relay_ui_kit::{
    ActiveTheme, Button, Checkbox, Icon, IconName, IconSize, Menu, MenuItem, Radio, StatusDot,
    TextInput, TextInputAction, TextInputState, Toggle, Tone, overlay, space,
};

use crate::GalleryApp;

pub(super) fn section(
    cx: &mut Context<GalleryApp>,
    title: &str,
    body: impl IntoElement,
) -> impl IntoElement {
    let theme = *cx.theme();
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text_muted)
                .child(title.to_uppercase()),
        )
        .child(
            div()
                .p_3()
                .rounded(px(relay_ui_kit::radius::LG))
                .bg(theme.panel)
                .border_1()
                .border_color(theme.border)
                .child(body),
        )
}

pub(super) fn scene_stack() -> gpui::Div {
    div().flex().flex_col().gap(px(space::XL))
}

pub(super) fn strip() -> gpui::Div {
    div().flex().items_center().gap_3().flex_wrap()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn text_input_field(
    host: &Entity<GalleryApp>,
    id: &'static str,
    input: &TextInputState,
    focus: FocusHandle,
    focused: bool,
    icon: Option<IconName>,
    placeholder: &'static str,
) -> impl IntoElement {
    let host = host.clone();
    let is_name = id.contains("name");
    let mut field = TextInput::new(id, focus, input)
        .placeholder(placeholder)
        .focused(focused)
        .on_key(move |event, _window, cx| {
            host.update(cx, |this, cx| {
                let target = if is_name {
                    &mut this.gallery.name_input
                } else {
                    &mut this.gallery.search_input
                };
                match target.handle_key(event) {
                    TextInputAction::Edited | TextInputAction::Submit | TextInputAction::Cancel => {
                        cx.notify()
                    }
                    TextInputAction::Ignored => {}
                }
            });
        });
    if let Some(icon) = icon {
        field = field.leading_icon(icon);
    }
    field
}

pub(super) fn checkbox_row(host: &Entity<GalleryApp>, checked: bool) -> impl IntoElement {
    Checkbox::new("cb-notify", checked)
        .label("Enable notifications")
        .on_click({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.gallery.notifications = !this.gallery.notifications;
                    cx.notify();
                });
            }
        })
}

pub(super) fn toggle_row(host: &Entity<GalleryApp>, on: bool) -> impl IntoElement {
    Toggle::new("tg-archive", on)
        .label("Auto-archive merged tasks")
        .on_click({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.gallery.auto_archive = !this.gallery.auto_archive;
                    cx.notify();
                });
            }
        })
}

pub(super) fn radio_row(
    host: &Entity<GalleryApp>,
    key: &'static str,
    label: &'static str,
    selected: &'static str,
) -> impl IntoElement {
    Radio::new(key, key == selected, label).on_click({
        let host = host.clone();
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.gallery.theme_choice = key;
                cx.notify();
            });
        }
    })
}

pub(super) fn dropdown_trigger(host: &Entity<GalleryApp>, open: bool) -> impl IntoElement {
    Button::new("dd-trigger", "Branch actions")
        .icon(if open {
            IconName::ChevronDown
        } else {
            IconName::ChevronRight
        })
        .on_click({
            let host = host.clone();
            move |_event, _window, cx| {
                host.update(cx, |this, cx| {
                    this.gallery.menu_open = !this.gallery.menu_open;
                    cx.notify();
                });
            }
        })
}

pub(super) fn dropdown_menu(host: &Entity<GalleryApp>) -> impl IntoElement {
    let close = {
        let host = host.clone();
        move |cx: &mut gpui::App| {
            host.update(cx, |this, cx| {
                this.gallery.menu_open = false;
                cx.notify();
            });
        }
    };
    let close_a = close.clone();
    let close_b = close.clone();
    let close_c = close;
    overlay(Menu::new(
        "dd-menu",
        vec![
            MenuItem::new("Checkout")
                .icon(IconName::GitBranch)
                .on_click(move |_e, _w, cx| close_a(cx)),
            MenuItem::new("New worktree")
                .icon(IconName::FolderPlus)
                .on_click(move |_e, _w, cx| close_b(cx)),
            MenuItem::separator(),
            MenuItem::new("Delete branch")
                .icon(IconName::Archive)
                .danger()
                .on_click(move |_e, _w, cx| close_c(cx)),
        ],
    ))
    .offset(0.0, 34.0)
}

pub(super) fn dot_label(theme: relay_ui_kit::Theme, tone: Tone, label: &str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(StatusDot::new(tone))
        .child(
            div()
                .text_sm()
                .text_color(theme.text_secondary)
                .child(label.to_string()),
        )
}

pub(super) fn icon_sample(theme: relay_ui_kit::Theme, name: IconName) -> impl IntoElement {
    div()
        .size(px(32.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(relay_ui_kit::radius::MD))
        .bg(theme.panel_alt)
        .border_1()
        .border_color(theme.border)
        .child(
            Icon::new(name)
                .size(IconSize::Medium)
                .color(theme.text_secondary),
        )
}
