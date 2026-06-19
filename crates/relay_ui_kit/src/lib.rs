//! Relay UI kit — a GPUI component library in the Orca product direction.
//!
//! Every component here is a `RenderOnce` builder that reads the active
//! [`theme::Theme`] from the [`gpui::App`] globals and carries generic click /
//! key callbacks. None of them depend on a concrete view, so the same component
//! drops into the gallery, the real workbench, or a test without dragging app
//! state into scope.
//!
//! Stateful controls (text input, checkbox, dropdown open/closed) follow a
//! model/view split: the host owns a small state struct (e.g.
//! [`input::TextInputState`]) and feeds events into it, while the component is a
//! stateless renderer of that state. This keeps the library free of hidden
//! global state and lets the host drive everything.
//!
//! Install the theme once at startup with [`theme::init`], and register the
//! embedded icon set with `Application::new().with_assets(icon::KitAssets)`.

pub mod button;
pub mod choice;
pub mod controls;
pub mod display;
pub mod icon;
pub mod input;
pub mod overlay;
pub mod row;
pub mod structure;
pub mod theme;
pub mod tone;

// Flat re-exports so callers write `relay_ui_kit::Button` etc.
pub use button::{Button, ButtonVariant, IconButton};
pub use choice::{Checkbox, Radio, Toggle};
pub use controls::{PanelHeader, SearchField, Segment, SegmentedControl};
pub use display::{Badge, BadgeStyle, Divider, EmptyState, StatusDot};
pub use icon::{Icon, IconName, IconSize, KitAssets};
pub use input::{TextInput, TextInputAction, TextInputState};
pub use overlay::{Menu, MenuItem, Overlay, TooltipBody, overlay};
pub use row::{NavRow, TaskRow, TaskRowData, TreeRow};
pub use structure::{KeyValue, ListSection, Tab, Tabs};
pub use theme::{ActiveTheme, Theme, radius, space};
pub use tone::Tone;
