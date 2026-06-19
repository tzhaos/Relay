//! Relay design tokens.
//!
//! Single source of truth for color, spacing, sizing, and typography across the
//! workbench. Aligned to `DESIGN.md`'s Visual / Layout / Typography contracts:
//! layered near-white surfaces, quiet warm-gray chrome, a dark dominant terminal
//! surface, a sparse green accent, muted amber/red status colors, 1px borders,
//! and a 4/8/12/16/24 spacing scale.
//!
//! Field names are semantic (what a color *means*, not where it is painted), so
//! panes read intent rather than raw palette slots.

use gpui::{Hsla, rgb};

// ---------------------------------------------------------------------------
// Color tokens
// ---------------------------------------------------------------------------

/// The Relay workbench palette, expressed as semantic tokens.
///
/// During the staged UI redesign a handful of legacy field aliases are kept so
/// not-yet-migrated panes keep compiling. They are marked `legacy` and will be
/// removed once every pane has moved onto the semantic tokens + shared
/// components.
#[derive(Debug, Clone, Copy)]
pub struct RelayTheme {
    // --- Background surfaces (outer -> inner, building quiet depth) ---
    /// Outermost window background.
    pub app_bg: Hsla,
    /// Chrome surfaces: title bar, status bar, pane headers.
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
    /// Muted text: labels, counts, captions.
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

    // --- Legacy aliases (removed once all panes migrate to tokens + components) ---
    /// legacy: alias for [`RelayTheme::app_bg`].
    pub bg: Hsla,
    /// legacy: alias for [`RelayTheme::panel_alt`].
    pub chrome_alt: Hsla,
    /// legacy: alias for [`RelayTheme::border`].
    pub line: Hsla,
    /// legacy: alias for [`RelayTheme::border_strong`].
    pub selection_line: Hsla,
    /// legacy: alias for [`RelayTheme::text_muted`].
    pub muted: Hsla,
}

impl RelayTheme {
    /// The Orca-direction light palette: native, dense, quiet.
    pub fn orca() -> Self {
        let app_bg: Hsla = rgb(0xf7f7f6).into();
        let chrome: Hsla = rgb(0xf1f1f0).into();
        let panel: Hsla = rgb(0xfcfcfb).into();
        let panel_alt: Hsla = rgb(0xf4f4f2).into();
        let inset: Hsla = rgb(0xededeb).into();

        let text: Hsla = rgb(0x1a1c1e).into();
        let text_secondary: Hsla = rgb(0x4b5158).into();
        let text_muted: Hsla = rgb(0x6b7280).into();

        let border: Hsla = rgb(0xe1e1dd).into();
        let border_strong: Hsla = rgb(0xcdcdc8).into();

        let terminal_bg: Hsla = rgb(0x1e222a).into();
        let terminal_text: Hsla = rgb(0xe8e8e3).into();
        let terminal_dim: Hsla = rgb(0x737b87).into();

        let accent: Hsla = rgb(0x16a34a).into();
        let accent_bg: Hsla = rgb(0xe7f6ee).into();
        let accent_border: Hsla = rgb(0xa7e3c4).into();
        let warning: Hsla = rgb(0xb45309).into();
        let danger: Hsla = rgb(0xb91c1c).into();
        let info: Hsla = rgb(0x2563eb).into();

        let hover: Hsla = rgb(0xefefed).into();
        let selection: Hsla = rgb(0xeaeae9).into();

        Self {
            app_bg,
            chrome,
            panel,
            panel_alt,
            inset,
            text,
            text_secondary,
            text_muted,
            border,
            border_strong,
            terminal_bg,
            terminal_text,
            terminal_dim,
            accent,
            accent_bg,
            accent_border,
            warning,
            danger,
            info,
            hover,
            selection,
            // Legacy aliases mirror their semantic counterparts.
            bg: app_bg,
            chrome_alt: panel_alt,
            line: border,
            selection_line: border_strong,
            muted: text_muted,
        }
    }
}

// ---------------------------------------------------------------------------
// Spacing + sizing scale
// ---------------------------------------------------------------------------

/// Named spacing and sizing constants (pixels) per `DESIGN.md`.
///
/// Compose with `gpui::px(...)`, e.g. `px(spacing::MD)`. The scale is the
/// contract: 4 / 8 / 12 / 16 / 24, plus the fixed chrome/pane/row dimensions.
pub mod spacing {
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
    pub const ROW_MD: f32 = 34.0;
    /// Task row (56-72px band).
    pub const TASK_ROW: f32 = 60.0;

    // --- Pane widths (DESIGN.md Layout Contract) ---
    /// Left rail target width (280-320px band).
    pub const RAIL_WIDTH: f32 = 300.0;
    /// Right context pane target width (340-380px band).
    pub const CONTEXT_WIDTH: f32 = 360.0;
}

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Monospace family for terminal + code, with a per-OS fallback so the app
/// renders correctly off Windows (the old code hard-coded `"Consolas"` in 14
/// places, which fell back to the default sans font elsewhere).
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
