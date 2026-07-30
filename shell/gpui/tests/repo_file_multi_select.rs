mod harness;

use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext};
use harness::{install_test_globals, load_selected_change_files, settle_visual};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::repo::window::FileBatchAction;
use jayjay_gpui::ui::context_menu::{ContextAction, ContextMenuItem};
use jj_test::LinearFixture;

/// Working copy with four files in list order: README.md, feature.txt, wip1.txt, wip2.txt.
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

fn file_index(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, path: &str) -> usize {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .and_then(|files| files.iter().position(|h| h.path == path))
            .unwrap_or_else(|| panic!("file {path} present in the loaded diff"))
    })
}

fn click(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, path: &str, modifiers: Modifiers) {
    let ix = file_index(view, cx, path);
    view.update_in(cx, |view, _, cx| {
        view.handle_file_row_click(ix, modifiers, cx);
    });
    settle_visual(cx);
}

fn selection(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) -> Vec<String> {
    view.read_with(cx, |view, cx| view.multi_selected_file_paths(cx))
}

fn primary_path(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .selected_hunk()
            .map(|h| h.path.clone())
    })
}

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Default::default()
    }
}

#[gpui::test]
fn secondary_click_toggles_files_in_and_out_of_the_selection(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);

    click(&view, cx, "wip1.txt", Modifiers::default());
    assert_eq!(selection(&view, cx), ["wip1.txt"]);

    click(&view, cx, "wip2.txt", Modifiers::secondary_key());
    assert_eq!(selection(&view, cx), ["wip1.txt", "wip2.txt"]);
    assert_eq!(
        primary_path(&view, cx).as_deref(),
        Some("wip2.txt"),
        "toggle-add moves the primary selection to the added file"
    );

    click(&view, cx, "wip1.txt", Modifiers::secondary_key());
    assert_eq!(
        selection(&view, cx),
        ["wip2.txt"],
        "toggling a member removes it without disturbing the rest"
    );
}

#[gpui::test]
fn shift_click_extends_a_range_from_the_anchor(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);

    click(&view, cx, "README.md", Modifiers::default());
    click(&view, cx, "wip1.txt", shift());
    assert_eq!(
        selection(&view, cx),
        ["README.md", "feature.txt", "wip1.txt"]
    );

    // The anchor stays put, so a second shift-click re-ranges from README.md.
    click(&view, cx, "wip2.txt", shift());
    assert_eq!(
        selection(&view, cx),
        ["README.md", "feature.txt", "wip1.txt", "wip2.txt"]
    );
}

#[gpui::test]
fn plain_click_collapses_the_selection_to_one_file(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);

    click(&view, cx, "README.md", Modifiers::default());
    click(&view, cx, "wip2.txt", shift());
    assert_eq!(selection(&view, cx).len(), 4);

    click(&view, cx, "feature.txt", Modifiers::default());
    assert_eq!(selection(&view, cx), ["feature.txt"]);
}

#[gpui::test]
fn switching_changes_clears_the_multi_selection(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);

    click(&view, cx, "wip1.txt", Modifiers::default());
    click(&view, cx, "wip2.txt", Modifiers::secondary_key());
    assert_eq!(selection(&view, cx).len(), 2);

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

    assert!(selection(&view, cx).is_empty());
}

fn menu_labels(items: &[ContextMenuItem]) -> Vec<String> {
    items.iter().map(|item| item.label.to_string()).collect()
}

fn has_split_or_commit(items: &[ContextMenuItem]) -> bool {
    items.iter().any(|item| {
        matches!(
            &item.action,
            ContextAction::FileBatch(batch)
                if matches!(batch.as_ref(), FileBatchAction::Split(_) | FileBatchAction::Commit(_))
        )
    })
}

#[gpui::test]
fn file_menu_offers_split_and_commit_only_on_the_working_copy(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);

    let items = view.read_with(cx, |view, cx| view.build_file_context_menu("wip1.txt", cx));
    let labels = menu_labels(&items);
    assert!(
        labels.iter().any(|l| l == "Split to New Change"),
        "{labels:?}"
    );
    assert!(labels.iter().any(|l| l == "Commit File"), "{labels:?}");

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

    let items = view.read_with(cx, |view, cx| {
        view.build_file_context_menu("feature.txt", cx)
    });
    assert!(
        !has_split_or_commit(&items),
        "non-working-copy changes must not offer split/commit: {:?}",
        menu_labels(&items)
    );
    assert!(
        menu_labels(&items).iter().any(|l| l == "Copy Path"),
        "the inspection menu still applies off the working copy"
    );
}

#[gpui::test]
fn file_menu_omits_split_and_commit_in_compare_mode(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);

    let (wc_ix, other_ix) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let wc = vm.selected.expect("working copy selected");
        let other = vm
            .graph
            .changes
            .iter()
            .position(|c| c.description.trim() == "add feature")
            .expect("fixture contains add feature change");
        (wc, other)
    });
    assert_ne!(wc_ix, other_ix);
    view.update_in(cx, |view, _, cx| {
        view.select_or_compare_change(other_ix, true, cx);
    });
    settle_visual(cx);
    assert!(view.read_with(cx, |view, cx| view.view_model().read(cx).compare.is_some()));

    let items = view.read_with(cx, |view, cx| view.build_file_context_menu("wip1.txt", cx));
    assert!(
        !has_split_or_commit(&items),
        "compare mode shows an interdiff, so split/commit must be gated off: {:?}",
        menu_labels(&items)
    );
}

#[gpui::test]
fn batch_menu_targets_the_whole_selection_and_drops_single_file_items(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);

    click(&view, cx, "wip1.txt", Modifiers::default());
    click(&view, cx, "wip2.txt", Modifiers::secondary_key());

    let items = view.read_with(cx, |view, cx| view.build_file_context_menu("wip1.txt", cx));
    let labels = menu_labels(&items);
    assert!(
        labels.iter().any(|l| l == "Split 2 Files to New Change"),
        "{labels:?}"
    );
    assert!(labels.iter().any(|l| l == "Commit 2 Files"), "{labels:?}");
    assert!(
        !labels.iter().any(|l| l == "Copy Path"),
        "single-file inspection items must not appear on a batch selection: {labels:?}"
    );

    // A right-click outside the selection targets just the clicked file.
    let items = view.read_with(cx, |view, cx| view.build_file_context_menu("README.md", cx));
    let labels = menu_labels(&items);
    assert!(
        labels.iter().any(|l| l == "Split to New Change"),
        "{labels:?}"
    );
}

#[gpui::test]
fn space_toggles_review_marks_for_the_whole_selection(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);
    click(&view, cx, "README.md", Modifiers::default());
    click(&view, cx, "wip1.txt", shift());
    let selected = selection(&view, cx);
    assert!(selected.len() >= 2, "range selection active: {selected:?}");

    let reviewed = |view: &Entity<RepoWindow>, cx: &mut VisualTestContext, path: &str| {
        view.read_with(cx, |view, cx| {
            let vm = view.view_model().read(cx);
            let change_id = vm.selected_change().expect("change").change_id.id.clone();
            let hunk = vm
                .files
                .as_ref()
                .and_then(|files| files.iter().find(|h| h.path == path).cloned())
                .expect("hunk");
            view.is_reviewed(&change_id, &hunk.path, &hunk.review_identity)
        })
    };

    view.update_in(cx, |view, _, cx| {
        view.toggle_reviewed_for_selected_files(cx)
    });
    settle_visual(cx);
    for path in &selected {
        assert!(
            reviewed(&view, cx, path),
            "{path} marked with the selection"
        );
    }

    view.update_in(cx, |view, _, cx| {
        view.toggle_reviewed_for_selected_files(cx)
    });
    settle_visual(cx);
    for path in &selected {
        assert!(
            !reviewed(&view, cx, path),
            "{path} unmarked with the selection"
        );
    }
}
