//! Workbench shell components.
//!
//! The shell layer owns Relay's client-side chrome, pane sizing, split handles,
//! and status strip. Individual files stay scoped to one UI concept so the kit
//! remains easy to review as it grows.

mod app_shell;
mod pane;
mod split_pane;
mod status_bar;
mod title_bar;
mod toolbar;

pub use app_shell::AppShell;
pub use pane::{Pane, PaneSurface, PaneWidth};
pub use split_pane::{SplitAxis, SplitPane};
pub use status_bar::{StatusBar, StatusItem};
pub use title_bar::{TitleBar, WindowControls};
pub use toolbar::{PaneToolbar, TopToolbar, WorkspaceBreadcrumb};
