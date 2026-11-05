use super::actions::*;
use gpui::Action;
use jayjay_core::shortcuts::ShortcutCommand;

pub(crate) trait ShortcutAction {
    fn gpui_action(self) -> Option<Box<dyn Action>>;
    fn key_context(self) -> Option<&'static str>;
}

impl ShortcutAction for ShortcutCommand {
    fn gpui_action(self) -> Option<Box<dyn Action>> {
        Some(match self {
            Self::AppOpenRepository => Box::new(OpenRepository),
            Self::AppKeyboardShortcuts => Box::new(OpenKeyboardShortcuts),
            Self::AppZoomIn => Box::new(ZoomIn),
            Self::AppZoomOut => Box::new(ZoomOut),
            Self::AppResetZoom => Box::new(ResetZoom),
            Self::AppSettings => Box::new(OpenSettings),
            Self::AppQuit => Box::new(Quit),
            Self::AppCloseWindow => Box::new(CloseWindow),
            Self::RepoCommandPalette => Box::new(OpenCommandPalette),
            Self::RepoRefresh => Box::new(Refresh),
            Self::RepoToggleSidebar => Box::new(ToggleSidebar),
            Self::RepoUndoLastOperation => Box::new(OpenOperationLog),
            Self::RepoFilterByRevset => Box::new(FilterByRevset),
            Self::RepoBookmarkManager => Box::new(OpenBookmarkManager),
            Self::RepoOverview => Box::new(OpenOverview),
            Self::RepoShowInFileManager => Box::new(ShowRepoInFileManager),
            Self::RepoFindInDiff => Box::new(OpenFind),
            Self::RepoFetch => Box::new(GitFetchOrigin),
            Self::RepoPush => Box::new(GitPushDefault),
            Self::RepoNewWorkspace => Box::new(NewWorkspace),
            Self::RepoImportPullRequest => Box::new(NewWorkspaceFromPullRequest),
            Self::RepoRebaseOntoTrunk => Box::new(RebaseOntoTrunk),
            Self::RepoBisect => Box::new(OpenBisect),
            Self::RepoCleanUpStaleBookmarks => Box::new(ForgetStaleBookmarks),
            Self::RepoToggleSideBySide => Box::new(ToggleSideBySideDiff),
            Self::RepoToggleIgnoreWhitespace => Box::new(ToggleIgnoreWhitespace),
            Self::RepoToggleTreeFileList => Box::new(ToggleTreeFileList),
            Self::RepoNewChange => Box::new(NewChange),
            Self::RepoEditChange => Box::new(EditChange),
            Self::RepoDescribeChange => Box::new(DescribeChange),
            Self::RepoSquashChange => Box::new(SquashChange),
            Self::RepoNewBookmark => Box::new(NewBookmark),
            Self::RepoShowEvolution => Box::new(ShowEvolution),
            Self::RepoCopyDiffSelection => Box::new(CopyDiffSelection),
            Self::OverviewFilterLanes => Box::new(OpenOverviewFilter),
            Self::DiffEditorExpandAllFiles => Box::new(DiffEditExpandAll),
            Self::DiffEditorCollapseAllFiles => Box::new(DiffEditCollapseAll),
            Self::TextSaveFile => Box::new(SaveFileEditor),
            Self::TextSaveReviewNote => Box::new(SaveNoteComposer),
            Self::MergeAcceptLeft => Box::new(MergeUseLeftHunk),
            Self::MergeAcceptRight => Box::new(MergeUseRightHunk),
            Self::MergePreviousConflict => Box::new(MergePreviousHunk),
            Self::MergeNextConflict => Box::new(MergeNextHunk),
            Self::ChangesAbandon => Box::new(AbandonChange),
            Self::RepoSwitchPane
            | Self::RepoCyclePanes
            | Self::RepoOpenContextMenu
            | Self::ChangesNext
            | Self::ChangesPrevious
            | Self::FilesNext
            | Self::FilesPrevious
            | Self::FilesMarkReviewed
            | Self::DiffNextPreviousFileCard
            | Self::DiffCollapseExpandFileCard
            | Self::DiffToggleFileCard => return None,
        })
    }

    fn key_context(self) -> Option<&'static str> {
        match self {
            Self::DiffEditorExpandAllFiles | Self::DiffEditorCollapseAllFiles => Some("DiffEditor"),
            Self::TextSaveFile => Some("FileEditor"),
            Self::TextSaveReviewNote => Some("NoteComposer"),
            Self::MergeAcceptLeft
            | Self::MergeAcceptRight
            | Self::MergePreviousConflict
            | Self::MergeNextConflict => Some("MergeHunks"),
            Self::ChangesAbandon => Some("ChangesList"),
            _ => None,
        }
    }
}
