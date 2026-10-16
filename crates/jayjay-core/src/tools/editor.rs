/// External editors recognized by the "Open in Editor" actions.
/// Shared IDs mirror SwiftUI's `AppSettings.ExternalEditor`; system default uses the host platform.
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

    pub(super) fn opens_at_line(self) -> bool {
        !matches!(self, Self::SystemDefault | Self::Custom)
    }

    /// Trailing arguments naming `path`, positioned at the 1-based `line` when the editor has a syntax for it.
    pub(super) fn target_args(self, path: &str, line: Option<u32>) -> Vec<String> {
        let Some(line) = line else {
            return vec![path.to_owned()];
        };
        match self {
            Self::VsCode | Self::VsCodium | Self::Cursor if !has_colon_after_drive(path) => {
                vec!["-g".to_owned(), format!("{path}:{line}")]
            }
            Self::Zed | Self::Sublime if !has_colon_after_drive(path) => {
                vec![format!("{path}:{line}")]
            }
            Self::Xcode | Self::Kate => vec!["-l".to_owned(), line.to_string(), path.to_owned()],
            Self::GnomeText => vec![path.to_owned(), format!("+{line}")],
            Self::Vim | Self::Neovim => vec![format!("+{line}"), path.to_owned()],
            _ => vec![path.to_owned()],
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

/// `file:line` parsers read any numeric colon segment as a position, and have no escape for colons in the path itself.
fn has_colon_after_drive(path: &str) -> bool {
    let bytes = path.as_bytes();
    let rest = if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        &path[2..]
    } else {
        path
    };
    rest.contains(':')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_args_follow_each_editor_line_syntax() {
        let at_line = |editor: Editor| editor.target_args("/r/a.rs", Some(7));
        assert_eq!(at_line(Editor::Cursor), ["-g", "/r/a.rs:7"]);
        assert_eq!(at_line(Editor::Zed), ["/r/a.rs:7"]);
        assert_eq!(at_line(Editor::Xcode), ["-l", "7", "/r/a.rs"]);
        assert_eq!(at_line(Editor::Neovim), ["+7", "/r/a.rs"]);
        assert_eq!(at_line(Editor::GnomeText), ["/r/a.rs", "+7"]);
        assert_eq!(at_line(Editor::Custom), ["/r/a.rs"]);
        assert_eq!(Editor::Zed.target_args("/r/a.rs", None), ["/r/a.rs"]);
    }

    #[test]
    fn colon_suffix_editors_fall_back_to_plain_open_for_colon_paths() {
        assert_eq!(
            Editor::VsCode.target_args("/r/report:2026", Some(7)),
            ["/r/report:2026"]
        );
        assert_eq!(Editor::Zed.target_args("/r/a::b", Some(7)), ["/r/a::b"]);
        assert_eq!(
            Editor::VsCode.target_args(r"C:\r\a.rs", Some(7)),
            ["-g", r"C:\r\a.rs:7"]
        );
    }
}
