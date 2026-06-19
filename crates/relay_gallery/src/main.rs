//! Relay UI kit gallery.
//!
//! A standalone, fully-interactive showcase app that proves the `relay_ui_kit`
//! components render and behave at Orca quality in GPUI, with no dependency on
//! the real workbench domain. Two pages:
//!
//! - **Workbench** — the Orca three-column shell (left rail / center terminal /
//!   right context). Click tasks to activate them, switch Files/Diff/Review and
//!   Terminal/Preview, filter the file tree, open the row context menu.
//! - **Components** — every kit primitive in its states: type in the inputs,
//!   toggle checkboxes/switches, pick radios, open the dropdown.
//!
//! Interactivity pattern: components carry view-free callbacks
//! (`Fn(&ClickEvent, &mut Window, &mut App)`). The page render functions receive
//! the `Entity<GalleryApp>`, so a callback closes over it and calls
//! `entity.update(cx, |this, cx| ...)` to mutate state + `cx.notify()`.
//!
//! Run with `cargo run -p relay_gallery`.

mod gallery;
mod workbench_demo;

use gpui::{
    App, AppContext, Application, Bounds, Context, Entity, FocusHandle, FontWeight,
    InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled,
    Window, WindowBounds, WindowDecorations, WindowOptions, div, prelude::FluentBuilder, px, size,
};
use relay_ui_kit::{ActiveTheme, KitAssets, TitleBar, WorkspaceBreadcrumb, theme};

/// Which gallery page is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Workbench,
    Components,
}

pub struct GalleryApp {
    page: Page,
    pub gallery: gallery::GalleryState,
    pub workbench: workbench_demo::WorkbenchState,
}

impl GalleryApp {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            page: Page::Workbench,
            gallery: gallery::GalleryState::new(cx),
            workbench: workbench_demo::WorkbenchState::new(cx),
        }
    }

    fn top_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        TitleBar::new("Relay")
            .project("UI Kit")
            .center(WorkspaceBreadcrumb::new(vec![
                "Relay".into(),
                "Gallery".into(),
                self.page_label().into(),
            ]))
            .actions(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(self.page_tab(Page::Workbench, "Workbench", cx))
                    .child(self.page_tab(Page::Components, "Components", cx)),
            )
    }

    fn page_label(&self) -> &'static str {
        match self.page {
            Page::Workbench => "Workbench",
            Page::Components => "Components",
        }
    }

    fn page_tab(
        &self,
        page: Page,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = *cx.theme();
        let is_active = self.page == page;
        div()
            .id(label)
            .h(px(26.0))
            .px_3()
            .flex()
            .items_center()
            .rounded(px(theme::radius::MD))
            .text_xs()
            .font_weight(if is_active {
                FontWeight::SEMIBOLD
            } else {
                FontWeight::MEDIUM
            })
            .when(is_active, |this| {
                this.bg(theme.selection).text_color(theme.text)
            })
            .when(!is_active, |this| {
                this.text_color(theme.text_muted)
                    .cursor_pointer()
                    .hover(move |s| s.bg(theme.hover).text_color(theme.text_secondary))
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.page = page;
                cx.notify();
            }))
            .child(label)
    }
}

impl Render for GalleryApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();
        let entity = cx.entity();
        let body = match self.page {
            Page::Workbench => {
                workbench_demo::render(&self.workbench, &entity, window, cx).into_any_element()
            }
            Page::Components => {
                gallery::render(&self.gallery, &entity, window, cx).into_any_element()
            }
        };

        div()
            .size_full()
            .bg(theme.app_bg)
            .text_color(theme.text)
            .font_family(theme::ui_family())
            .flex()
            .flex_col()
            .child(self.top_bar(cx))
            .child(div().flex_1().min_h_0().child(body))
    }
}

/// Shared helper: a focusable handle factory for state structs.
pub fn focus(cx: &mut Context<GalleryApp>) -> FocusHandle {
    cx.focus_handle()
}

/// Type alias so page modules can name the host entity tersely.
pub type Host = Entity<GalleryApp>;

fn main() {
    Application::new()
        .with_assets(KitAssets)
        .run(|cx: &mut App| {
            theme::init(cx);
            let bounds = Bounds::centered(None, size(px(1440.0), px(900.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: None,
                    window_decorations: Some(WindowDecorations::Client),
                    window_min_size: Some(size(px(1180.0), px(780.0))),
                    app_id: Some("relay-gallery".to_string()),
                    ..Default::default()
                },
                |_, cx| cx.new(GalleryApp::new),
            )
            .expect("open gallery window");
            cx.activate(true);
        });
}
