use super::editor::Editor;
use super::terminal::Terminal;

/// A one-colour app mark that both shells draw on the toolbar's shared tile; `scale` is its optical size relative to a line mark.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolMark {
    pub name: &'static str,
    pub scale: f32,
}

impl ToolMark {
    const fn line(name: &'static str) -> Option<Self> {
        Some(Self { name, scale: 1. })
    }

    const fn solid(name: &'static str) -> Option<Self> {
        Some(Self {
            name,
            scale: 14. / 15.,
        })
    }

    /// A filled square reads heavier than any other shape at tile size.
    const fn square(name: &'static str) -> Option<Self> {
        Some(Self { name, scale: 0.9 })
    }

    /// `None` means the shell draws its generic editor symbol.
    pub fn editor(id: &str) -> Option<Self> {
        match Editor::from_id(id)? {
            Editor::VsCode => Self::line("editor.vscode"),
            Editor::VsCodium => Self::line("editor.vscodium"),
            Editor::Cursor => Self::solid("editor.cursor"),
            Editor::Zed => Self::line("editor.zed"),
            Editor::Vim => Self::line("editor.vim"),
            Editor::Neovim => Self::solid("editor.neovim"),
            Editor::Sublime => Self::solid("editor.sublime"),
            Editor::SystemDefault
            | Editor::Xcode
            | Editor::GnomeText
            | Editor::Kate
            | Editor::Custom => None,
        }
    }

    /// `None` means the shell draws its generic terminal symbol.
    pub fn terminal(id: &str) -> Option<Self> {
        match Terminal::from_id(id)? {
            Terminal::ITerm => Self::square("terminal.iterm"),
            Terminal::Ghostty => Self::solid("terminal.ghostty"),
            Terminal::Rex => Self::line("terminal.rex"),
            Terminal::GnomeTerminal => Self::square("terminal.gnome-terminal"),
            Terminal::Alacritty => Self::solid("terminal.alacritty"),
            Terminal::Kitty => Self::solid("terminal.kitty"),
            Terminal::SystemDefault
            | Terminal::PowerShell
            | Terminal::Konsole
            | Terminal::LxTerminal
            | Terminal::Custom => None,
        }
    }
}
