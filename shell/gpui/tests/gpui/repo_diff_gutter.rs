use std::fs;

use crate::harness::{install_test_globals, load_selected_change_files, settle_visual};
use gpui::{Entity, Point, TestAppContext, VisualTestContext, point, px};
use jayjay_core::{
    DiffContent, DiffHunk, DiffProjection, DiffProjectionMode, DiffRenderKind, HunkType,
};
use jayjay_gpui::repo::{RepoWindow, revset};
use jayjay_gpui::ui::context_menu::ContextAction;
use jj_test::LinearFixture;

#[gpui::test]
fn gutter_drag_selection_sets_line_range(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_files(cx);

    view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("a.txt".to_owned(), 1, cx);
        view.extend_gutter_selection("a.txt", 3, cx);
    });

    let sel = view
        .read_with(cx, |view, _| view.gutter_selection())
        .expect("gutter selection after drag");
    assert_eq!(sel.path, "a.txt");
    assert_eq!(sel.anchor_line_ix, 1);
    assert_eq!(sel.focus_line_ix, 3);
}

#[gpui::test]
fn gutter_shift_click_extends_from_anchor(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_files(cx);

    view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("a.txt".to_owned(), 1, cx);
        view.shift_extend_gutter_selection("a.txt".to_owned(), 4, cx);
    });
    let sel = view
        .read_with(cx, |view, _| view.gutter_selection())
        .expect("gutter selection after shift-click");
    assert_eq!(
        sel.anchor_line_ix, 1,
        "shift-click keeps the original anchor"
    );
    assert_eq!(sel.focus_line_ix, 4);

    view.update_in(cx, |view, _, cx| {
        view.shift_extend_gutter_selection("a.txt".to_owned(), 0, cx);
    });
    let sel = view
        .read_with(cx, |view, _| view.gutter_selection())
        .expect("gutter selection after second shift-click");
    assert_eq!(sel.anchor_line_ix, 1);
    assert_eq!(sel.focus_line_ix, 0);
}

#[gpui::test]
fn switching_files_clears_gutter_selection(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_files(cx);

    view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("a.txt".to_owned(), 1, cx);
        view.select_file(1, cx);
    });

    view.read_with(cx, |view, _| {
        assert_eq!(view.gutter_selection(), None);
    });
}

#[gpui::test]
fn starting_gutter_and_content_selection_are_mutually_exclusive(cx: &mut TestAppContext) {
    use jayjay_gpui::diff::SbsSide;

    let (_fixture, view, cx) = open_repo_with_files(cx);

    view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("a.txt".to_owned(), 1, cx);
        view.start_diff_selection(1, 0, SbsSide::Unified, cx);
    });
    view.read_with(cx, |view, _| {
        assert!(view.has_diff_selection());
        assert_eq!(
            view.gutter_selection(),
            None,
            "content selection clears the gutter selection"
        );
    });

    view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("a.txt".to_owned(), 2, cx);
    });
    view.read_with(cx, |view, _| {
        assert!(
            !view.has_diff_selection(),
            "gutter selection clears the content selection"
        );
        assert!(view.gutter_selection().is_some());
    });
}

#[gpui::test]
fn right_click_outside_selection_moves_it_to_the_clicked_line(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_files(cx);

    view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("a.txt".to_owned(), 1, cx);
        view.open_gutter_context_menu("a.txt".to_owned(), 5, anchor(), cx);
    });

    let sel = view
        .read_with(cx, |view, _| view.gutter_selection())
        .expect("gutter selection after right-click outside range");
    assert_eq!(sel.anchor_line_ix, 5);
    assert_eq!(sel.focus_line_ix, 5);
}

#[gpui::test]
fn right_click_inside_selection_leaves_it_untouched(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_files(cx);

    view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("a.txt".to_owned(), 1, cx);
        view.shift_extend_gutter_selection("a.txt".to_owned(), 3, cx);
        view.open_gutter_context_menu("a.txt".to_owned(), 2, anchor(), cx);
    });

    let sel = view
        .read_with(cx, |view, _| view.gutter_selection())
        .expect("gutter selection after right-click inside range");
    assert_eq!(sel.anchor_line_ix, 1);
    assert_eq!(sel.focus_line_ix, 3);
}

#[gpui::test]
fn review_notes_context_allows_working_copy_hunk_with_identity(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_files(cx);

    let change_id = view.update_in(cx, |view, _, cx| {
        let expected = view
            .view_model()
            .read(cx)
            .selected_change()
            .expect("selected working copy")
            .change_id
            .id
            .clone();
        let context = view.review_notes_context(&hunk("abc123", None), cx);
        assert_eq!(context, Some(expected.clone()));
        expected
    });
    assert!(!change_id.is_empty());
}

#[gpui::test]
fn review_notes_context_blocks_compare_mode(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_files(cx);

    view.update_in(cx, |view, _, cx| {
        let change = view
            .view_model()
            .read(cx)
            .selected_change()
            .expect("selected working copy")
            .clone();
        view.view_model().update(cx, |vm, _| {
            vm.compare = Some(revset::compare_state(&change))
        });

        assert_eq!(view.review_notes_context(&hunk("abc123", None), cx), None);
    });
}

#[gpui::test]
fn review_notes_context_blocks_projected_hunk(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_files(cx);

    view.update_in(cx, |view, _, cx| {
        let projection = DiffProjection {
            plugin_id: "notebook".to_owned(),
            plugin_label: "Notebook".to_owned(),
            plugin_version: 1,
            mode: DiffProjectionMode::Raw,
            render_kind: DiffRenderKind::Markdown,
            virtual_path: "a.ipynb.md".to_owned(),
            diagnostics: Vec::new(),
        };
        assert_eq!(
            view.review_notes_context(&hunk("abc123", Some(projection)), cx),
            None
        );
    });
}

#[gpui::test]
fn review_notes_context_blocks_empty_identity(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_files(cx);

    view.update_in(cx, |view, _, cx| {
        assert_eq!(view.review_notes_context(&hunk("", None), cx), None);
    });
}

#[gpui::test]
fn abandon_selected_lines_absent_on_non_working_copy_change(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_files(cx);

    let items = view.update_in(cx, |view, _, cx| {
        let parent_ix = view
            .view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .position(|c| !c.is_working_copy)
            .expect("fixture has an ancestor change");
        view.view_model()
            .update(cx, |vm, cx| vm.select_change(parent_ix, cx));
        view.start_gutter_selection("a.txt".to_owned(), 0, cx);
        view.build_diff_gutter_menu(&hunk("abc123", None), 0, cx)
    });
    assert!(no_abandon_item(&items));
}

#[gpui::test]
fn abandon_selected_lines_absent_in_compare_mode(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_files(cx);

    let items = view.update_in(cx, |view, _, cx| {
        let change = view
            .view_model()
            .read(cx)
            .selected_change()
            .expect("selected working copy")
            .clone();
        view.view_model().update(cx, |vm, _| {
            vm.compare = Some(revset::compare_state(&change))
        });
        view.start_gutter_selection("a.txt".to_owned(), 0, cx);
        view.build_diff_gutter_menu(&hunk("abc123", None), 0, cx)
    });
    assert!(no_abandon_item(&items));
}

#[gpui::test]
fn abandon_selected_lines_absent_for_renamed_hunk(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_files(cx);

    let renamed = DiffHunk {
        hunk_type: HunkType::Renamed,
        ..hunk("abc123", None)
    };
    let items = view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("a.txt".to_owned(), 0, cx);
        view.build_diff_gutter_menu(&renamed, 0, cx)
    });
    assert!(no_abandon_item(&items));
}

#[gpui::test]
fn abandon_selected_lines_absent_for_projected_hunk(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo_with_files(cx);

    let projection = DiffProjection {
        plugin_id: "notebook".to_owned(),
        plugin_label: "Notebook".to_owned(),
        plugin_version: 1,
        mode: DiffProjectionMode::Raw,
        render_kind: DiffRenderKind::Markdown,
        virtual_path: "a.ipynb.md".to_owned(),
        diagnostics: Vec::new(),
    };
    let items = view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("a.txt".to_owned(), 0, cx);
        view.build_diff_gutter_menu(&hunk("abc123", Some(projection)), 0, cx)
    });
    assert!(no_abandon_item(&items));
}

#[gpui::test]
fn abandon_selected_lines_absent_when_selection_has_no_changed_lines(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_with_multiline_feature_edit(cx);

    // Display line 0/1/2 are "second"/"third"/"fourth" (added); line 3 is "feature" (context).
    let items = view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("feature.txt".to_owned(), 3, cx);
        view.build_diff_gutter_menu(&hunk, 3, cx)
    });
    assert!(no_abandon_item(&items));
}

#[gpui::test]
fn abandon_change_group_title_when_selection_covers_whole_group(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_with_multiline_feature_edit(cx);

    // Display indices 0..=2 are "second".."fourth" — the whole (single) added run.
    let items = view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("feature.txt".to_owned(), 0, cx);
        view.extend_gutter_selection("feature.txt", 2, cx);
        view.build_diff_gutter_menu(&hunk, 0, cx)
    });
    let item = abandon_item(&items).expect("abandon item present for the whole change group");
    assert_eq!(item.label.as_ref(), "Abandon Change Group");
}

#[gpui::test]
fn abandon_selected_lines_reverts_only_selected_lines(cx: &mut TestAppContext) {
    let (fixture, view, cx, hunk) = open_repo_with_multiline_feature_edit(cx);

    // Display indices 0..=1 are "second"/"third" — a sub-range of the added run, not the whole group.
    let items = view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("feature.txt".to_owned(), 0, cx);
        view.extend_gutter_selection("feature.txt", 1, cx);
        view.build_diff_gutter_menu(&hunk, 0, cx)
    });
    let item = abandon_item(&items).expect("abandon item present for a partial selection");
    assert_eq!(item.label.as_ref(), "Abandon Selected Lines");

    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(item.action.clone(), cx);
    });
    settle_visual(cx);

    assert_eq!(
        fs::read_to_string(fixture.path.join("feature.txt")).expect("read feature.txt"),
        "fourth\nfeature\n",
        "only the selected lines revert; the rest of the addition survives"
    );
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.gutter_selection(),
            None,
            "a successful abandon clears the gutter selection"
        );
        assert_eq!(
            view.toast(),
            None,
            "a successful abandon is silent, not a toast"
        );
    });
}

/// Regression: a stale selection must be rejected, not silently used to reconstruct the file from outdated content.
#[gpui::test]
fn abandon_selected_lines_rejects_stale_selection_and_refreshes(cx: &mut TestAppContext) {
    let (fixture, view, cx, hunk) = open_repo_with_multiline_feature_edit(cx);

    let items = view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("feature.txt".to_owned(), 0, cx);
        view.extend_gutter_selection("feature.txt", 1, cx);
        view.build_diff_gutter_menu(&hunk, 0, cx)
    });
    let item = abandon_item(&items).expect("abandon item present for a partial selection");

    let changed = "second\nthird\nfourth\nfeature\nappended after render\n";
    fs::write(fixture.path.join("feature.txt"), changed).expect("edit on disk after render");

    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(item.action.clone(), cx);
    });
    settle_visual(cx);

    assert_eq!(
        fs::read_to_string(fixture.path.join("feature.txt")).expect("read feature.txt"),
        changed,
        "a stale selection must be rejected, leaving the intervening edit untouched"
    );
    // The guard's refresh also clears `vm.error`, so the durable signal here is the diff catching up to disk, not a surviving error banner.
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(
            vm.current_diff_new_content.as_deref(),
            Some(changed),
            "the guard's refresh must show the file's current content, not the stale render"
        );
    });
}

/// Regression: a Removed-file hunk's absent new side must map to `None`, not `Some("")`, or the staleness guard's materialized-vs-selection comparison falsely rejects every deleted-file abandon.
#[gpui::test]
fn abandon_change_group_restores_a_deleted_file(cx: &mut TestAppContext) {
    let (fixture, view, cx, hunk) = open_repo_with_deleted_file(cx);
    assert_eq!(
        hunk.hunk_type,
        HunkType::Removed,
        "sanity: hello.txt has no new-side content"
    );

    let items = view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("hello.txt".to_owned(), 0, cx);
        view.build_diff_gutter_menu(&hunk, 0, cx)
    });
    let item =
        abandon_item(&items).expect("abandon item present for a deleted file's removed lines");

    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(item.action.clone(), cx);
    });
    settle_visual(cx);

    assert_eq!(
        fs::read_to_string(fixture.path.join("hello.txt")).expect("hello.txt restored"),
        "hello\n",
        "abandoning a deleted file's removed lines must restore it from its parent, not falsely reject as stale"
    );
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(
            vm.error.is_none(),
            "restoring a deleted file must not surface a stale-selection error: {:?}",
            vm.error
        );
    });
}

/// Companion to the deleted-file regression above: an Added-file hunk's absent old side must stay mappable without regressing the (already-correct) new-content guard check.
#[gpui::test]
fn abandon_change_group_removes_a_newly_added_file(cx: &mut TestAppContext) {
    let (fixture, view, cx) = open_repo_with_files(cx);

    let wip1_ix = view.update_in(cx, |view, _, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .position(|h| h.path == "wip1.txt")
            .expect("wip1.txt is a newly added file in this fixture")
    });
    view.update_in(cx, |view, _, cx| view.select_file(wip1_ix, cx));
    settle_visual(cx);
    let hunk = view.update_in(cx, |view, _, cx| {
        view.view_model().read(cx).files.as_ref().unwrap()[wip1_ix].clone()
    });
    assert_eq!(
        hunk.hunk_type,
        HunkType::Added,
        "sanity: wip1.txt has no parent content"
    );

    let items = view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("wip1.txt".to_owned(), 0, cx);
        view.build_diff_gutter_menu(&hunk, 0, cx)
    });
    let item = abandon_item(&items).expect("abandon item present for an added file's lines");

    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(item.action.clone(), cx);
    });
    settle_visual(cx);

    assert!(
        !fixture.path.join("wip1.txt").exists(),
        "abandoning all lines of an added file removes it entirely"
    );
}

#[gpui::test]
fn abandon_selected_lines_reselects_the_same_file_after_refresh(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_with_multiline_feature_edit_and_leading_file(cx);

    // Display indices 0..=1 are "second"/"third" — a sub-range of the added run, not the whole group.
    let items = view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("feature.txt".to_owned(), 0, cx);
        view.extend_gutter_selection("feature.txt", 1, cx);
        view.build_diff_gutter_menu(&hunk, 0, cx)
    });
    let item = abandon_item(&items).expect("abandon item present for a partial selection");

    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(item.action.clone(), cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let files = vm.files.as_ref().expect("files reloaded after the refresh");
        let ix = vm
            .selected_file_ix
            .expect("a file remains selected after the refresh");
        assert_ne!(ix, 0, "feature.txt is not the first file in this fixture");
        assert_eq!(
            files[ix].path, "feature.txt",
            "the file the user abandoned lines in must stay selected after the refresh"
        );
    });
}

/// Regression: reselection-by-path must fall back to the default when the abandoned file drops out of the reloaded list, not a stale/out-of-bounds index.
#[gpui::test]
fn abandon_change_group_falls_back_to_default_selection_when_the_file_disappears(
    cx: &mut TestAppContext,
) {
    let (_fixture, view, cx, hunk) = open_repo_with_multiline_feature_edit_and_leading_file(cx);

    // Display indices 0..=2 are "second".."fourth" — abandoning the whole run reverts feature.txt to its parent's content, so its diff disappears.
    let items = view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("feature.txt".to_owned(), 0, cx);
        view.extend_gutter_selection("feature.txt", 2, cx);
        view.build_diff_gutter_menu(&hunk, 0, cx)
    });
    let item = abandon_item(&items).expect("abandon item present for the whole change group");
    assert_eq!(item.label.as_ref(), "Abandon Change Group");

    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(item.action.clone(), cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let files = vm.files.as_ref().expect("files reloaded after the refresh");
        assert!(
            !files.iter().any(|f| f.path == "feature.txt"),
            "feature.txt's diff is now empty, so it drops out of the file list"
        );
        assert_eq!(
            vm.selected_file_ix,
            Some(0),
            "falls back to the default first-file selection once the abandoned file is gone"
        );
    });
}

#[gpui::test]
fn abandon_selected_lines_completion_preserves_a_newer_selection(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_with_multiline_feature_edit(cx);

    let items = view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("feature.txt".to_owned(), 0, cx);
        view.extend_gutter_selection("feature.txt", 1, cx);
        view.build_diff_gutter_menu(&hunk, 0, cx)
    });
    let item = abandon_item(&items).expect("abandon item present for a partial selection");

    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(item.action.clone(), cx);
        view.start_gutter_selection("feature.txt".to_owned(), 2, cx);
    });
    settle_visual(cx);

    let sel = view
        .read_with(cx, |view, _| view.gutter_selection())
        .expect("a selection started during the async gap must survive the earlier completion");
    assert_eq!(sel.anchor_line_ix, 2);
    assert_eq!(sel.focus_line_ix, 2);
}

#[gpui::test]
fn abandon_selected_lines_absent_for_conflicted_hunk(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_with_conflict_marker_edit(cx);

    // Display line 0 is the collapsed conflict-block summary; the raw diff has one more line than the display basis, which is the index mismatch the conflict gate exists to avoid.
    let items = view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("feature.txt".to_owned(), 0, cx);
        view.build_diff_gutter_menu(&hunk, 0, cx)
    });
    assert!(no_abandon_item(&items));
}

fn no_abandon_item(items: &[jayjay_gpui::ui::context_menu::ContextMenuItem]) -> bool {
    abandon_item(items).is_none()
}

fn abandon_item(
    items: &[jayjay_gpui::ui::context_menu::ContextMenuItem],
) -> Option<&jayjay_gpui::ui::context_menu::ContextMenuItem> {
    items
        .iter()
        .find(|item| matches!(item.action, ContextAction::AbandonSelectedLines(_)))
}

fn anchor() -> Point<gpui::Pixels> {
    point(px(0.), px(0.))
}

fn hunk(review_identity: &str, projection: Option<DiffProjection>) -> DiffHunk {
    DiffHunk {
        path: "a.txt".to_owned(),
        old_path: None,
        old: DiffContent::default(),
        new: DiffContent::default(),
        hunk_type: HunkType::Modified,
        supports_conflict_editor: false,
        supports_file_editor: false,
        review_identity: review_identity.to_owned(),
        projection,
    }
}

fn open_repo_with_files(
    cx: &mut TestAppContext,
) -> (
    LinearFixture,
    gpui::Entity<RepoWindow>,
    &mut VisualTestContext,
) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    (fixture, view, cx)
}

/// Feature.txt's multi-line addition: display lines 0..=2 are added, line 3 is context; returns the loaded `DiffHunk` directly so tests don't depend on `vm.files` ordering.
fn open_repo_with_multiline_feature_edit(
    cx: &mut TestAppContext,
) -> (
    LinearFixture,
    Entity<RepoWindow>,
    &mut VisualTestContext,
    DiffHunk,
) {
    let fixture = LinearFixture::build();
    fixture.add_multiline_working_copy_edit();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let feature_ix = view.update_in(cx, |view, _, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .position(|h| h.path == "feature.txt")
            .expect("feature.txt hunk present")
    });
    view.update_in(cx, |view, _, cx| view.select_file(feature_ix, cx));
    settle_visual(cx);
    let hunk = view.update_in(cx, |view, _, cx| {
        view.view_model().read(cx).files.as_ref().unwrap()[feature_ix].clone()
    });
    (fixture, view, cx, hunk)
}

/// Like `open_repo_with_multiline_feature_edit`, but also edits `README.md` so `feature.txt` lands at a non-zero index, proving reselection works by path rather than the first-file fallback.
fn open_repo_with_multiline_feature_edit_and_leading_file(
    cx: &mut TestAppContext,
) -> (
    LinearFixture,
    Entity<RepoWindow>,
    &mut VisualTestContext,
    DiffHunk,
) {
    let fixture = LinearFixture::build();
    fs::write(
        fixture.path.join("README.md"),
        "# Sample project\nEdited in GPUI test\n",
    )
    .expect("write README.md");
    fixture.add_multiline_working_copy_edit();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let feature_ix = view.update_in(cx, |view, _, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .position(|h| h.path == "feature.txt")
            .expect("feature.txt hunk present")
    });
    assert_ne!(feature_ix, 0, "README.md must sort before feature.txt");
    view.update_in(cx, |view, _, cx| view.select_file(feature_ix, cx));
    settle_visual(cx);
    let hunk = view.update_in(cx, |view, _, cx| {
        view.view_model().read(cx).files.as_ref().unwrap()[feature_ix].clone()
    });
    (fixture, view, cx, hunk)
}

/// hello.txt exists in the fixture's parent change and is deleted in the working copy, producing a whole-file `HunkType::Removed` hunk.
fn open_repo_with_deleted_file(
    cx: &mut TestAppContext,
) -> (
    LinearFixture,
    Entity<RepoWindow>,
    &mut VisualTestContext,
    DiffHunk,
) {
    let fixture = LinearFixture::build();
    fixture.remove_tracked_working_copy_file("hello.txt");
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let hello_ix = view.update_in(cx, |view, _, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .position(|h| h.path == "hello.txt")
            .expect("hello.txt hunk present")
    });
    view.update_in(cx, |view, _, cx| view.select_file(hello_ix, cx));
    settle_visual(cx);
    let hunk = view.update_in(cx, |view, _, cx| {
        view.view_model().read(cx).files.as_ref().unwrap()[hello_ix].clone()
    });
    (fixture, view, cx, hunk)
}

fn open_repo_with_conflict_marker_edit(
    cx: &mut TestAppContext,
) -> (
    LinearFixture,
    Entity<RepoWindow>,
    &mut VisualTestContext,
    DiffHunk,
) {
    let fixture = LinearFixture::build();
    fixture.add_conflict_marker_working_copy_edit();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let feature_ix = view.update_in(cx, |view, _, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .position(|h| h.path == "feature.txt")
            .expect("feature.txt hunk present")
    });
    view.update_in(cx, |view, _, cx| view.select_file(feature_ix, cx));
    settle_visual(cx);
    let hunk = view.update_in(cx, |view, _, cx| {
        view.view_model().read(cx).files.as_ref().unwrap()[feature_ix].clone()
    });
    (fixture, view, cx, hunk)
}
