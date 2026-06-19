use gpui::{Context, Entity, IntoElement, ParentElement, Styled, div, prelude::FluentBuilder, px};
use relay_ui_kit::{CommandRow, IconName, KeyboardShortcut, Theme};

use super::{
    GalleryState,
    product_samples::{command_sample, launcher_sample},
    shared::{dropdown_menu, dropdown_trigger, scene_stack, section, strip},
};
use crate::GalleryApp;

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryApp>,
    theme: Theme,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    scene_stack()
        .child(section(
            cx,
            "Command palette",
            div()
                .flex()
                .items_start()
                .gap_3()
                .flex_wrap()
                .child(command_sample(state, host))
                .child(launcher_sample(state, host, theme)),
        ))
        .child(section(
            cx,
            "Shortcuts",
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(shortcut_row(
                    theme,
                    "New terminal",
                    KeyboardShortcut::new(["Ctrl", "Shift", "T"]),
                ))
                .child(shortcut_row(
                    theme,
                    "Launch Codex",
                    KeyboardShortcut::new(["Ctrl", "K"]),
                ))
                .child(shortcut_row(
                    theme,
                    "Filter files",
                    KeyboardShortcut::new(["Ctrl", "F"]),
                )),
        ))
        .child(section(
            cx,
            "Context menu",
            div()
                .relative()
                .child(dropdown_trigger(host, state.menu_open))
                .when(state.menu_open, |this| this.child(dropdown_menu(host))),
        ))
        .child(section(
            cx,
            "Command rows",
            div()
                .max_w(px(560.0))
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    CommandRow::new("cmd-row-terminal", "terminal:new", "New Terminal")
                        .detail("Open a shell session")
                        .icon(IconName::Terminal)
                        .shortcut(KeyboardShortcut::new(["Ctrl", "Shift", "T"]))
                        .selected(state.launcher_choice == "terminal:new"),
                )
                .child(
                    CommandRow::new("cmd-row-agent", "agent:codex", "Launch Codex")
                        .detail("Attach Codex to the active terminal")
                        .icon(IconName::Bot)
                        .shortcut(KeyboardShortcut::new(["Ctrl", "K"]))
                        .selected(state.launcher_choice == "agent:codex"),
                ),
        ))
}

fn shortcut_row(theme: Theme, label: &'static str, shortcut: KeyboardShortcut) -> impl IntoElement {
    strip()
        .w(px(340.0))
        .justify_between()
        .child(
            div()
                .text_sm()
                .text_color(theme.text_secondary)
                .child(label),
        )
        .child(shortcut)
}
