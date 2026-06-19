//! Relay design tokens, exposed as a GPUI [`Global`].
//!
//! This is the single source of truth for color, spacing, sizing, and
//! typography across every Relay surface. It is intentionally decoupled from any
//! concrete view: components read the theme from the [`App`] via
//! [`ActiveTheme::theme`], so the same component library works in the gallery, in
//! the real workbench, or in tests without dragging `AppShell` into scope.
//!
//! Direction (per `DESIGN.md`): Orca's native, dense, quiet desktop language with
//! Zed-grade crispness — layered near-white surfaces, soft warm-gray chrome, a
//! dark dominant terminal surface, a sparse green accent, muted amber/red status
//! colors, 1px borders, and a 4/8/12/16/24 spacing scale.

use gpui::{App, Global, Hsla, rgb};

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

/// The Relay palette, expressed as semantic tokens.
///
/// Field names describe what a color *means* (intent), not where it is painted,
/// so surfaces read as `theme.panel` / `theme.accent` rather than raw palette
/// slots. This keeps a future dark theme a drop-in swap.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    // --- Background surfaces (outer -> inner, building quiet depth) ---
    /// Outermost window background.
    pub app_bg: Hsla,
    /// Chrome surfaces: title bar, status bar, pane headers, left rail.
    pub chrome: Hsla,
    /// Primary content panel surface (the brightest reading surface).
    pub panel: Hsla,
    /// Secondary panel surface: cards, grouped rows, embedded lists.
    pub panel_alt: Hsla,
    /// Recessed surface: input fields, code blocks, terminal-adjacent wells.
    pub inset: Hsla,

    // --- Text (three contrast steps) ---
    /// Primary text, near-black.
    pub text: Hsla,
    /// Secondary text: branch names, paths, metadata.
    pub text_secondary: Hsla,
    /// Muted text: labels, counts, captions, idle icons.
    pub text_muted: Hsla,

    // --- Borders / dividers (1px, soft) ---
    /// Default 1px hairline divider.
    pub border: Hsla,
    /// Stronger border for focus / selection edges.
    pub border_strong: Hsla,

    // --- Terminal (dark, visually dominant surface) ---
    pub terminal_bg: Hsla,
    pub terminal_text: Hsla,
    /// Dimmed terminal text: prompts, secondary output.
    pub terminal_dim: Hsla,

    // --- Accent + status (sparse, meaningful) ---
    /// Green accent: running / active / selected rows.
    pub accent: Hsla,
    /// On-accent foreground (text/icon painted on top of `accent` fills).
    pub on_accent: Hsla,
    /// Tinted accent background: selected row fill, accent badge fill.
    pub accent_bg: Hsla,
    /// Accent-tinted border.
    pub accent_border: Hsla,
    /// Amber: waiting / needs-attention / draft.
    pub warning: Hsla,
    /// Red: failed / destructive.
    pub danger: Hsla,
    /// Blue: informational / connection state.
    pub info: Hsla,

    // --- Interaction ---
    /// Hover fill for rows and controls.
    pub hover: Hsla,
    /// Selection fill for active/selected items.
    pub selection: Hsla,
}

impl Theme {
    /// The Orca-direction light palette: native, dense, quiet.
    pub fn light() -> Self {
        Self {
            app_bg: rgb(0xf7f7f6).into(),
            chrome: rgb(0xf1f1f0).into(),
            panel: rgb(0xfcfcfb).into(),
            panel_alt: rgb(0xf4f4f2).into(),
            inset: rgb(0xededeb).into(),

            text: rgb(0x1a1c1e).into(),
            text_secondary: rgb(0x4b5158).into(),
            text_muted: rgb(0x868d96).into(),

            border: rgb(0xe6e6e2).into(),
            border_strong: rgb(0xd2d2cc).into(),

            terminal_bg: rgb(0x1e222a).into(),
            terminal_text: rgb(0xe8e8e3).into(),
            terminal_dim: rgb(0x737b87).into(),

            accent: rgb(0x16a34a).into(),
            on_accent: rgb(0xffffff).into(),
            accent_bg: rgb(0xe7f6ee).into(),
            accent_border: rgb(0xa7e3c4).into(),
            warning: rgb(0xb45309).into(),
            danger: rgb(0xb91c1c).into(),
            info: rgb(0x2563eb).into(),

            hover: rgb(0xececeb).into(),
            selection: rgb(0xe4e9e6).into(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

impl Global for Theme {}

/// Convenience accessor for the active [`Theme`] held in the [`App`] globals.
///
/// Implemented for `App` so components can write `cx.theme()` regardless of which
/// view is rendering them. The theme must be installed once at startup via
/// [`init`]; until then this falls back to [`Theme::light`].
pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Theme {
        self.global::<Theme>()
    }
}

/// Install the default theme into the app globals. Call once at startup, before
/// opening any window that renders kit components.
pub fn init(cx: &mut App) {
    cx.set_global(Theme::light());
}

// ---------------------------------------------------------------------------
// Spacing + sizing scale
// ---------------------------------------------------------------------------

/// Named spacing and sizing constants (pixels) per `DESIGN.md`.
///
/// Compose with `gpui::px(...)`, e.g. `px(space::MD)`. The 4/8/12/16/24 scale is
/// the contract, alongside the fixed chrome/pane/row dimensions.
pub mod space {
    /// 4px — tight intra-component gaps.
    pub const XS: f32 = 4.0;
    /// 8px — default rhythm.
    pub const SM: f32 = 8.0;
    /// 12px — row padding, group insets.
    pub const MD: f32 = 12.0;
    /// 16px — section padding.
    pub const LG: f32 = 16.0;
    /// 24px — major separation.
    pub const XL: f32 = 24.0;

    // --- Fixed chrome heights (DESIGN.md Layout Contract) ---
    /// Top app bar height (40-44px band).
    pub const TITLE_BAR: f32 = 40.0;
    /// Pane header height (40-42px band).
    pub const PANE_HEADER: f32 = 40.0;
    /// Bottom status strip height (28-32px band).
    pub const STATUS_BAR: f32 = 28.0;

    // --- Row heights ---
    /// Compact nav/file row (28-36px band).
    pub const ROW_SM: f32 = 28.0;
    /// Standard nav row.
    pub const ROW_MD: f32 = 32.0;
    /// Task row (56-72px band).
    pub const TASK_ROW: f32 = 64.0;

    // --- Pane widths (DESIGN.md Layout Contract) ---
    /// Left rail target width (280-320px band).
    pub const RAIL_WIDTH: f32 = 300.0;
    /// Right context pane target width (340-380px band).
    pub const CONTEXT_WIDTH: f32 = 360.0;
}

// ---------------------------------------------------------------------------
// Radius scale
// ---------------------------------------------------------------------------

/// Corner radii. Small-to-medium per the Visual Contract: 6-10px for inputs and
/// selected rows.
pub mod radius {
    /// 4px — chips, badges, tight controls.
    pub const SM: f32 = 4.0;
    /// 6px — buttons, rows, inputs.
    pub const MD: f32 = 6.0;
    /// 8px — cards, popovers.
    pub const LG: f32 = 8.0;
}

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Monospace family for terminal + code, with a per-OS fallback.
pub fn mono_family() -> &'static str {
    if cfg!(target_os = "windows") {
        "Consolas"
    } else if cfg!(target_os = "macos") {
        "Menlo"
    } else {
        "monospace"
    }
}

/// UI sans-serif family with a per-OS system stack.
pub fn ui_family() -> &'static str {
    if cfg!(target_os = "windows") {
        "Segoe UI"
    } else if cfg!(target_os = "macos") {
        "-apple-system"
    } else {
        "sans-serif"
    }
}
