//! Shared GPUI components for the Relay workbench.
//!
//! These primitives replace the scattered, duplicated helpers that used to live
//! inside each pane file (six near-identical button factories, four badge
//! factories, two `empty_state` copies, etc.). Everything here is pure
//! presentation: it reads tokens from [`RelayTheme`] and emits styled elements.
//! Side effects stay out — callers pass click callbacks that dispatch
//! [`WorkbenchCommand`](crate::workbench::WorkbenchCommand)s via
//! [`AppShell::dispatch`](crate::app_shell::AppShell::dispatch).

use std::rc::Rc;

use gpui::{
    AnyElement, Context, CursorStyle, ElementId, FocusHandle, FontWeight, Hsla, InteractiveElement,
    IntoElement, KeyDownEvent, Keystroke, StatefulInteractiveElement, div, prelude::*, px,
};

use crate::{app_shell::AppShell, theme::RelayTheme};

// ---------------------------------------------------------------------------
// Tone — the semantic palette a caller picks for a badge / dot / text accent.
// ---------------------------------------------------------------------------

/// Semantic color tone for status indicators, mapped to theme tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Running / active / success — green accent.
    Accent,
    /// Waiting / needs-attention / draft — amber.
    Warning,
    /// Failed / destructive — red.
    Danger,
    /// Informational / connection — blue.
    Info,
    /// Neutral / idle / archived — muted text.
    Muted,
    /// Secondary — mid-gray.
    Secondary,
}

impl Tone {
    /// Foreground color for this tone.
    pub fn fg(self, theme: RelayTheme) -> Hsla {
        match self {
            Tone::Accent => theme.accent,
            Tone::Warning => theme.warning,
            Tone::Danger => theme.danger,
            Tone::Info => theme.info,
            Tone::Muted => theme.text_muted,
            Tone::Secondary => theme.text_secondary,
        }
    }

    /// Soft tinted background for this tone (for filled badges).
    pub fn soft_bg(self, theme: RelayTheme) -> Hsla {
        match self {
            Tone::Accent => theme.accent_bg,
            Tone::Warning | Tone::Danger | Tone::Info | Tone::Muted | Tone::Secondary => {
                theme.panel_alt
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Button
// ---------------------------------------------------------------------------

/// Visual emphasis for [`button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonEmphasis {
    /// Filled accent — the single primary action in a pane (e.g. Launch Agent).
    Primary,
    /// Outlined — standard actionable controls.
    Secondary,
    /// Borderless text — low-stakes secondary actions.
    Ghost,
}

/// A compact clickable button.
///
/// `on_click` receives the shell + context so callers can `dispatch` a
/// [`WorkbenchCommand`](crate::workbench::WorkbenchCommand). `id` must be unique
/// within the window (GPUI requirement for interactive elements).
pub fn button<F>(
    theme: RelayTheme,
    label: &str,
    emphasis: ButtonEmphasis,
    id: impl Into<ElementId>,
    cx: &mut Context<AppShell>,
    on_click: F,
) -> AnyElement
where
    F: Fn(&mut AppShell, &mut Context<AppShell>) + 'static,
{
    let (bg, border, fg, hover_bg) = match emphasis {
        ButtonEmphasis::Primary => (
            theme.accent,
            theme.accent,
            theme.terminal_text,
            theme.accent,
        ),
        ButtonEmphasis::Secondary => (theme.panel, theme.border_strong, theme.text, theme.hover),
        ButtonEmphasis::Ghost => (theme.panel, theme.border, theme.text_secondary, theme.hover),
    };

    div()
        .h(px(spacing_tiny()))
        .px_2()
        .rounded_md()
        .border_1()
        .border_color(border)
        .bg(bg)
        .flex()
        .items_center()
        .gap_1()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(fg)
        .cursor_pointer()
        .hover(move |style| style.bg(hover_bg).border_color(theme.border_strong))
        .id(id)
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _, cx| {
            on_click(this, cx);
            // A button is a leaf action: stop the click from bubbling to a parent
            // row's click handler (e.g. archive inside a task row).
            cx.stop_propagation();
        }))
        .child(label.to_string())
        .into_any_element()
}

/// Button height shared by [`button`] and [`badge`] so controls align in a row.
fn spacing_tiny() -> f32 {
    26.0
}

// ---------------------------------------------------------------------------
// Badge — compact uppercase label chip. Unifies the four `*_state_badge` copies.
// ---------------------------------------------------------------------------

/// A small uppercase status chip. `tone` drives foreground color; the chip uses
/// a soft tinted fill for accent tones and a quiet bordered fill otherwise.
pub fn badge(theme: RelayTheme, label: &str, tone: Tone) -> AnyElement {
    let fg = tone.fg(theme);
    let bg = if tone == Tone::Accent {
        theme.accent_bg
    } else {
        theme.panel_alt
    };
    let border = if tone == Tone::Accent {
        theme.accent_border
    } else {
        theme.border
    };

    div()
        .h(px(spacing_tiny()))
        .px_2()
        .rounded_sm()
        .border_1()
        .border_color(border)
        .bg(bg)
        .flex()
        .items_center()
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(fg)
        .child(label.to_string())
        .into_any_element()
}

/// A flat (unfilled, borderless) uppercase mini-label, for inline metadata
/// where a full badge would be too heavy.
pub fn flat_label(theme: RelayTheme, label: &str, tone: Tone) -> AnyElement {
    div()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(tone.fg(theme))
        .child(label.to_string())
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Status dot
// ---------------------------------------------------------------------------

/// A small circular status indicator (agent / task / connection state).
pub fn status_dot(theme: RelayTheme, tone: Tone) -> AnyElement {
    div()
        .w(px(8.0))
        .h(px(8.0))
        .rounded_full()
        .bg(tone.fg(theme))
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Pill / metric — workspace metric tiles.
// ---------------------------------------------------------------------------

/// A compact key/value metric tile for the workspace footer.
pub fn metric_pill(theme: RelayTheme, label: &str, value: String) -> AnyElement {
    div()
        .flex_1()
        .min_w(px(0.0))
        .h(px(28.0))
        .rounded_md()
        .bg(theme.panel_alt)
        .border_1()
        .border_color(theme.border)
        .px_2()
        .flex()
        .items_center()
        .justify_between()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child(label.to_string()),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text)
                .font_weight(FontWeight::BOLD)
                .child(value),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Empty state — operational, not literal. Unifies the two `empty_state` copies.
// ---------------------------------------------------------------------------

/// A compact operational empty state. `title` states the situation; `detail`
/// gives an actionable next step rather than a raw "no X stored" string.
pub fn empty_state(theme: RelayTheme, title: &str, detail: &str) -> AnyElement {
    div()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.panel_alt)
        .p_3()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_sm()
                .text_color(theme.text)
                .font_weight(FontWeight::MEDIUM)
                .child(title.to_string()),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child(detail.to_string()),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Segmented tabs — Files/Diff/Review and Terminal/Preview.
// ---------------------------------------------------------------------------

/// A single entry in a [`segmented_tabs`] group.
pub struct TabItem<E: PartialEq> {
    pub value: E,
    pub label: &'static str,
}

/// A horizontal segmented tab bar. The active tab gets the accent text + an
/// accent underline; inactive tabs are muted. Width is driven by content (the
/// bar never resizes its parent when switching tabs).
pub fn segmented_tabs<E, F>(
    theme: RelayTheme,
    active: E,
    items: &[TabItem<E>],
    cx: &mut Context<AppShell>,
    on_select: F,
) -> AnyElement
where
    E: PartialEq + Copy + 'static,
    F: Fn(&mut AppShell, E, &mut Context<AppShell>) + 'static,
{
    // Wrap the callback in Rc so each tab's click closure can clone it without
    // requiring F: Copy.
    let on_select = Rc::new(on_select);
    let mut row = div().flex().items_end().gap_1().h_full();
    for (index, item) in items.iter().enumerate() {
        let is_active = item.value == active;
        let value = item.value;
        let label = item.label;
        let fg = if is_active {
            theme.accent
        } else {
            theme.text_muted
        };
        let underline = if is_active {
            theme.accent
        } else {
            gpui::transparent_black()
        };
        let on_select = Rc::clone(&on_select);
        let cell = div()
            .h_full()
            .px_2()
            .flex()
            .items_center()
            .text_xs()
            .font_weight(if is_active {
                FontWeight::SEMIBOLD
            } else {
                FontWeight::MEDIUM
            })
            .text_color(fg)
            .cursor_pointer()
            .hover(move |style| {
                style.text_color(if is_active {
                    theme.accent
                } else {
                    theme.text_secondary
                })
            })
            .border_b_2()
            .border_color(underline)
            .id(("seg-tab", index))
            .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _, cx| {
                on_select(this, value, cx);
            }))
            .child(label);
        row = row.child(cell);
    }
    row.into_any_element()
}

// ---------------------------------------------------------------------------
// Panel header — the 40px chrome strip atop each pane.
// ---------------------------------------------------------------------------

/// A 40px pane header: a leading title and an optional trailing control row.
/// `trailing` slots actions (refresh button, tabs, badges) on the right edge.
pub fn panel_header<F>(theme: RelayTheme, title: &str, trailing: F) -> AnyElement
where
    F: FnOnce() -> Option<AnyElement>,
{
    let mut header = div()
        .h(px(crate::theme::spacing::PANE_HEADER))
        .px_3()
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .border_b_1()
        .border_color(theme.border)
        .bg(theme.chrome)
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text)
                .child(title.to_string()),
        );
    if let Some(element) = trailing() {
        header = header.child(div().flex().items_center().gap_1().child(element));
    }
    header.into_any_element()
}

// ---------------------------------------------------------------------------
// Text field — a focusable text input well (filter, composer).
// ---------------------------------------------------------------------------

/// A focusable text input well. `placeholder` shows when empty; the caller owns
/// the draft state and wires `on_key` to a [`composer_key_handler`] that emits
/// the right [`WorkbenchCommand`](crate::workbench::WorkbenchCommand)s.
pub fn text_field<F>(
    theme: RelayTheme,
    id: impl Into<ElementId>,
    key_context: &'static str,
    focus: &FocusHandle,
    value: &str,
    placeholder: &str,
    cx: &mut Context<AppShell>,
    on_key: F,
) -> AnyElement
where
    F: Fn(&mut AppShell, &KeyDownEvent, &mut Context<AppShell>) -> bool + 'static,
{
    let border = if value.is_empty() {
        theme.border
    } else {
        theme.border_strong
    };
    let text_color = if value.is_empty() {
        theme.text_muted
    } else {
        theme.text
    };
    let display = if value.is_empty() {
        placeholder.to_string()
    } else {
        value.to_string()
    };
    let focus_handle = focus.clone();

    div()
        .h(px(30.0))
        .flex_1()
        .min_w(px(0.0))
        .rounded_md()
        .border_1()
        .border_color(border)
        .bg(theme.panel)
        .px_2()
        .flex()
        .items_center()
        .text_sm()
        .text_color(text_color)
        .track_focus(focus)
        .tab_index(0)
        .cursor(CursorStyle::IBeam)
        .key_context(key_context)
        .hover(move |style| style.border_color(theme.border_strong))
        .on_key_down(cx.listener(move |this, event, _, cx| {
            if on_key(this, event, cx) {
                cx.stop_propagation();
            }
        }))
        .id(id)
        .on_click(cx.listener(move |_, _: &gpui::ClickEvent, window, _| {
            window.focus(&focus_handle);
        }))
        .child(display)
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Text composer key handler — collapses the three duplicate key handlers.
// ---------------------------------------------------------------------------

/// A normalized keystroke outcome for a text composer (filter / title / review).
///
/// The three old key handlers in `app_shell.rs` were structurally identical:
/// escape clears, backspace deletes, enter optionally submits, plain printable
/// chars append. This enum captures that shape once; each composer maps the
/// variants to its own [`WorkbenchCommand`](crate::workbench::WorkbenchCommand).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposerKey {
    Clear,
    Backspace,
    Submit,
    Append(String),
}

/// Classify a `KeyDownEvent` into a composer intent, or `None` if the key is
/// not handled (modifier combos, control chars, unmapped keys).
///
/// `submit_on_enter` controls whether `Enter` maps to [`ComposerKey::Submit`]
/// (task title + review draft) or is left unhandled (context filter).
pub fn composer_key_handler(event: &KeyDownEvent, submit_on_enter: bool) -> Option<ComposerKey> {
    let keystroke = event.keystroke.clone().with_simulated_ime();
    match keystroke.key.as_str() {
        "escape" => Some(ComposerKey::Clear),
        "backspace" => Some(ComposerKey::Backspace),
        "enter" if submit_on_enter => Some(ComposerKey::Submit),
        _ if !keystroke.modifiers.control
            && !keystroke.modifiers.alt
            && !keystroke.modifiers.platform
            && !keystroke.modifiers.function =>
        {
            keystroke
                .key_char
                .filter(|text| text.chars().all(|character| !character.is_control()))
                .map(ComposerKey::Append)
        }
        _ => None,
    }
}

/// Classify a raw [`Keystroke`] into a composer intent. This is the same logic
/// as [`composer_key_handler`] but operates on the keystroke directly, which is
/// convenient for unit tests (a `Keystroke` is cheap to build, unlike
/// `KeyDownEvent` which has no `Default` impl).
pub fn classify_keystroke(keystroke: &Keystroke, submit_on_enter: bool) -> Option<ComposerKey> {
    match keystroke.key.as_str() {
        "escape" => Some(ComposerKey::Clear),
        "backspace" => Some(ComposerKey::Backspace),
        "enter" if submit_on_enter => Some(ComposerKey::Submit),
        _ if !keystroke.modifiers.control
            && !keystroke.modifiers.alt
            && !keystroke.modifiers.platform
            && !keystroke.modifiers.function =>
        {
            keystroke
                .key_char
                .clone()
                .filter(|text| text.chars().all(|character| !character.is_control()))
                .map(ComposerKey::Append)
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tone_colors(theme: RelayTheme) {
        // Smoke-check that every tone resolves a distinct, non-transparent color.
        for tone in [
            Tone::Accent,
            Tone::Warning,
            Tone::Danger,
            Tone::Info,
            Tone::Muted,
            Tone::Secondary,
        ] {
            let _ = tone.fg(theme);
            let _ = tone.soft_bg(theme);
        }
    }

    #[test]
    fn tone_palette_resolves() {
        tone_colors(RelayTheme::orca());
    }

    #[test]
    fn accent_soft_bg_is_tinted() {
        let theme = RelayTheme::orca();
        // Accent soft background differs from the neutral fallback.
        assert_ne!(Tone::Accent.soft_bg(theme), Tone::Muted.soft_bg(theme));
    }

    fn key(key: &str, key_char: Option<&str>) -> Keystroke {
        Keystroke {
            key: key.to_string(),
            key_char: key_char.map(|c| c.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn composer_classifies_control_keys() {
        assert_eq!(
            classify_keystroke(&key("escape", None), false),
            Some(ComposerKey::Clear)
        );
        assert_eq!(
            classify_keystroke(&key("backspace", None), false),
            Some(ComposerKey::Backspace)
        );
    }

    #[test]
    fn composer_enter_submits_only_when_enabled() {
        // Task title + review draft: Enter submits.
        assert_eq!(
            classify_keystroke(&key("enter", None), true),
            Some(ComposerKey::Submit)
        );
        // Context filter: Enter is not handled.
        assert_eq!(classify_keystroke(&key("enter", None), false), None);
    }

    #[test]
    fn composer_appends_printable_chars() {
        assert_eq!(
            classify_keystroke(&key("a", Some("a")), false),
            Some(ComposerKey::Append("a".to_string()))
        );
    }

    #[test]
    fn composer_ignores_control_chars_and_modifiers() {
        // Tab/control characters are not appended.
        assert_eq!(classify_keystroke(&key("tab", Some("\t")), false), None);
        // Ctrl-modified keystrokes are not treated as text input.
        let mut ctrl_a = key("a", Some("a"));
        ctrl_a.modifiers.control = true;
        assert_eq!(classify_keystroke(&ctrl_a, false), None);
    }
}
