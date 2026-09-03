use std::fs;

use crate::harness::{open_repo, settle_visual};
use gpui::{Modifiers, TestAppContext};
use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_git, run_jj_in};

fn conflicted_repo() -> tempfile::TempDir {
    let temp_dir = init_jj_repo();
    let path = temp_dir.path().join("repo");

    fs::write(
        path.join("greeting.rs"),
        "fn greeting(name: &str) -> String {\n    format!(\"Hello, {name}\")\n}\n",
    )
    .expect("write base");
    run_jj_in(&path, &["describe", "-m", "base"]);
    run_jj_in(&path, &["new", "@"]);
    fs::write(
        path.join("greeting.rs"),
        "fn greeting(name: &str) -> String {\n    format!(\"Welcome from left, {name}\")\n}\n",
    )
    .expect("write left");
    run_jj_in(&path, &["describe", "-m", "left"]);
    run_jj_in(&path, &["bookmark", "create", "left", "-r", "@"]);
    run_jj_in(&path, &["new", "@-"]);
    fs::write(
        path.join("greeting.rs"),
        "fn greeting(name: &str) -> String {\n    format!(\"Welcome from right, {name}\")\n}\n",
    )
    .expect("write right");
    run_jj_in(&path, &["describe", "-m", "right"]);
    run_jj_in(&path, &["new", "left", "@"]);

    temp_dir
}

#[gpui::test]
fn conflict_only_file_row_hides_unusable_review_controls(cx: &mut TestAppContext) {
    let fixture = conflicted_repo();
    let path = fixture.path().join("repo");
    let detail = Repo::open(&path)
        .expect("open conflicted repo")
        .show_summary("@")
        .expect("show conflict summary");
    assert!(detail.info.is_working_copy);
    assert!(detail.diff[0].is_conflict_only_placeholder());

    let (view, cx) = open_repo(path, cx);
    assert!(cx.debug_bounds("file-row-0").is_some());
    assert!(cx.debug_bounds("review-flat-0").is_none());

    view.update_in(cx, |_, _, cx| {
        jayjay_gpui::app::config::update(cx, |config| config.diff.tree_file_list = true);
        cx.notify();
    });
    settle_visual(cx);
    assert!(cx.debug_bounds("tree-file-0").is_some());
    assert!(cx.debug_bounds("review-tree-0").is_none());
}

#[gpui::test]
fn conflict_banner_edits_and_saves_inside_the_repository_window(cx: &mut TestAppContext) {
    let fixture = conflicted_repo();
    let path = fixture.path().join("repo");
    let initial = Repo::open(&path).expect("open conflicted repo");
    let detail = initial.show("@").expect("show conflicted change");
    assert!(detail.info.has_conflict, "fixture must be conflicted");
    assert_eq!(
        initial.show_summary("@").expect("show summary").diff.len(),
        1,
        "conflicts with no ordinary diff still appear in the file list"
    );
    let (view, cx) = open_repo(path.clone(), cx);

    let edit = cx
        .debug_bounds("conflict-resolve-jayjay")
        .expect("Edit in JayJay button");
    cx.simulate_click(edit.center(), Modifiers::default());
    settle_visual(cx);

    assert!(view.read_with(cx, |view, _| view.conflict_editor_active()));
    assert!(view.read_with(cx, |view, cx| {
        view.conflict_editor_has_syntax_highlights(cx)
    }));
    assert!(view.read_with(cx, |view, cx| {
        view.conflict_editor_has_diff_highlights(cx)
    }));
    assert!(
        cx.debug_bounds("conflict-editor-use-base").is_none(),
        "base should stay hidden until requested"
    );
    assert!(cx.debug_bounds("conflict-editor-use-left").is_some());
    assert!(cx.debug_bounds("conflict-editor-use-right").is_some());
    let base_toggle = cx
        .debug_bounds("conflict-editor-base-toggle")
        .expect("Show Base button");
    cx.simulate_click(base_toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert!(cx.debug_bounds("conflict-editor-use-base").is_some());
    assert!(cx.debug_bounds("conflict-editor-use-left").is_none());
    assert!(cx.debug_bounds("conflict-editor-use-right").is_none());
    let base_toggle = cx
        .debug_bounds("conflict-editor-base-toggle")
        .expect("Back to Left & Right button");
    cx.simulate_click(base_toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert!(cx.debug_bounds("conflict-editor-use-base").is_none());
    assert!(cx.debug_bounds("conflict-editor-use-left").is_some());
    assert!(cx.debug_bounds("conflict-editor-use-right").is_some());

    view.update_in(cx, |view, _, cx| {
        view.set_conflict_editor_result("combined in JayJay\n".to_owned(), cx);
    });
    settle_visual(cx);
    let save = cx
        .debug_bounds("conflict-editor-save")
        .expect("Save Resolution button");
    cx.simulate_click(save.center(), Modifiers::default());
    settle_visual(cx);

    assert!(!view.read_with(cx, |view, _| view.conflict_editor_active()));
    let repo = Repo::open(&path).expect("reopen repo");
    assert!(!repo.show("@").expect("show change").info.has_conflict);
    assert_eq!(
        fs::read_to_string(path.join("greeting.rs")).expect("read resolved file"),
        "combined in JayJay\n"
    );
}

#[gpui::test]
fn selecting_another_change_cancels_an_in_flight_conflict_load(cx: &mut TestAppContext) {
    let fixture = conflicted_repo();
    let (view, cx) = open_repo(fixture.path().join("repo"), cx);

    let edit = cx
        .debug_bounds("conflict-resolve-jayjay")
        .expect("Edit in JayJay button");
    cx.simulate_click(edit.center(), Modifiers::default());
    view.update_in(cx, |view, _, cx| view.select_change(1, cx));
    settle_visual(cx);

    assert!(!view.read_with(cx, |view, _| view.conflict_editor_active()));
    assert!(
        cx.debug_bounds("conflict-editor-save").is_none(),
        "a superseded conflict load must not open its editor"
    );
}

#[gpui::test]
fn immutable_conflict_offers_no_editor(cx: &mut TestAppContext) {
    let fixture = conflicted_repo();
    let path = fixture.path().join("repo");
    let conflict_rev = Repo::open(&path)
        .expect("open conflicted repo")
        .show("@")
        .expect("show conflicted change")
        .info
        .change_id
        .id;
    run_jj_in(&path, &["new", "-m", "resolved child"]);
    fs::write(path.join("greeting.rs"), "fn greeting() {}\n").expect("resolve child");
    run_jj_in(&path, &["new", "-m", "working child"]);
    run_git(&path, &["tag", "release"]);
    run_jj_in(&path, &["st"]);
    let detail = Repo::open(&path)
        .expect("open conflicted repo")
        .show(&conflict_rev)
        .expect("show conflicted change");
    assert!(
        detail.info.is_immutable,
        "conflict below a tagged resolution must be immutable"
    );
    let (view, cx) = open_repo(path, cx);
    // The default view's context depth no longer reaches this far back; widen it so the conflicted change is loaded.
    view.update(cx, |view, cx| {
        view.view_model()
            .update(cx, |vm, cx| vm.apply_revset("all()", cx));
    });
    settle_visual(cx);
    let conflict_ix = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .position(|change| change.change_id.id == conflict_rev)
            .expect("conflicted change in graph")
    });
    view.update_in(cx, |view, _, cx| view.select_change(conflict_ix, cx));
    settle_visual(cx);

    assert!(cx.debug_bounds("conflict-resolve-jayjay").is_none());
}
