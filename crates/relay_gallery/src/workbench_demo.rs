//! Orca-direction workbench sample assembled from `relay_ui_kit`.

mod center;
mod context;
mod data;
mod rail;

use gpui::{Context, Entity, FocusHandle, IntoElement, Window};
use relay_ui_kit::{
    ActiveTheme, AppShell, SplitPane, StatusBar, StatusItem, TextInputState, Tone, icon::IconName,
    theme::space,
};

use crate::GalleryApp;
use center::center_pane;
use context::right_context;
use data::{active_session, active_task};
use rail::left_rail;

/// Interactive state for the Workbench page.
pub struct WorkbenchState {
    pub active_task: usize,
    pub active_session: usize,
    pub context_tab: &'static str,
    pub route: &'static str,
    pub filter: TextInputState,
    pub filter_focus: FocusHandle,
    pub launcher_open: bool,
    pub left_width: f32,
    pub terminal_width: f32,
}

impl WorkbenchState {
    pub fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            active_task: 0,
            active_session: 0,
            context_tab: "files",
            route: "terminal",
            filter: TextInputState::new(),
            filter_focus: cx.focus_handle(),
            launcher_open: false,
            left_width: space::RAIL_WIDTH,
            terminal_width: 760.0,
        }
    }
}

pub fn render(
    state: &WorkbenchState,
    host: &Entity<GalleryApp>,
    window: &Window,
    cx: &mut Context<GalleryApp>,
) -> impl IntoElement {
    let theme = *cx.theme();
    let left = left_rail(state, host, theme);
    let center = center_pane(state, host, theme);
    let right = right_context(state, host, window, theme);
    let center_and_context = SplitPane::new("center-context-split", center, right)
        .first_size(state.terminal_width)
        .min_sizes(560.0, 320.0)
        .on_resize({
            let host = host.clone();
            move |next, _window, cx| {
                host.update(cx, |this, cx| {
                    this.workbench.terminal_width = next;
                    cx.notify();
                });
            }
        });

    let workbench = SplitPane::new("workbench-left-split", left, center_and_context)
        .first_size(state.left_width)
        .min_sizes(260.0, 780.0)
        .on_resize({
            let host = host.clone();
            move |next, _window, cx| {
                host.update(cx, |this, cx| {
                    this.workbench.left_width = next;
                    cx.notify();
                });
            }
        });

    AppShell::new(workbench).status_bar(status_bar(state))
}

fn status_bar(state: &WorkbenchState) -> impl IntoElement {
    let task = active_task(state);
    let session = active_session(state);

    StatusBar::new()
        .left(
            StatusItem::new("Runtime", "Gallery")
                .icon(IconName::Terminal)
                .tone(Tone::Info),
        )
        .left(StatusItem::new("Focus", state.route).tone(Tone::Secondary))
        .left(StatusItem::new("Worktree", task.worktree).tone(task.tone))
        .right(StatusItem::new("Session", session.label).tone(session.tone))
        .right(StatusItem::new("Changes", task.changed.to_string()).tone(Tone::Secondary))
        .right(StatusItem::new("Review", task.review.to_string()).tone(Tone::Warning))
}
