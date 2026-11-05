use super::ShortcutCommand;
use crate::shortcuts::ShortcutContext;

impl ShortcutCommand {
    pub fn context(self) -> ShortcutContext {
        match self {
            Self::AppOpenRepository
            | Self::AppKeyboardShortcuts
            | Self::AppZoomIn
            | Self::AppZoomOut
            | Self::AppResetZoom
            | Self::AppSettings
            | Self::AppQuit
            | Self::AppCloseWindow => ShortcutContext::App,
            Self::RepoCommandPalette
            | Self::RepoRefresh
            | Self::RepoToggleSidebar
            | Self::RepoUndoLastOperation
            | Self::RepoFilterByRevset
            | Self::RepoBookmarkManager
            | Self::RepoOverview
            | Self::RepoShowInFileManager
            | Self::RepoFindInDiff
            | Self::RepoFetch
            | Self::RepoPush
            | Self::RepoNewWorkspace
            | Self::RepoImportPullRequest
            | Self::RepoRebaseOntoTrunk
            | Self::RepoBisect
            | Self::RepoCleanUpStaleBookmarks
            | Self::RepoToggleSideBySide
            | Self::RepoToggleIgnoreWhitespace
            | Self::RepoToggleTreeFileList
            | Self::RepoNewChange
            | Self::RepoEditChange
            | Self::RepoDescribeChange
            | Self::RepoSquashChange
            | Self::RepoNewBookmark
            | Self::RepoShowEvolution
            | Self::RepoSwitchPane
            | Self::RepoCyclePanes
            | Self::RepoOpenContextMenu => ShortcutContext::RepositoryWindow,
            Self::OverviewFilterLanes => ShortcutContext::Overview,
            Self::DiffEditorExpandAllFiles | Self::DiffEditorCollapseAllFiles => {
                ShortcutContext::DiffEditor
            }
            Self::TextSaveFile | Self::TextSaveReviewNote => ShortcutContext::TextEditor,
            Self::MergeAcceptLeft
            | Self::MergeAcceptRight
            | Self::MergePreviousConflict
            | Self::MergeNextConflict => ShortcutContext::MergeEditor,
            Self::ChangesAbandon | Self::ChangesNext | Self::ChangesPrevious => {
                ShortcutContext::ChangesList
            }
            Self::FilesNext | Self::FilesPrevious | Self::FilesMarkReviewed => {
                ShortcutContext::FileList
            }
            Self::RepoCopyDiffSelection
            | Self::DiffNextPreviousFileCard
            | Self::DiffCollapseExpandFileCard
            | Self::DiffToggleFileCard => ShortcutContext::Diff,
        }
    }
}
