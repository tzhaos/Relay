use gpui::{Hsla, rgb};

#[derive(Debug, Clone, Copy)]
pub struct RelayTheme {
    pub bg: Hsla,
    pub chrome: Hsla,
    pub chrome_alt: Hsla,
    pub panel: Hsla,
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
    pub fn orca() -> Self {
        Self {
            bg: rgb(0xf8f8f7).into(),
            chrome: rgb(0xf3f3f2).into(),
            chrome_alt: rgb(0xebebe9).into(),
            panel: rgb(0xffffff).into(),
            terminal_bg: rgb(0x282d35).into(),
            terminal_text: rgb(0xf7f7f4).into(),
            line: rgb(0xdededa).into(),
            selection: rgb(0xe2e2e0).into(),
            selection_line: rgb(0xd1d1cd).into(),
            text: rgb(0x18191c).into(),
            muted: rgb(0x68707a).into(),
            accent: rgb(0x05b978).into(),
            warning: rgb(0xb98208).into(),
            danger: rgb(0xbb5148).into(),
        }
    }
}
