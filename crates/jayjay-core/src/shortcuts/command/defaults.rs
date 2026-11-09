use super::ShortcutCommand;
use crate::shortcuts::{Chord, KeyPlatform};

impl ShortcutCommand {
    pub fn default_chord(self, platform: KeyPlatform) -> Option<Chord> {
        if self.is_locked() {
            return self.locked_primary(platform);
        }
        let spec = match self {
            Self::AppOpenRepository => "mod-o",
            Self::AppKeyboardShortcuts => "mod-/",
            Self::AppZoomIn => "mod-+",
            Self::AppZoomOut => "mod--",
            Self::AppResetZoom => "mod-0",
            Self::RepoCommandPalette => "shift-mod-p",
            Self::RepoRefresh => "mod-r",
            Self::RepoToggleSidebar => match platform {
                KeyPlatform::Mac => "ctrl-cmd-s",
                KeyPlatform::Other => "ctrl-alt-s",
            },
            Self::RepoOperationLog => "shift-mod-u",
            Self::RepoFilterByRevset => "mod-l",
            Self::RepoBookmarkManager => "shift-mod-b",
            Self::RepoOverview => "shift-mod-o",
            Self::RepoShowInFileManager => "alt-mod-f",
            Self::RepoFindInDiff => "mod-f",
            Self::OverviewFilterLanes => "mod-f",
            Self::DiffEditorExpandAllFiles => "alt-mod-e",
            Self::DiffEditorCollapseAllFiles => "alt-mod-c",
            Self::TextSaveFile => "mod-s",
            Self::TextSaveReviewNote => "mod-enter",
            Self::MergeAcceptLeft => "alt-left",
            Self::MergeAcceptRight => "alt-right",
            Self::MergePreviousConflict => "alt-up",
            Self::MergeNextConflict => "alt-down",
            _ => return None,
        };
        Chord::parse_default(spec, platform).ok()
    }

    pub fn aliases(self, platform: KeyPlatform) -> Vec<Chord> {
        let mut chords = Vec::new();
        if let Some(rest) = self.locked_alias_specs(platform) {
            for spec in rest {
                if let Ok(chord) = Chord::parse_default(spec, platform) {
                    chords.push(chord);
                }
            }
        }
        chords
    }

    pub fn is_locked(self) -> bool {
        matches!(
            self,
            Self::AppSettings
                | Self::AppQuit
                | Self::AppCloseWindow
                | Self::RepoCopyDiffSelection
                | Self::ChangesNext
                | Self::ChangesPrevious
                | Self::FilesNext
                | Self::FilesPrevious
                | Self::RepoSwitchPane
                | Self::RepoCyclePanes
                | Self::RepoOpenContextMenu
                | Self::FilesMarkReviewed
                | Self::DiffNextPreviousFileCard
                | Self::DiffCollapseExpandFileCard
                | Self::DiffToggleFileCard
        )
    }

    fn locked_primary(self, platform: KeyPlatform) -> Option<Chord> {
        let spec = match self {
            Self::AppSettings => "mod-,",
            Self::AppQuit => "mod-q",
            Self::AppCloseWindow => "mod-w",
            Self::RepoCopyDiffSelection => "mod-c",
            Self::ChangesNext | Self::FilesNext => "j",
            Self::ChangesPrevious | Self::FilesPrevious => "k",
            Self::RepoSwitchPane => "h",
            Self::RepoCyclePanes => "tab",
            Self::RepoOpenContextMenu => "ctrl-enter",
            Self::FilesMarkReviewed => "space",
            Self::DiffNextPreviousFileCard => "j",
            Self::DiffCollapseExpandFileCard => "left",
            Self::DiffToggleFileCard => "enter",
            _ => return None,
        };
        Chord::parse_default(spec, platform).ok()
    }

    fn locked_alias_specs(self, platform: KeyPlatform) -> Option<&'static [&'static str]> {
        match self {
            Self::ChangesNext => Some(&["down", "ctrl-n"]),
            Self::ChangesPrevious => Some(&["up", "ctrl-p"]),
            Self::FilesNext => Some(&["down"]),
            Self::FilesPrevious => Some(&["up"]),
            Self::DiffNextPreviousFileCard => Some(&["k"]),
            Self::DiffCollapseExpandFileCard => Some(&["right"]),
            Self::RepoSwitchPane => Some(&["l"]),
            Self::RepoOpenContextMenu if platform == KeyPlatform::Other => {
                Some(&["shift-f10", "menu"])
            }
            _ => None,
        }
    }
}
