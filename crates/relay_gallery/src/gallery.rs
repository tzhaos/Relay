//! App-shaped gallery scenes for Relay UI kit components.
//!
//! Each scene presents components in the kind of surface where Relay will use
//! them, instead of packing every primitive into one long showcase page.

use gpui::{
    AnyElement, Context, Entity, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use relay_ui_kit::{ActiveTheme, TextInputState, space};

use crate::GalleryApp;

mod command_scene;
mod foundations_scene;
mod product_samples;
mod review_scene;
mod settings_scene;
mod shared;
mod stress_scene;
mod terminal_scene;
mod viewer_samples;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GallerySurface {
    Terminal,
    Review,
    Command,
    Settings,
    Foundations,
    Stress,
}

/// Interactive state shared by the gallery scenes.
pub struct GalleryState {
    pub name_input: TextInputState,
    pub name_focus: FocusHandle,
    pub search_input: TextInputState,
    pub search_focus: FocusHandle,
    pub notifications: bool,
    pub auto_archive: bool,
    pub theme_choice: &'static str,
    pub seg_tab: &'static str,
    pub terminal_session: &'static str,
    pub launcher_choice: &'static str,
    pub branch_choice: &'static str,
    pub branch_event: String,
    pub viewer_tab: &'static str,
    pub shell_split_width: f32,
    pub branch_picker_open: bool,
    pub branch_actions_open: bool,
}

impl GalleryState {
    pub fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            name_input: TextInputState::with_text("relay-agent"),
            name_focus: cx.focus_handle(),
            search_input: TextInputState::new(),
            search_focus: cx.focus_handle(),
            notifications: true,
            auto_archive: false,
            theme_choice: "system",
            seg_tab: "diff",
            terminal_session: "codex",
            launcher_choice: "powershell",
            branch_choice: "ui-kit-branch-controls",
            branch_event: "Ready".into(),
            viewer_tab: "diff",
            shell_split_width: 260.0,
            branch_picker_open: false,
            branch_actions_open: false,
        }
    }
}

pub fn render(
    surface: GallerySurface,
    state: &GalleryState,
    host: &Entity<GalleryApp>,
    window: &Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    let theme = *cx.theme();
    let content: AnyElement = match surface {
        GallerySurface::Terminal => {
            terminal_scene::render(state, host, theme, cx).into_any_element()
        }
        GallerySurface::Review => {
            review_scene::render(state, host, window, theme, cx).into_any_element()
        }
        GallerySurface::Command => command_scene::render(state, host, theme, cx).into_any_element(),
        GallerySurface::Settings => {
            settings_scene::render(state, host, window, theme, cx).into_any_element()
        }
        GallerySurface::Foundations => {
            foundations_scene::render(state, host, theme, cx).into_any_element()
        }
        GallerySurface::Stress => stress_scene::render(state, host, theme, cx).into_any_element(),
    };

    div()
        .id("gallery-scroll")
        .size_full()
        .overflow_y_scroll()
        .bg(theme.app_bg)
        .child(
            div()
                .max_w(px(1160.0))
                .mx_auto()
                .p(px(space::XL))
                .child(content),
        )
}
