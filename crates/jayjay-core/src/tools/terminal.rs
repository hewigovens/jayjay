#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(super) enum Terminal {
    SystemDefault,
    PowerShell,
    ITerm,
    Ghostty,
    GnomeTerminal,
    Konsole,
    LxTerminal,
    Alacritty,
    Kitty,
    Custom,
}

impl Terminal {
    pub(super) fn from_id(id: &str) -> Option<Self> {
        Some(match id {
            "terminal" => Self::SystemDefault,
            "powershell" => Self::PowerShell,
            "iterm" => Self::ITerm,
            "ghostty" => Self::Ghostty,
            "gnome-terminal" => Self::GnomeTerminal,
            "konsole" => Self::Konsole,
            "lxterminal" => Self::LxTerminal,
            "alacritty" => Self::Alacritty,
            "kitty" => Self::Kitty,
            "custom" => Self::Custom,
            _ => return None,
        })
    }
}
