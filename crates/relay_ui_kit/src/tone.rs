//! Semantic color tone — the meaning a caller assigns to a status indicator.
//!
//! A [`Tone`] maps to theme tokens so badges, dots, and inline labels stay
//! consistent: "running" is always the accent green, "failed" is always the
//! danger red, regardless of which surface paints it.

use gpui::Hsla;

use crate::theme::Theme;

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
    pub fn fg(self, theme: &Theme) -> Hsla {
        match self {
            Tone::Accent => theme.accent,
            Tone::Warning => theme.warning,
            Tone::Danger => theme.danger,
            Tone::Info => theme.info,
            Tone::Muted => theme.text_muted,
            Tone::Secondary => theme.text_secondary,
        }
    }

    /// Soft tinted background for this tone (for filled badges). Only the accent
    /// tone gets a true tint; the rest fall back to the quiet neutral panel so
    /// status color stays sparse and meaningful.
    pub fn soft_bg(self, theme: &Theme) -> Hsla {
        match self {
            Tone::Accent => theme.accent_bg,
            Tone::Warning | Tone::Danger | Tone::Info | Tone::Muted | Tone::Secondary => {
                theme.panel_alt
            }
        }
    }

    /// Soft border for a filled badge of this tone.
    pub fn soft_border(self, theme: &Theme) -> Hsla {
        match self {
            Tone::Accent => theme.accent_border,
            Tone::Warning | Tone::Danger | Tone::Info | Tone::Muted | Tone::Secondary => {
                theme.border
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn every_tone_resolves_a_foreground() {
        let theme = Theme::light();
        for tone in [
            Tone::Accent,
            Tone::Warning,
            Tone::Danger,
            Tone::Info,
            Tone::Muted,
            Tone::Secondary,
        ] {
            // Smoke-check that each tone resolves all three roles without panic.
            let _ = tone.fg(&theme);
            let _ = tone.soft_bg(&theme);
            let _ = tone.soft_border(&theme);
        }
    }

    #[test]
    fn accent_is_the_only_truly_tinted_tone() {
        let theme = Theme::light();
        // Accent gets a real tint; the rest fall back to the neutral panel so
        // status color stays sparse and meaningful.
        assert_eq!(Tone::Accent.soft_bg(&theme), theme.accent_bg);
        assert_eq!(Tone::Warning.soft_bg(&theme), theme.panel_alt);
        assert_eq!(Tone::Danger.soft_bg(&theme), theme.panel_alt);
    }
}
