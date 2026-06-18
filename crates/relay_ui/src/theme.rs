use gpui::{Hsla, rgb};

#[derive(Debug, Clone, Copy)]
pub struct RelayTheme {
    pub bg: Hsla,
    pub chrome: Hsla,
    pub chrome_alt: Hsla,
    pub panel: Hsla,
    pub panel_alt: Hsla,
    pub terminal_bg: Hsla,
    pub terminal_text: Hsla,
    pub line: Hsla,
    pub selection: Hsla,
    pub selection_line: Hsla,
    pub text: Hsla,
    pub muted: Hsla,
    pub accent: Hsla,
    pub warning: Hsla,
    pub danger: Hsla,
}

impl RelayTheme {
    pub fn dark() -> Self {
        Self {
            bg: rgb(0xf7f7f6).into(),
            chrome: rgb(0xf1f1f0).into(),
            chrome_alt: rgb(0xe7e7e5).into(),
            panel: rgb(0xffffff).into(),
            panel_alt: rgb(0x252a32).into(),
            terminal_bg: rgb(0x252a32).into(),
            terminal_text: rgb(0xf3f4f6).into(),
            line: rgb(0xd6d6d3).into(),
            selection: rgb(0xdfdfdd).into(),
            selection_line: rgb(0xc9c9c5).into(),
            text: rgb(0x202124).into(),
            muted: rgb(0x73777f).into(),
            accent: rgb(0x10b981).into(),
            warning: rgb(0xb8871f).into(),
            danger: rgb(0xb5524b).into(),
        }
    }
}
