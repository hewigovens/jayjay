/// External editors recognized by the "Open in Editor" actions.
/// Mirrors SwiftUI's `AppSettings.ExternalEditor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Editor {
    VsCode,
    VsCodium,
    Zed,
    Xcode,
    Vim,
    Custom,
}

impl Editor {
    pub(super) fn from_id(id: &str) -> Option<Self> {
        Some(match id {
            "vscode" => Self::VsCode,
            "vscodium" => Self::VsCodium,
            "zed" => Self::Zed,
            "xcode" => Self::Xcode,
            "vim" => Self::Vim,
            "custom" => Self::Custom,
            _ => return None,
        })
    }

    pub(super) fn command(self) -> &'static str {
        match self {
            Self::VsCode => "code",
            Self::VsCodium => "codium",
            Self::Zed => "zed",
            Self::Vim => "vim",
            // Xcode ships `xed` via Command Line Tools; it goes through
            // `xcode-select` so it works regardless of whether the user has
            // `Xcode.app`, `Xcode-26.4.0.app`, or a beta installed.
            Self::Xcode => "xed",
            Self::Custom => "",
        }
    }

    pub(super) fn is_terminal(self) -> bool {
        matches!(self, Self::Vim)
    }
}
