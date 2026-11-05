use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Char(char),
    F(u8),
    Enter,
    Backspace,
    Delete,
    Space,
    Tab,
    Escape,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Menu,
}

impl Key {
    pub(super) fn write_token(self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Char(c) => write!(f, "{c}"),
            Self::F(n) => write!(f, "f{n}"),
            Self::Enter => f.write_str("enter"),
            Self::Backspace => f.write_str("backspace"),
            Self::Delete => f.write_str("delete"),
            Self::Space => f.write_str("space"),
            Self::Tab => f.write_str("tab"),
            Self::Escape => f.write_str("escape"),
            Self::Up => f.write_str("up"),
            Self::Down => f.write_str("down"),
            Self::Left => f.write_str("left"),
            Self::Right => f.write_str("right"),
            Self::Home => f.write_str("home"),
            Self::End => f.write_str("end"),
            Self::PageUp => f.write_str("pageup"),
            Self::PageDown => f.write_str("pagedown"),
            Self::Menu => f.write_str("menu"),
        }
    }

    pub(super) fn from_token(token: &str) -> Option<Self> {
        let mut chars = token.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            if c.is_whitespace() || c.is_control() {
                return None;
            }
            let mut lower = c.to_lowercase();
            return match (lower.next(), lower.next()) {
                (Some(lower), None) => Some(Self::Char(lower)),
                _ => Some(Self::Char(c)),
            };
        }
        match token {
            "enter" | "return" => Some(Self::Enter),
            "backspace" => Some(Self::Backspace),
            "delete" => Some(Self::Delete),
            "space" => Some(Self::Space),
            "tab" => Some(Self::Tab),
            "escape" | "esc" => Some(Self::Escape),
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "home" => Some(Self::Home),
            "end" => Some(Self::End),
            "pageup" => Some(Self::PageUp),
            "pagedown" => Some(Self::PageDown),
            "menu" => Some(Self::Menu),
            "f1" => Some(Self::F(1)),
            "f2" => Some(Self::F(2)),
            "f3" => Some(Self::F(3)),
            "f4" => Some(Self::F(4)),
            "f5" => Some(Self::F(5)),
            "f6" => Some(Self::F(6)),
            "f7" => Some(Self::F(7)),
            "f8" => Some(Self::F(8)),
            "f9" => Some(Self::F(9)),
            "f10" => Some(Self::F(10)),
            "f11" => Some(Self::F(11)),
            "f12" => Some(Self::F(12)),
            _ => None,
        }
    }
}
