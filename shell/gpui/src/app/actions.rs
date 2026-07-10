use gpui::actions;

actions!(
    jayjay,
    [
        OpenSettings,
        OpenAbout,
        CloseWindow,
        Dismiss,
        Refresh,
        OpenRepository,
        OpenCommandPalette,
        OpenFind,
        CopyDiffSelection,
        OpenUserGuide,
        OpenJujutsuDocumentation,
        ReportIssue,
        OpenBookmarkManager,
        OpenOperationLog,
        OpenRepoInEditor,
        OpenRepoInTerminal,
        ShowRepoInFileManager,
        OpenRemoteRepository,
        GitFetchOrigin,
        GitPushDefault,
        ForgetStaleBookmarks,
        ToggleSideBySideDiff,
        ToggleIgnoreWhitespace,
        ToggleHideGitLfsFiles,
        ToggleTreeFileList,
        ZoomIn,
        ZoomOut,
        ResetZoom,
        ClearRecentRepositories,
        Quit,
        SaveNoteComposer
    ]
);

#[derive(Clone, PartialEq, Debug, gpui::Action)]
#[action(namespace = jayjay, no_json)]
pub struct OpenRecentRepository {
    pub path: String,
}
