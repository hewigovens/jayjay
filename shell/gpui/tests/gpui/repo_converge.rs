use crate::harness::*;
use gpui::{Modifiers, TestAppContext, VisualTestContext};
use jayjay_core::Repo;
use jj_test::{LinearFixture, current_op_id, run_jj_in};

const MENU_LABEL: &str = "Resolve Divergent Change…";

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let bounds = cx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("missing {selector}"));
    cx.simulate_click(bounds.center(), Modifiers::default());
    settle_visual(cx);
}

fn letter_of(fixture: &LinearFixture, change_id: &str, subject: &str) -> char {
    let plan = Repo::open(&fixture.path)
        .unwrap()
        .converge_preview(change_id)
        .unwrap();
    let index = plan
        .versions
        .iter()
        .position(|version| version.description.trim() == subject)
        .unwrap();
    char::from(b'A' + index as u8)
}

#[gpui::test]
fn resolving_a_divergent_change_reloads_a_stale_plan_then_converges(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let base_op = current_op_id(&fixture.path);
    run_jj_in(&fixture.path, &["describe", "-m", "left"]);
    run_jj_in(
        &fixture.path,
        &["--at-op", &base_op, "describe", "-m", "right"],
    );
    run_jj_in(&fixture.path, &["status"]);
    let (view, cx) = open_focused(&fixture, cx);

    let left = change_with_subject(&view, cx, "left");
    assert!(left.is_divergent);
    let change_id = left.change_id.id.clone();
    let action = view.read_with(cx, |view, cx| {
        view.build_change_menu(&left, cx)
            .into_iter()
            .find(|item| item.label == MENU_LABEL)
            .expect("divergent rows offer the resolve entry")
            .action
    });
    view.update_in(cx, |view, _, cx| view.dispatch_context_action(action, cx));
    settle_visual(cx);
    assert!(cx.debug_bounds("converge-unresolved").is_some());
    assert!(cx.debug_bounds("converge-same-author").is_some());

    keys(cx, "alt-right");
    assert!(cx.debug_bounds("converge-ready").is_some());

    run_jj_in(
        &fixture.path,
        &["describe", "-r", &left.commit_id.id, "-m", "left again"],
    );
    click(cx, "converge-apply");
    assert!(cx.debug_bounds("converge-stale").is_some());
    assert!(cx.debug_bounds("converge-unresolved").is_some());

    let right = letter_of(&fixture, &change_id, "right");
    click(cx, selector(format!("converge-use-description-{right}")));
    click(cx, "converge-edit-description");
    cx.simulate_input("edited");
    settle_visual(cx);
    click(cx, "converge-card-contents");
    cx.simulate_input("ignored");
    settle_visual(cx);
    click(cx, "converge-apply");

    assert!(cx.debug_bounds("converge-sheet").is_none());
    let repo = Repo::open(&fixture.path).unwrap();
    let versions = repo.log(&format!("change_id({change_id})")).unwrap();
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].description.trim(), "right\nedited");
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "{:?}", vm.error);
        assert_eq!(
            vm.selected_change().unwrap().commit_id.id,
            versions[0].commit_id.id
        );
    });
}

#[gpui::test]
fn converging_conflicting_contents_opens_the_merge_editor(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let path = &fixture.path;
    std::fs::write(path.join("settings.txt"), "base\n").unwrap();
    run_jj_in(path, &["describe", "-m", "Settings base"]);
    run_jj_in(path, &["new", "-m", "Tune settings"]);
    run_jj_in(path, &["new", "@-", "-m", "Remote settings"]);
    std::fs::write(path.join("settings.txt"), "remote\n").unwrap();
    run_jj_in(path, &["edit", "subject(\"Tune settings\")"]);
    let base_op = current_op_id(path);
    std::fs::write(path.join("settings.txt"), "local\n").unwrap();
    run_jj_in(path, &["status"]);
    run_jj_in(
        path,
        &[
            "--at-op",
            &base_op,
            "restore",
            "--from",
            "subject(\"Remote settings\")",
            "--into",
            "subject(\"Tune settings\")",
        ],
    );
    run_jj_in(path, &["status"]);
    let (view, cx) = open_focused(&fixture, cx);
    view.update_in(cx, |view, _, cx| {
        view.view_model()
            .update(cx, |vm, cx| vm.apply_revset("divergent()", cx))
    });
    settle_visual(cx);
    let tune = change_with_subject(&view, cx, "Tune settings");
    assert!(tune.is_divergent);
    let ix = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        vm.graph
            .changes
            .iter()
            .position(|change| change.commit_id.id == tune.commit_id.id)
            .unwrap()
    });
    view.update_in(cx, |view, _, cx| view.select_change(ix, cx));
    settle_visual(cx);

    click(cx, "divergent-resolve");
    click(cx, "converge-apply-and-resolve");

    assert!(cx.debug_bounds("converge-sheet").is_none());
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "{:?}", vm.error);
        assert_ne!(vm.revset(), "divergent()");
        assert!(!vm.selected_change().unwrap().is_divergent);
        assert_eq!(vm.selected_hunk().unwrap().path, "settings.txt");
        assert!(view.conflict_editor_active());
    });
}
