use gpui::{App, IntoElement, RenderOnce, Window};

use crate::{display::Badge, tone::Tone};

/// Small status badge for terminal/session state.
#[derive(IntoElement)]
pub struct TerminalStatusBadge {
    tone: Tone,
}

impl TerminalStatusBadge {
    pub fn new(tone: Tone) -> Self {
        Self { tone }
    }
}

impl RenderOnce for TerminalStatusBadge {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        Badge::new(status_label(self.tone)).tone(self.tone).soft()
    }
}

fn status_label(tone: Tone) -> &'static str {
    match tone {
        Tone::Accent => "LIVE",
        Tone::Warning => "BUSY",
        Tone::Danger => "ERR",
        Tone::Info => "SYNC",
        Tone::Muted | Tone::Secondary => "IDLE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_status_badge_labels_map_running_state() {
        assert_eq!(status_label(Tone::Accent), "LIVE");
    }
}
