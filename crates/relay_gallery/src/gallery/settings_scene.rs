use gpui::{Context, Entity, IntoElement, ParentElement, Styled, Window, div, px};
use relay_ui_kit::{Badge, EmptyState, IconName, Theme, Tone};

use super::{
    GalleryState,
    shared::{checkbox_row, radio_row, scene_stack, section, strip, text_input_field, toggle_row},
};
use crate::GalleryApp;

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryApp>,
    window: &Window,
    theme: Theme,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    let name_focused = state.name_focus.is_focused(window);
    let search_focused = state.search_focus.is_focused(window);

    scene_stack()
        .child(section(
            cx,
            "Agent profile",
            div()
                .max_w(px(420.0))
                .flex()
                .flex_col()
                .gap_3()
                .child(text_input_field(
                    host,
                    "settings-name",
                    &state.name_input,
                    state.name_focus.clone(),
                    name_focused,
                    None,
                    "Agent name",
                ))
                .child(text_input_field(
                    host,
                    "settings-filter",
                    &state.search_input,
                    state.search_focus.clone(),
                    search_focused,
                    Some(IconName::Search),
                    "Default file filter",
                )),
        ))
        .child(section(
            cx,
            "Behavior",
            div()
                .max_w(px(420.0))
                .flex()
                .flex_col()
                .gap_3()
                .child(checkbox_row(host, state.notifications))
                .child(toggle_row(host, state.auto_archive))
                .child(
                    div()
                        .pt_1()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(radio_row(
                            host,
                            "system",
                            "Follow system",
                            state.theme_choice,
                        ))
                        .child(radio_row(host, "light", "Always light", state.theme_choice))
                        .child(radio_row(host, "dark", "Always dark", state.theme_choice)),
                ),
        ))
        .child(section(
            cx,
            "Feedback",
            div()
                .max_w(px(520.0))
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    strip()
                        .child(Badge::new("RUNNING").tone(Tone::Accent).soft())
                        .child(Badge::new("WAITING").tone(Tone::Warning).soft())
                        .child(Badge::new("FAILED").tone(Tone::Danger).soft())
                        .child(Badge::new("main").tone(Tone::Secondary)),
                )
                .child(
                    EmptyState::new(
                        "No saved agent profile",
                        "Create a profile to reuse defaults.",
                    )
                    .icon(IconName::Bot),
                )
                .child(div().text_xs().text_color(theme.text_muted).child(format!(
                    "Current launcher choice: {}",
                    state.launcher_choice
                ))),
        ))
}
