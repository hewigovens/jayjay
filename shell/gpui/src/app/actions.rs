use super::shortcut_action::ShortcutAction;
use gpui::actions;
use jayjay_core::shortcuts::{EffectiveBindings, KeyPlatform};

actions!(
    jayjay,
    [
        OpenSettings,
        OpenAbout,
        CloseWindow,
        Dismiss,
        Refresh,
        OpenRepository,
        OpenSshRepository,
        CloneRepository,
        OpenCommandPalette,
        OpenKeyboardShortcuts,
        OpenFind,
        CopyDiffSelection,
        OpenUserGuide,
        OpenJujutsuDocumentation,
        ReportIssue,
        SendFeedback,
        RevealLogs,
        OpenBookmarkManager,
        OpenOverview,
        OpenOperationLog,
        FilterByRevset,
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
        ToggleSidebar,
        ZoomIn,
        ZoomOut,
        ResetZoom,
        ClearRecentRepositories,
        Quit,
        SaveNoteComposer,
        SubmitStackedPr,
        NewWorkspace,
        NewWorkspaceFromPullRequest,
        DiffEditExpandAll,
        DiffEditCollapseAll,
        SaveFileEditor,
        MergeUseLeftHunk,
        MergeUseRightHunk,
        MergePreviousHunk,
        OpenOverviewFilter,
        RebaseOntoTrunk,
        OpenBisect,
        NewChange,
        EditChange,
        DescribeChange,
        SquashChange,
        NewBookmark,
        ShowEvolution,
        AbandonChange,
        MergeNextHunk
    ]
);

#[derive(Clone, PartialEq, Debug, gpui::Action)]
#[action(namespace = jayjay, no_json)]
pub struct OpenRecentRepository {
    pub(crate) path: String,
}

pub fn app_key_bindings(
    overrides: &jayjay_core::shortcuts::ShortcutOverrides,
) -> Vec<gpui::KeyBinding> {
    let mut bindings = Vec::new();
    for entry in EffectiveBindings::resolve(overrides, KeyPlatform::current()).entries() {
        if let Some(action) = entry.command.gpui_action() {
            for chord in entry.chords() {
                bindings.push(
                    gpui::KeyBinding::load(
                        &chord.to_string(),
                        action.boxed_clone(),
                        entry.command.key_context().map(|context| {
                            gpui::KeyBindingContextPredicate::parse(context)
                                .expect("shell key context")
                                .into()
                        }),
                        false,
                        None,
                        &gpui::DummyKeyboardMapper,
                    )
                    .expect("registry chord"),
                );
            }
        }
    }
    bindings.push(gpui::KeyBinding::new("escape", Dismiss, None));
    bindings.push(gpui::KeyBinding::new(
        "enter",
        SubmitStackedPr,
        Some("StackedPrPanel && !StackedPrInput"),
    ));
    bindings.extend(crate::ui::text_area::key_bindings(crate::platform::MOD_KEY));
    bindings
}
