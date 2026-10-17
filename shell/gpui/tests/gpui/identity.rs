use crate::harness::*;
use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::windows::identity_dialog::IdentityDialog;
use jayjay_gpui::windows::settings::{SettingsSection, SettingsView};
use jj_test::{init_colocated, scoped_env};

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let bounds = cx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("missing {selector}"));
    cx.simulate_click(bounds.center(), Modifiers::default());
    settle_visual(cx);
}

fn displayed_changes(repo: &Entity<RepoWindow>, cx: &mut VisualTestContext) -> usize {
    repo.read_with(cx, |view, cx| {
        view.view_model().read(cx).graph.changes.len()
    })
}

#[gpui::test]
fn set_up_in_settings_saves_the_identity_and_refreshes_open_repos(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().expect("tempdir");
    let jj_config = dir.path().join("jj").join("config.toml");
    let git_config = dir.path().join("gitconfig");
    std::fs::write(&git_config, "[user]\n\tname = Git Name\n").expect("write git config");
    let _env = scoped_env([
        ("JJ_CONFIG", Some(jj_config.as_os_str())),
        ("GIT_CONFIG_GLOBAL", Some(git_config.as_os_str())),
        ("JJ_USER", None),
        ("JJ_EMAIL", None),
    ]);
    init_colocated(&dir.path().join("repo"));

    let (repo, repo_cx) = open_repo(dir.path().join("repo"), cx);
    repo.update_in(repo_cx, |view, _, cx| {
        view.view_model()
            .update(cx, |vm, cx| vm.apply_revset("mine()", cx));
    });
    settle_visual(repo_cx);
    assert!(
        displayed_changes(&repo, repo_cx) > 0,
        "an unset email matches the unauthored changes"
    );

    repo_cx
        .cx
        .update(|cx| SettingsView::open_section(SettingsSection::Jujutsu, cx));
    let window_of = |cx: &TestAppContext, is: fn(&gpui::AnyWindowHandle) -> bool| {
        cx.windows().into_iter().filter(is).collect::<Vec<_>>()
    };
    let settings = window_of(&repo_cx.cx, |w| w.downcast::<SettingsView>().is_some())[0];
    let mut settings_cx = VisualTestContext::from_window(settings, &repo_cx.cx);
    settle_visual(&mut settings_cx);
    assert!(
        settings_cx
            .debug_bounds("identity-status-Not set up")
            .is_some()
    );

    click(&mut settings_cx, "identity-set-up");
    click(&mut settings_cx, "identity-set-up");
    let dialogs = window_of(&repo_cx.cx, |w| w.downcast::<IdentityDialog>().is_some());
    assert_eq!(dialogs.len(), 1, "a second Set Up focuses the open dialog");
    let mut dialog_cx = VisualTestContext::from_window(dialogs[0], &repo_cx.cx);
    settle_visual(&mut dialog_cx);
    click(&mut dialog_cx, "identity-email");
    dialog_cx.simulate_input("dev@example.com");
    click(&mut dialog_cx, "identity-save");
    settle_visual(&mut settings_cx);
    settle_visual(repo_cx);

    let saved = std::fs::read_to_string(&jj_config).expect("read saved jj config");
    assert!(saved.contains("name = \"Git Name\""), "{saved}");
    assert!(saved.contains("email = \"dev@example.com\""), "{saved}");
    assert!(
        settings_cx
            .debug_bounds("identity-status-Git Name <dev@example.com>")
            .is_some()
    );
    assert_eq!(
        displayed_changes(&repo, repo_cx),
        0,
        "the open repo still shows changes matching the old email"
    );
}
