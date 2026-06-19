//! Relay UI kit gallery.
//!
//! A standalone, fully-interactive showcase app that proves the `relay_ui_kit`
//! components render and behave at Orca quality in GPUI, with no dependency on
//! the real workbench domain. The gallery is a studio that launches several
//! small app-shaped scenes so components appear in the kind of surface where
//! Relay will use them:
//!
//! - **Workbench** — the Orca three-column shell (left rail / center terminal /
//!   right context). Click tasks to activate them, switch Files/Diff/Review and
//!   Terminal/Preview, filter the file tree, open the row context menu.
//! - **Terminal Hub** — terminal tabs, agent quick launch, and session UI.
//! - **Review** — file tree, Markdown/code preview, and diff review surfaces.
//! - **Command Center** — command palette, launcher, shortcuts, and menus.
//! - **Settings** — forms, choices, dropdowns, and feedback states.
//! - **Foundations** — buttons, icons, badges, rows, tabs, and empty states.
//! - **Stress Lab** — long labels, dense rows, disabled states, and overflow.
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
    App, AppContext, Application, Bounds, Context, Entity, FocusHandle, IntoElement, ParentElement,
    Render, Styled, Window, WindowBounds, WindowDecorations, WindowOptions, div, px, size,
};
use relay_ui_kit::{
    ActiveTheme, IconName, KitAssets, NavRow, TitleBar, WorkspaceBreadcrumb, theme,
};

/// Which gallery page is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Workbench,
    Terminal,
    Review,
    Command,
    Settings,
    Foundations,
    Stress,
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

    fn top_bar(&self) -> impl IntoElement {
        TitleBar::new("Relay")
            .project("UI Kit")
            .center(WorkspaceBreadcrumb::new(vec![
                "Relay".into(),
                "Studio".into(),
                self.page_label().into(),
            ]))
    }

    fn page_label(&self) -> &'static str {
        match self.page {
            Page::Workbench => "Workbench",
            Page::Terminal => "Terminal Hub",
            Page::Review => "Review Desk",
            Page::Command => "Command Center",
            Page::Settings => "Settings",
            Page::Foundations => "Foundation Lab",
            Page::Stress => "Stress Lab",
        }
    }

    fn page_icon(page: Page) -> IconName {
        match page {
            Page::Workbench => IconName::PanelLeft,
            Page::Terminal => IconName::Terminal,
            Page::Review => IconName::FileDiff,
            Page::Command => IconName::Zap,
            Page::Settings => IconName::Settings,
            Page::Foundations => IconName::LayoutGrid,
            Page::Stress => IconName::ListChecks,
        }
    }

    fn page_count(page: Page) -> Option<usize> {
        match page {
            Page::Review => Some(3),
            Page::Command => Some(4),
            Page::Stress => Some(9),
            _ => None,
        }
    }

    fn catalog(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .w(px(relay_ui_kit::space::RAIL_WIDTH))
            .flex_shrink_0()
            .h_full()
            .px_3()
            .py_3()
            .flex()
            .flex_col()
            .gap_1()
            .border_r_1()
            .border_color(theme.border)
            .bg(theme.chrome)
            .child(self.catalog_row(Page::Workbench, cx))
            .child(self.catalog_row(Page::Terminal, cx))
            .child(self.catalog_row(Page::Review, cx))
            .child(self.catalog_row(Page::Command, cx))
            .child(self.catalog_row(Page::Settings, cx))
            .child(self.catalog_row(Page::Foundations, cx))
            .child(self.catalog_row(Page::Stress, cx))
    }

    fn catalog_row(&self, page: Page, cx: &mut Context<Self>) -> impl IntoElement {
        let mut row = NavRow::new(
            Self::page_key(page),
            Self::page_icon(page),
            self.page_label_for(page),
        )
        .selected(self.page == page)
        .on_click(cx.listener(move |this, _, _, cx| {
            this.page = page;
            cx.notify();
        }));
        if let Some(count) = Self::page_count(page) {
            row = row.count(count);
        }
        row
    }

    fn page_key(page: Page) -> &'static str {
        match page {
            Page::Workbench => "studio-workbench",
            Page::Terminal => "studio-terminal",
            Page::Review => "studio-review",
            Page::Command => "studio-command",
            Page::Settings => "studio-settings",
            Page::Foundations => "studio-foundations",
            Page::Stress => "studio-stress",
        }
    }

    fn page_label_for(&self, page: Page) -> &'static str {
        match page {
            Page::Workbench => "Workbench",
            Page::Terminal => "Terminal Hub",
            Page::Review => "Review Desk",
            Page::Command => "Command Center",
            Page::Settings => "Settings",
            Page::Foundations => "Foundation Lab",
            Page::Stress => "Stress Lab",
        }
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
            Page::Terminal => gallery::render(
                gallery::GallerySurface::Terminal,
                &self.gallery,
                &entity,
                window,
                cx,
            )
            .into_any_element(),
            Page::Review => gallery::render(
                gallery::GallerySurface::Review,
                &self.gallery,
                &entity,
                window,
                cx,
            )
            .into_any_element(),
            Page::Command => gallery::render(
                gallery::GallerySurface::Command,
                &self.gallery,
                &entity,
                window,
                cx,
            )
            .into_any_element(),
            Page::Settings => gallery::render(
                gallery::GallerySurface::Settings,
                &self.gallery,
                &entity,
                window,
                cx,
            )
            .into_any_element(),
            Page::Foundations => gallery::render(
                gallery::GallerySurface::Foundations,
                &self.gallery,
                &entity,
                window,
                cx,
            )
            .into_any_element(),
            Page::Stress => gallery::render(
                gallery::GallerySurface::Stress,
                &self.gallery,
                &entity,
                window,
                cx,
            )
            .into_any_element(),
        };

        div()
            .size_full()
            .bg(theme.app_bg)
            .text_color(theme.text)
            .font_family(theme::ui_family())
            .flex()
            .flex_col()
            .child(self.top_bar())
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(self.catalog(cx))
                    .child(div().flex_1().min_w_0().min_h_0().child(body)),
            )
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
