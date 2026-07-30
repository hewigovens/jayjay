//! Batch file actions from the file-column context menu: review toggle, restore, delete, ignore & untrack (SwiftUI's FileColumn menu is the behavioral reference).

mod harness;

use std::fs;

use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext};
use harness::{install_test_globals, load_selected_change_files, settle_visual};
use jayjay_gpui::repo::RepoWindow;
use jj_test::{LinearFixture, run_git, run_jj_in};

/// Working copy with four files in list order: README.md, feature.txt (tracked edits), wip1.txt, wip2.txt (new files).
fn open_repo(
    cx: &mut TestAppContext,
) -> (LinearFixture, Entity<RepoWindow>, &mut VisualTestContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    (fixture, view, cx)
}

fn click(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, path: &str, modifiers: Modifiers) {
    let ix = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .and_then(|files| files.iter().position(|h| h.path == path))
            .unwrap_or_else(|| panic!("file {path} present in the loaded diff"))
    });
    view.update_in(cx, |view, _, cx| {
        view.handle_file_row_click(ix, modifiers, cx);
    });
    settle_visual(cx);
}

fn menu_labels(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    clicked: &str,
) -> Vec<String> {
    view.read_with(cx, |view, cx| {
        view.build_file_context_menu(clicked, cx)
            .iter()
            .map(|item| item.label.to_string())
            .collect()
    })
}

/// Prefix match so items whose labels embed a repo-generated suffix (e.g. "Restore to Parent 1: <hex>") stay addressable.
fn dispatch_menu_item(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    clicked: &str,
    label: &str,
) {
    let action = view.read_with(cx, |view, cx| {
        view.build_file_context_menu(clicked, cx)
            .into_iter()
            .find(|item| item.label.as_ref().starts_with(label))
            .unwrap_or_else(|| panic!("menu item '{label}' present"))
            .action
    });
    view.update_in(cx, |view, _, cx| view.dispatch_context_action(action, cx));
    settle_visual(cx);
}

fn loaded_file_paths(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) -> Vec<String> {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .map(|files| files.iter().map(|h| h.path.clone()).collect())
            .unwrap_or_default()
    })
}

fn assert_no_vm_error(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) {
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "action errored: {:?}", vm.error);
    });
}

#[gpui::test]
fn batch_menu_offers_the_swiftui_action_set_on_the_working_copy(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);

    click(&view, cx, "wip1.txt", Modifiers::default());
    click(&view, cx, "wip2.txt", Modifiers::secondary_key());

    let labels = menu_labels(&view, cx, "wip1.txt");
    for expected in [
        "Mark 2 Files as Reviewed",
        "Split 2 Files to New Change",
        "Commit 2 Files",
        "Restore 2 Files to Parent",
        "Delete 2 Files from Disk",
        "Ignore & Untrack 2 Files",
    ] {
        assert!(
            labels.iter().any(|l| l == expected),
            "{expected}: {labels:?}"
        );
    }
}

#[gpui::test]
fn non_working_copy_changes_offer_restore_and_ignore_but_not_delete_or_review(
    cx: &mut TestAppContext,
) {
    let (_fixture, view, cx) = open_repo(cx);

    let other_ix = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .position(|c| c.description.trim() == "add feature")
            .expect("fixture contains add feature change")
    });
    view.update_in(cx, |view, _, cx| view.select_change(other_ix, cx));
    settle_visual(cx);

    let labels = menu_labels(&view, cx, "feature.txt");
    assert!(
        labels.iter().any(|l| l == "Restore to Parent"),
        "{labels:?}"
    );
    assert!(labels.iter().any(|l| l == "Ignore & Untrack"), "{labels:?}");
    assert!(
        !labels.iter().any(|l| l.contains("Delete")),
        "delete is working-copy-only: {labels:?}"
    );
    assert!(
        !labels.iter().any(|l| l.starts_with("Mark")),
        "review marks apply to the working-copy session only: {labels:?}"
    );
}

/// Restore rewrites the selected change, so an immutable change must not offer it — the same gate the change context menu applies to Abandon.
#[gpui::test]
fn immutable_changes_offer_no_restore_item(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    // Colocated git HEAD tracks @-, so this tags "add feature"; tags() are inside the built-in immutable_heads(), and `jj st` imports the new ref.
    run_git(&fixture.path, &["tag", "release"]);
    run_jj_in(&fixture.path, &["st"]);
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let feature_ix = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        vm.graph
            .changes
            .iter()
            .position(|c| c.description.trim() == "add feature")
            .expect("fixture contains add feature change")
    });
    view.update_in(cx, |view, _, cx| view.select_change(feature_ix, cx));
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        let change = view
            .view_model()
            .read(cx)
            .selected_change()
            .expect("add feature selected");
        assert!(change.is_immutable, "fixture change must be immutable");
    });

    let labels = menu_labels(&view, cx, "feature.txt");
    assert!(
        !labels.iter().any(|l| l.contains("Restore")),
        "an immutable change must not offer restore: {labels:?}"
    );
    assert!(
        labels.iter().any(|l| l == "Ignore & Untrack"),
        "actions that don't rewrite the change stay available: {labels:?}"
    );
}

#[gpui::test]
fn compare_mode_offers_no_batch_actions(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);

    let other_ix = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .position(|c| c.description.trim() == "add feature")
            .expect("fixture contains add feature change")
    });
    view.update_in(cx, |view, _, cx| {
        view.select_or_compare_change(other_ix, true, cx);
    });
    settle_visual(cx);
    assert!(view.read_with(cx, |view, cx| view.view_model().read(cx).compare.is_some()));

    let labels = menu_labels(&view, cx, "wip1.txt");
    assert!(
        !labels
            .iter()
            .any(|l| l.contains("Restore") || l.contains("Ignore") || l.starts_with("Mark")),
        "an interdiff's files are not the change's files: {labels:?}"
    );
}

#[gpui::test]
fn restore_returns_files_to_parent_content_and_clears_the_selection(cx: &mut TestAppContext) {
    let (fixture, view, cx) = open_repo(cx);

    click(&view, cx, "README.md", Modifiers::default());
    click(&view, cx, "feature.txt", Modifiers::secondary_key());
    dispatch_menu_item(&view, cx, "README.md", "Restore 2 Files to Parent");

    assert_no_vm_error(&view, cx);
    assert_eq!(
        fs::read_to_string(fixture.path.join("README.md")).expect("read README.md"),
        "# Sample project\n",
        "restore discards the working-copy edit back to the parent's content"
    );
    assert_eq!(
        fs::read_to_string(fixture.path.join("feature.txt")).expect("read feature.txt"),
        "feature\n"
    );
    assert_eq!(
        loaded_file_paths(&view, cx),
        ["wip1.txt", "wip2.txt"],
        "restored files leave the working-copy diff"
    );
    assert!(
        view.read_with(cx, |view, cx| view.multi_selected_file_paths(cx).is_empty()),
        "the multi-selection prunes to nothing once its files leave the diff"
    );
}

/// A merge working copy offers one restore item per parent; the chosen parent is the content SOURCE, so the merge's file reverts to that parent's version and the parent itself is never rewritten.
#[gpui::test]
fn merge_restore_uses_the_chosen_parent_as_source_without_rewriting_it(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    // Branch a side change off "add hello" (main-), then merge it with main ("add feature") so @ is a two-parent merge with parent 1 = main; the merge edits feature.txt away from parent 1's "feature\n".
    run_jj_in(&fixture.path, &["new", "main-", "-m", "side change"]);
    fs::write(fixture.path.join("side.txt"), "side\n").expect("write side.txt");
    run_jj_in(&fixture.path, &["new", "main", "@", "-m", "merge wip"]);
    fs::write(
        fixture.path.join("feature.txt"),
        "feature edited in merge\n",
    )
    .expect("edit feature.txt in merge");

    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let parents = view.read_with(cx, |view, cx| {
        let change = view
            .view_model()
            .read(cx)
            .selected_change()
            .expect("working-copy merge selected");
        assert!(change.is_working_copy, "the merge is the working copy");
        change.parents.clone()
    });
    assert_eq!(parents.len(), 2, "the fixture merge has two parents");

    let labels = menu_labels(&view, cx, "feature.txt");
    assert!(
        labels.iter().any(|l| l.starts_with("Restore to Parent 1:")),
        "{labels:?}"
    );
    assert!(
        labels.iter().any(|l| l.starts_with("Restore to Parent 2:")),
        "{labels:?}"
    );
    assert!(
        !labels.iter().any(|l| l == "Restore to Parent"),
        "merges get only per-parent restore items: {labels:?}"
    );

    dispatch_menu_item(&view, cx, "feature.txt", "Restore to Parent 1:");

    assert_no_vm_error(&view, cx);
    assert_eq!(
        fs::read_to_string(fixture.path.join("feature.txt")).expect("read feature.txt"),
        "feature\n",
        "the merge's file reverts to parent 1's content"
    );
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        for (ix, parent) in parents.iter().enumerate() {
            assert!(
                vm.graph.changes.iter().any(|c| c.commit_id.id == *parent),
                "parent {} must keep commit {parent} instead of being rewritten",
                ix + 1
            );
        }
    });
}

#[gpui::test]
fn ignore_and_untrack_appends_gitignore_and_drops_the_file_from_the_diff(cx: &mut TestAppContext) {
    let (fixture, view, cx) = open_repo(cx);

    dispatch_menu_item(&view, cx, "wip1.txt", "Ignore & Untrack");

    assert_no_vm_error(&view, cx);
    let gitignore = fs::read_to_string(fixture.path.join(".gitignore")).expect("gitignore written");
    assert_eq!(gitignore, "wip1.txt\n");
    assert!(
        fixture.path.join("wip1.txt").exists(),
        "ignore keeps the file on disk"
    );
    let paths = loaded_file_paths(&view, cx);
    assert!(
        !paths.iter().any(|p| p == "wip1.txt"),
        "the untracked file leaves the diff: {paths:?}"
    );
}

#[gpui::test]
fn delete_removes_working_copy_files_from_disk(cx: &mut TestAppContext) {
    let (fixture, view, cx) = open_repo(cx);

    dispatch_menu_item(&view, cx, "wip1.txt", "Delete from Disk");

    assert_no_vm_error(&view, cx);
    assert!(!fixture.path.join("wip1.txt").exists());
    let paths = loaded_file_paths(&view, cx);
    assert!(
        !paths.iter().any(|p| p == "wip1.txt"),
        "the deleted file leaves the diff: {paths:?}"
    );
}

#[gpui::test]
fn review_toggle_marks_and_unmarks_the_whole_selection(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);

    click(&view, cx, "wip1.txt", Modifiers::default());
    click(&view, cx, "wip2.txt", Modifiers::secondary_key());
    dispatch_menu_item(&view, cx, "wip1.txt", "Mark 2 Files as Reviewed");

    let (change_id, identities) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let change_id = vm
            .selected_change()
            .expect("working copy selected")
            .change_id
            .id
            .clone();
        let identities: Vec<(String, String)> = vm
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .filter(|h| h.path.starts_with("wip"))
            .map(|h| (h.path.clone(), h.review_identity.clone()))
            .collect();
        (change_id, identities)
    });
    assert_eq!(identities.len(), 2);
    for (path, identity) in &identities {
        assert!(
            view.read_with(cx, |view, _| view.is_reviewed(&change_id, path, identity)),
            "{path} marked reviewed"
        );
    }

    // With every selected file reviewed, the same menu slot flips to unmark.
    dispatch_menu_item(&view, cx, "wip1.txt", "Mark 2 Files as Unreviewed");
    for (path, identity) in &identities {
        assert!(
            view.read_with(cx, |view, _| !view.is_reviewed(&change_id, path, identity)),
            "{path} unmarked"
        );
    }
}
