use crate::harness::{install_test_globals, keys, open_focused, selector, settle_visual};
use gpui::{Modifiers, TestAppContext, VisualTestContext};
use jayjay_core::shortcuts::{Chord, KeyPlatform, ShortcutCommand};
use jayjay_gpui::app::actions::{OpenKeyboardShortcuts, Refresh};
use jayjay_gpui::app::{config, menus};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::windows::keyboard_shortcuts::KeyboardShortcutsView;
use jayjay_gpui::windows::settings::{SettingsSection, SettingsView};
use jj_test::LinearFixture;
use std::cell::Cell;
use std::rc::Rc;

#[gpui::test]
fn shortcut_action_opens_complete_reference_window(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| {
        menus::install(cx);
        cx.dispatch_action(&OpenKeyboardShortcuts);
    });

    let any_window = cx.windows().last().copied().expect("shortcuts window");
    assert!(any_window.downcast::<KeyboardShortcutsView>().is_some());
    let mut shortcuts_cx = VisualTestContext::from_window(any_window, cx);
    settle_visual(&mut shortcuts_cx);

    for selector in [
        "keyboard-shortcuts-window",
        "keyboard-shortcuts-scroll",
        "shortcut-section-general",
        "shortcut-entry-general-3",
        "shortcut-section-diff-review",
        "shortcut-entry-navigation-1",
        "shortcut-entry-view-0",
    ] {
        assert!(
            shortcuts_cx.debug_bounds(selector).is_some(),
            "missing shortcut reference selector {selector}"
        );
    }
}

#[gpui::test]
fn mod_slash_opens_keyboard_shortcuts(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    cx.update(menus::install);
    let (_view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);
    let before = cx.cx.windows().len();

    cx.simulate_keystrokes(if cfg!(target_os = "macos") {
        "cmd-/"
    } else {
        "ctrl-/"
    });
    settle_visual(cx);

    assert_eq!(cx.cx.windows().len(), before + 1);
    assert!(
        cx.cx
            .windows()
            .last()
            .and_then(|window| window.downcast::<KeyboardShortcutsView>())
            .is_some()
    );
}

fn click(cx: &mut VisualTestContext, id: &str) {
    let bounds = cx.debug_bounds(selector(id.to_owned())).expect(id);
    cx.simulate_click(bounds.center(), Modifiers::default());
    settle_visual(cx);
}

fn chord(spec: &str) -> String {
    Chord::parse_default(spec, KeyPlatform::current())
        .unwrap()
        .to_string()
}

fn stored(cx: &mut VisualTestContext, id: &str) -> Option<Vec<String>> {
    cx.cx
        .update(|cx| config::current(cx).shortcuts.get(id).cloned())
}

fn settings(cx: &mut TestAppContext) -> VisualTestContext {
    cx.update(|cx| SettingsView::open_section(SettingsSection::KeyboardShortcuts, cx));
    let window = *cx.windows().last().unwrap();
    let mut view = VisualTestContext::from_window(window, cx);
    settle_visual(&mut view);
    view
}

fn search(cx: &mut VisualTestContext, query: &str) {
    click(cx, "shortcuts-query");
    keys(cx, &chord("mod-a"));
    cx.simulate_input(query);
    settle_visual(cx);
}

#[gpui::test]
fn recorder_rebinds_refresh_and_removes_old_dispatch(cx: &mut TestAppContext) {
    install_test_globals(cx);
    let calls = Rc::new(Cell::new(0));
    cx.update(|cx| {
        let calls = calls.clone();
        cx.on_action(move |_: &Refresh, _| calls.set(calls.get() + 1));
    });
    let mut settings = settings(cx);
    let old = chord("mod-r");
    keys(&mut settings, &old);
    assert_eq!(calls.get(), 1);
    search(&mut settings, "Refresh");
    click(&mut settings, "shortcut-edit-repo.refresh");
    keys(&mut settings, "f8");
    keys(&mut settings, "f8");
    assert_eq!(calls.get(), 2);
    keys(&mut settings, &old);
    assert_eq!(calls.get(), 2, "old chord must stop dispatching");
    settings.cx.update(KeyboardShortcutsView::open);
    let window = *settings.cx.windows().last().unwrap();
    let mut guide = VisualTestContext::from_window(window, &settings.cx);
    settle_visual(&mut guide);
    assert!(guide.debug_bounds("shortcut-key-general-2-0-F8").is_some());
    assert!(
        guide.debug_bounds("shortcut-entry-repository-5").is_none(),
        "unbound Fetch must not render a guide row"
    );
    guide.cx.update(|cx| {
        config::update(cx, |cfg| {
            cfg.shortcuts
                .insert("repo.fetch".to_owned(), vec!["f9".to_owned()]);
            cfg.shortcuts.insert("repo.refresh".to_owned(), Vec::new());
            for command in [
                ShortcutCommand::MergeAcceptLeft,
                ShortcutCommand::MergeAcceptRight,
                ShortcutCommand::MergePreviousConflict,
                ShortcutCommand::MergeNextConflict,
            ] {
                cfg.shortcuts.insert(command.id().to_owned(), Vec::new());
            }
        })
    });
    settle_visual(&mut guide);
    assert!(
        guide
            .debug_bounds("shortcut-key-repository-5-0-F9")
            .is_some()
    );
    assert!(guide.debug_bounds("shortcut-entry-general-2").is_none());
    assert!(guide.debug_bounds("shortcut-section-conflicts").is_none());
}

#[gpui::test]
fn recorder_captures_before_dispatch_and_escape_preserves_settings(cx: &mut TestAppContext) {
    install_test_globals(cx);
    let calls = Rc::new(Cell::new(0));
    cx.update(|cx| {
        let calls = calls.clone();
        cx.on_action(move |_: &Refresh, _| calls.set(calls.get() + 1));
    });
    let mut settings = settings(cx);
    let before = settings
        .cx
        .update(|cx| config::current(cx).shortcuts.clone());
    search(&mut settings, "Refresh");
    click(&mut settings, "shortcut-edit-repo.refresh");
    keys(&mut settings, &chord("mod-r"));
    assert_eq!(calls.get(), 0, "recorded chord dispatched Refresh");
    click(&mut settings, "shortcut-edit-repo.refresh");
    click(&mut settings, "shortcuts-query");
    keys(&mut settings, "end backspace");
    assert!(
        settings
            .debug_bounds("shortcut-edit-repo.refresh")
            .is_some(),
        "leaving the cell must stop recording"
    );
    click(&mut settings, "shortcut-edit-repo.refresh");
    keys(&mut settings, "escape");
    assert_eq!(
        settings
            .cx
            .update(|cx| config::current(cx).shortcuts.clone()),
        before
    );
    assert!(settings.debug_bounds("shortcuts-cancel").is_none());
}

#[gpui::test]
fn replacement_survives_config_reload_and_reset_reports_conflict(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| {
        config::update(cx, |cfg| {
            cfg.shortcuts
                .insert("future.action".to_owned(), vec!["alt-f9".to_owned()]);
        })
    });
    let mut settings = settings(cx);
    search(&mut settings, "Refresh");
    click(&mut settings, "shortcut-edit-repo.refresh");
    keys(&mut settings, &chord("mod-l"));
    assert!(settings.debug_bounds("shortcuts-replace").is_some());
    click(&mut settings, "shortcuts-replace");
    settings.cx.update(|cx| {
        let saved = toml::to_string(config::current(cx).as_ref()).unwrap();
        let loaded: config::AppConfig = toml::from_str(&saved).unwrap();
        assert_eq!(
            loaded.shortcuts.get("future.action"),
            Some(&vec!["alt-f9".to_owned()])
        );
        assert_eq!(
            loaded.shortcuts.get("repo.filter_by_revset"),
            Some(&Vec::new())
        );
        config::update(cx, |cfg| *cfg = loaded);
    });
    search(&mut settings, "Filter by Revset");
    click(&mut settings, "shortcut-reset-repo.filter_by_revset");
    assert!(settings.debug_bounds("shortcuts-replace").is_some());
    click(&mut settings, "shortcuts-cancel");
    settings.cx.update(|cx| {
        assert_eq!(
            config::current(cx).shortcuts.get("repo.filter_by_revset"),
            Some(&Vec::new())
        )
    });
}

#[gpui::test]
fn abandon_binding_requires_changes_focus(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);
    cx.cx.update(|cx| {
        config::update(cx, |cfg| {
            cfg.shortcuts.insert(
                ShortcutCommand::ChangesAbandon.id().to_owned(),
                vec!["f8".to_owned()],
            );
        })
    });
    settle_visual(cx);
    keys(cx, "f8");
    assert!(
        cx.debug_bounds("confirmation-submit").is_some(),
        "Changes list must receive the chord"
    );
    keys(cx, "escape");
    click(cx, "diff-pane-focus");
    keys(cx, "f8");
    assert!(
        cx.debug_bounds("confirmation-submit").is_none(),
        "reading the diff must not abandon the change"
    );
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    assert!(view.read_with(cx, |view, _| view.diff_edit_active()));
    keys(cx, "f8");
    assert!(
        cx.debug_bounds("confirmation-submit").is_none(),
        "diff focus must not abandon the change"
    );
}

#[gpui::test]
fn default_find_binding_opens_diff_search(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);
    keys(cx, &chord("mod-f"));
    assert_eq!(
        view.read_with(cx, |view, _| view.find_query_text().map(str::to_owned)),
        Some(String::new())
    );
}

#[gpui::test]
fn keyboard_controls_unbind_and_reset(cx: &mut TestAppContext) {
    install_test_globals(cx);
    let mut settings = settings(cx);
    keys(&mut settings, "tab");
    settings.simulate_input("Refresh");
    settle_visual(&mut settings);
    assert!(
        settings
            .debug_bounds("shortcut-edit-app.open_repository")
            .is_none()
    );
    keys(&mut settings, "tab tab tab enter");
    keys(&mut settings, "f8");
    assert_eq!(
        stored(&mut settings, "repo.refresh"),
        Some(vec!["f8".to_owned()])
    );
    click(&mut settings, "shortcut-edit-repo.refresh");
    keys(&mut settings, "delete");
    assert_eq!(stored(&mut settings, "repo.refresh"), Some(Vec::new()));
    click(&mut settings, "shortcuts-modified");
    assert!(
        settings
            .debug_bounds("shortcut-edit-repo.refresh")
            .is_some()
    );
    assert!(settings.debug_bounds("shortcuts-reset-all").is_none());
    click(&mut settings, "shortcuts-query");
    keys(&mut settings, &format!("{} backspace", chord("mod-a")));
    click(&mut settings, "shortcuts-reset-all");
    assert!(
        settings
            .debug_bounds("shortcut-edit-repo.refresh")
            .is_none()
    );
    assert!(
        settings
            .cx
            .update(|cx| config::current(cx).shortcuts.is_empty())
    );
}

#[gpui::test]
fn diff_editor_shortcuts_share_chords_with_repository_actions(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);
    for (command, collapsed) in [
        (ShortcutCommand::DiffEditorExpandAllFiles, false),
        (ShortcutCommand::DiffEditorCollapseAllFiles, true),
    ] {
        cx.cx.update(|cx| {
            config::update(cx, |cfg| {
                cfg.shortcuts.clear();
                for command in [ShortcutCommand::RepoToggleSidebar, command] {
                    cfg.shortcuts
                        .insert(command.id().to_owned(), vec!["f8".to_owned()]);
                }
            });
        });
        settle_visual(cx);
        keys(cx, "f8");
        assert!(
            view.read_with(cx, |view, _| view.sidebar_hidden()),
            "{command:?} swallowed the repository shortcut"
        );
        keys(cx, "f8");
        assert!(!view.read_with(cx, |view, _| view.sidebar_hidden()));
        view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
        settle_visual(cx);
        view.update_in(cx, |view, _, cx| {
            if collapsed {
                view.expand_all_diff_edit(cx);
            } else {
                view.collapse_all_diff_edit(cx);
            }
        });
        settle_visual(cx);
        assert_ne!(
            view.read_with(cx, |view, _| view.diff_edit_collapsed("wip1.txt")),
            collapsed
        );
        keys(cx, "f8");
        assert_eq!(
            view.read_with(cx, |view, _| view.diff_edit_collapsed("wip1.txt")),
            collapsed
        );
        assert!(!view.read_with(cx, |view, _| view.sidebar_hidden()));
        view.update_in(cx, |view, _, cx| view.exit_diff_edit(cx));
        settle_visual(cx);
        keys(cx, "f8");
        assert!(view.read_with(cx, |view, _| view.sidebar_hidden()));
        keys(cx, "f8");
    }
}
