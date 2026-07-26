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
        SendFeedback,
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
        SaveNoteComposer,
        SubmitStackedPr,
        NewWorkspace,
        DiffEditExpandAll,
        DiffEditCollapseAll
    ]
);

#[derive(Clone, PartialEq, Debug, gpui::Action)]
#[action(namespace = jayjay, no_json)]
pub struct OpenRecentRepository {
    pub path: String,
}

/// The app keymap, shared by `main` and the component-test harness so key dispatch in tests matches production.
pub fn app_key_bindings() -> Vec<gpui::KeyBinding> {
    let mod_key = crate::platform::MOD_KEY;
    let mut key_bindings = vec![
        gpui::KeyBinding::new(format!("{mod_key}-o").as_str(), OpenRepository, None),
        gpui::KeyBinding::new(format!("{mod_key}-,").as_str(), OpenSettings, None),
        gpui::KeyBinding::new(format!("{mod_key}-q").as_str(), Quit, None),
        gpui::KeyBinding::new(format!("{mod_key}-w").as_str(), CloseWindow, None),
        gpui::KeyBinding::new("escape", Dismiss, None),
        gpui::KeyBinding::new(format!("{mod_key}-r").as_str(), Refresh, None),
        gpui::KeyBinding::new(format!("{mod_key}-+").as_str(), ZoomIn, None),
        gpui::KeyBinding::new(format!("{mod_key}--").as_str(), ZoomOut, None),
        gpui::KeyBinding::new(format!("{mod_key}-0").as_str(), ResetZoom, None),
        gpui::KeyBinding::new(
            format!("{mod_key}-shift-p").as_str(),
            OpenCommandPalette,
            None,
        ),
        gpui::KeyBinding::new(
            format!("{mod_key}-shift-b").as_str(),
            OpenBookmarkManager,
            None,
        ),
        gpui::KeyBinding::new(
            format!("{mod_key}-shift-u").as_str(),
            OpenOperationLog,
            None,
        ),
        gpui::KeyBinding::new(
            format!("{mod_key}-alt-f").as_str(),
            ShowRepoInFileManager,
            None,
        ),
        gpui::KeyBinding::new(format!("{mod_key}-f").as_str(), OpenFind, None),
        gpui::KeyBinding::new(format!("{mod_key}-c").as_str(), CopyDiffSelection, None),
        gpui::KeyBinding::new(format!("{mod_key}-alt-e").as_str(), DiffEditExpandAll, None),
        gpui::KeyBinding::new(
            format!("{mod_key}-alt-c").as_str(),
            DiffEditCollapseAll,
            None,
        ),
        // Scoped to the "NoteComposer" key context, not bare "TextArea", so mod+Return saves the note without binding on every other TextArea (commit box, edit description, ...).
        gpui::KeyBinding::new(
            format!("{mod_key}-enter").as_str(),
            SaveNoteComposer,
            Some("NoteComposer"),
        ),
        gpui::KeyBinding::new(
            "enter",
            SubmitStackedPr,
            Some("StackedPrPanel && !StackedPrInput"),
        ),
    ];
    key_bindings.extend(crate::ui::text_area::key_bindings(mod_key));
    key_bindings
}
