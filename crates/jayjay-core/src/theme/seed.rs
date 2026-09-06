use super::color::{is_dark, mix};

/// A theme file needs only `background` and `foreground`; the rest derive from them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeSeed {
    pub background: u32,
    pub foreground: u32,
    pub surface: u32,
    pub muted: u32,
    pub border: u32,
    pub accent: u32,
    pub selection: u32,
    pub red: u32,
    pub green: u32,
    pub yellow: u32,
    pub blue: u32,
    pub magenta: u32,
    pub cyan: u32,
    pub orange: u32,
}

impl ThemeSeed {
    pub const ACCENT: u32 = 0x3b82f6;

    pub fn light() -> Self {
        Self {
            background: 0xffffff,
            foreground: 0x1d1d1f,
            surface: 0xf6f6f6,
            muted: 0x6e6e73,
            border: 0xe5e5e5,
            accent: Self::ACCENT,
            selection: 0xe8f1fd,
            red: 0xff3b30,
            green: 0x34c759,
            yellow: 0xffcc00,
            blue: 0x007aff,
            magenta: 0xaf52de,
            cyan: 0x32ade6,
            orange: 0xff9500,
        }
    }

    pub fn dark() -> Self {
        Self {
            background: 0x10131a,
            foreground: 0xe6e6e6,
            surface: 0x1a1f27,
            muted: 0x8a8f99,
            border: 0x252a33,
            accent: Self::ACCENT,
            selection: 0x1f2a3d,
            red: 0xff453a,
            green: 0x30d158,
            yellow: 0xffd60a,
            blue: 0x0a84ff,
            magenta: 0xbf5af2,
            cyan: 0x64d2ff,
            orange: 0xff9f0a,
        }
    }

    pub fn from_pair(background: u32, foreground: u32) -> Self {
        let dark = is_dark(background);
        let system = if dark { Self::dark() } else { Self::light() };
        Self {
            background,
            foreground,
            surface: mix(background, foreground, if dark { 0.05 } else { 0.035 }),
            muted: mix(foreground, background, 0.45),
            border: mix(background, foreground, if dark { 0.12 } else { 0.10 }),
            accent: Self::ACCENT,
            selection: Self::selection_for(background, Self::ACCENT),
            red: system.red,
            green: system.green,
            yellow: system.yellow,
            blue: system.blue,
            magenta: system.magenta,
            cyan: system.cyan,
            orange: system.orange,
        }
    }

    pub fn is_dark(&self) -> bool {
        is_dark(self.background)
    }

    pub fn selection_for(background: u32, accent: u32) -> u32 {
        mix(
            background,
            accent,
            if is_dark(background) { 0.25 } else { 0.12 },
        )
    }

    pub fn tint(&self, color: u32, amount: f32) -> u32 {
        mix(self.background, color, amount)
    }

    /// `color` pulled toward the foreground so it reads as text on `background`.
    pub fn ink(&self, color: u32) -> u32 {
        mix(
            color,
            self.foreground,
            if self.is_dark() { 0.45 } else { 0.5 },
        )
    }

    /// `color` as-is on dark backgrounds and deepened on light ones, for icons.
    pub fn deep(&self, color: u32) -> u32 {
        if self.is_dark() {
            color
        } else {
            mix(color, self.foreground, 0.25)
        }
    }
}
