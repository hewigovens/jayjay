/// External editors recognized by the "Open in Editor" actions.
/// Shared IDs mirror SwiftUI's `AppSettings.ExternalEditor`; system default is Linux-specific.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Editor {
    SystemDefault,
    VsCode,
    VsCodium,
    Cursor,
    Zed,
    Xcode,
    Sublime,
    GnomeText,
    Kate,
    Vim,
    Neovim,
    Custom,
}

impl Editor {
    pub(super) fn from_id(id: &str) -> Option<Self> {
        Some(match id {
            "system" => Self::SystemDefault,
            "vscode" => Self::VsCode,
            "vscodium" => Self::VsCodium,
            "cursor" => Self::Cursor,
            "zed" => Self::Zed,
            "xcode" => Self::Xcode,
            "sublime" => Self::Sublime,
            "gnome-text-editor" => Self::GnomeText,
            "kate" => Self::Kate,
            "vim" => Self::Vim,
            "nvim" => Self::Neovim,
            "custom" => Self::Custom,
            _ => return None,
        })
    }

    pub(super) fn command(self) -> &'static str {
        match self {
            Self::SystemDefault => "",
            Self::VsCode => "code",
            Self::VsCodium => "codium",
            Self::Cursor => "cursor",
            Self::Zed => "zed",
            Self::Vim => "vim",
            // Xcode ships `xed` via Command Line Tools; it goes through
            // `xcode-select` so it works regardless of whether the user has
            // `Xcode.app`, `Xcode-26.4.0.app`, or a beta installed.
            Self::Xcode => "xed",
            Self::Sublime => "subl",
            Self::GnomeText => "gnome-text-editor",
            Self::Kate => "kate",
            Self::Custom => "",
            Self::Neovim => "nvim",
        }
    }

    /// Extra arguments prepended before the path when launching.
    pub(super) fn launch_args(self) -> &'static [&'static str] {
        match self {
            // Cursor (a VS Code fork) opens its Agent window by default when
            // launched from the CLI; `--classic` forces the editor layout.
            Self::Cursor => &["--classic"],
            _ => &[],
        }
    }
}
