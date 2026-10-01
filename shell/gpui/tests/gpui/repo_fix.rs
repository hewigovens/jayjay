use crate::harness::*;
use gpui::{Entity, TestAppContext, VisualTestContext};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::ui::context_menu::ContextMenuItem;
use jayjay_gpui::windows::command_palette::CommandPalette;
use jj_test::{LinearFixture, configure_fix_tool};

const TXT_FILES: &str = r#"["glob:'**/*.txt'"]"#;

fn fix_menu_item(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) -> ContextMenuItem {
    view.update(cx, |view, cx| {
        let change = view
            .view_model()
            .read(cx)
            .selected_change()
            .expect("selected change")
            .clone();
        view.build_change_menu(&change, cx)
            .into_iter()
            .find(|item| item.label.as_ref() == "Run formatters (jj fix)")
            .expect("fix menu item")
    })
}

#[cfg(unix)]
fn run_fix_from_menu(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) {
    let item = fix_menu_item(view, cx);
    assert!(item.enabled, "fix is enabled with a usable tool");
    view.update(cx, |view, cx| view.dispatch_context_action(item.action, cx));
    settle_visual(cx);
}

#[cfg(unix)]
#[gpui::test]
fn fix_rewrites_the_selected_change_and_toasts_the_summary(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    configure_fix_tool(&fixture.path, "#!/bin/sh\nsort\n", TXT_FILES);
    std::fs::write(fixture.path.join("unsorted.txt"), "b\na\n").expect("write unsorted file");
    let (view, cx) = open_fixture(&fixture, cx);
    let before = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .selected_change()
            .expect("selected working copy")
            .clone()
    });

    run_fix_from_menu(&view, cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "fix errored: {:?}", vm.error);
        let selected = vm.selected_change().expect("selected change after fix");
        assert_eq!(selected.change_id.id, before.change_id.id);
        assert_ne!(selected.commit_id.id, before.commit_id.id);
        assert_eq!(
            view.toast().as_deref(),
            Some("Formatters rewrote 1 of 1 change")
        );
    });
    assert_eq!(
        std::fs::read_to_string(fixture.path.join("unsorted.txt")).expect("fixed file"),
        "a\nb\n",
    );
}

#[cfg(unix)]
#[gpui::test]
fn tool_failures_stay_on_screen_after_the_refresh(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    configure_fix_tool(
        &fixture.path,
        "#!/bin/sh\necho 'fixer exploded' >&2\nexit 3\n",
        TXT_FILES,
    );
    std::fs::write(fixture.path.join("unsorted.txt"), "b\na\n").expect("write unsorted file");
    let (view, cx) = open_fixture(&fixture, cx);

    run_fix_from_menu(&view, cx);

    view.read_with(cx, |view, cx| {
        let error = view
            .view_model()
            .read(cx)
            .error
            .clone()
            .expect("failure report");
        assert!(
            error.starts_with("Formatters rewrote 0 of 1 change\nfixer failed on ")
                && error.ends_with(": fixer exploded"),
            "{error}"
        );
        assert_eq!(view.toast(), None);
    });
}

#[gpui::test]
fn unusable_tool_config_disables_fix_and_the_palette_explains_why(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    configure_fix_tool(&fixture.path, "", r#"["glob:'**/*"]"#);
    let (view, cx) = open_fixture(&fixture, cx);
    assert!(!fix_menu_item(&view, cx).enabled);

    let repo_cx = &mut cx.cx.clone();
    repo_cx.update(|cx| CommandPalette::open("".into(), Some(view.clone()), cx));
    let window = repo_cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, repo_cx);
    settle_visual(&mut palette_cx);
    palette_cx.simulate_input("run formatters");
    palette_cx.simulate_keystrokes("enter");
    settle_visual(cx);

    let error = view
        .read_with(cx, |view, cx| view.view_model().read(cx).error.clone())
        .expect("the palette reports why fix cannot run");
    assert!(error.starts_with("fix.tools.fixer: "), "{error}");
}
