use gpui::{Context, Entity, IntoElement, ParentElement, Styled, div};
use relay_ui_kit::Theme;

use super::{
    GalleryState,
    product_samples::{command_sample, launcher_sample, shell_sample, terminal_sample},
    shared::{scene_stack, section},
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
            "Terminal session",
            terminal_sample(state, host, theme),
        ))
        .child(section(
            cx,
            "Launch command",
            div()
                .flex()
                .items_start()
                .gap_3()
                .flex_wrap()
                .child(launcher_sample(state, host, theme))
                .child(command_sample(state, host)),
        ))
        .child(section(cx, "Shell split", shell_sample(state, host)))
}
