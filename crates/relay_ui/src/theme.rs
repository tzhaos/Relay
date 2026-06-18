use gpui::{Hsla, rgb};

#[derive(Debug, Clone, Copy)]
pub struct RelayTheme {
    pub bg: Hsla,
    pub panel: Hsla,
    pub panel_alt: Hsla,
    pub line: Hsla,
    pub text: Hsla,
    pub muted: Hsla,
    pub accent: Hsla,
    pub warning: Hsla,
    pub danger: Hsla,
}

impl RelayTheme {
    pub fn dark() -> Self {
        Self {
            bg: rgb(0x101113).into(),
            panel: rgb(0x17191c).into(),
            panel_alt: rgb(0x1d2024).into(),
            line: rgb(0x2b3036).into(),
            text: rgb(0xe7eaee).into(),
            muted: rgb(0x9299a3).into(),
            accent: rgb(0x48a6a0).into(),
            warning: rgb(0xc49a3a).into(),
            danger: rgb(0xc45f54).into(),
        }
    }
}
