mod harness;

use gpui::{Entity, Modifiers, TestAppContext, VisualContext, VisualTestContext};
use harness::{install_test_globals, load_selected_change_files, settle_visual};
use jayjay_gpui::repo::window::FileBatchAction;
use jayjay_gpui::repo::{RepoWindow, revset};
use jayjay_gpui::ui::context_menu::{ContextAction, ContextMenuItem};
use jj_test::LinearFixture;

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

fn is_split(action: &ContextAction) -> bool {
    matches!(action, ContextAction::FileBatch(batch) if matches!(batch.as_ref(), FileBatchAction::Split(_)))
}

fn is_commit_files(action: &ContextAction) -> bool {
    matches!(action, ContextAction::FileBatch(batch) if matches!(batch.as_ref(), FileBatchAction::Commit(_)))
}

/// Working copy with exactly two edited files: wip1.txt and wip2.txt.
fn open_repo(
    cx: &mut TestAppContext,
) -> (LinearFixture, Entity<RepoWindow>, &mut VisualTestContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    (fixture, view, cx)
}

fn menu_action(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    path: &str,
    pred: impl Fn(&ContextAction) -> bool,
) -> ContextAction {
    let items: Vec<ContextMenuItem> =
        view.read_with(cx, |view, cx| view.build_file_context_menu(path, cx));
    items
        .iter()
        .find(|item| pred(&item.action))
        .map(|item| item.action.clone())
        .expect("expected file menu action present")
}

fn wc_change_id(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) -> String {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|c| c.is_working_copy)
            .expect("working copy present")
            .change_id
            .id
            .clone()
    })
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

/// Paths in the change with `description`, read straight from core.
fn change_paths(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    description: &str,
) -> Vec<String> {
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let change = vm
            .graph
            .changes
            .iter()
            .find(|c| c.description.trim() == description)
            .unwrap_or_else(|| panic!("change '{description}' present in graph"));
        let repo = vm.repo.clone().expect("repo open");
        let detail = repo
            .show_summary(&revset::change_revision(change))
            .expect("show_summary");
        detail.diff.iter().map(|h| h.path.clone()).collect()
    })
}

#[gpui::test]
fn split_selected_file_moves_it_into_a_new_change(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);
    let old_wc_id = wc_change_id(&view, cx);
    let wip1_identity = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .and_then(|files| files.iter().find(|h| h.path == "wip1.txt"))
            .expect("wip1.txt in working copy diff")
            .review_identity
            .clone()
    });
    view.update_in(cx, |view, _, cx| {
        view.toggle_reviewed(
            old_wc_id.clone(),
            "wip1.txt".to_owned(),
            wip1_identity.clone(),
            cx,
        );
    });

    let action = menu_action(&view, cx, "wip1.txt", is_split);
    view.update_in(cx, |view, _, cx| view.dispatch_context_action(action, cx));
    assert!(view.read_with(cx, |view, _| view.has_text_modal()));

    let input = view
        .read_with(cx, |view, _| view.text_modal_input())
        .expect("split modal input");
    cx.focus(&input);
    cx.simulate_input("split wip1");
    view.update_in(cx, |view, _, cx| view.submit_text_modal(cx));
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "split errored: {:?}", vm.error);
        assert!(vm.selected_change().is_some_and(|c| c.is_working_copy));
    });
    assert_eq!(
        change_paths(&view, cx, "split wip1"),
        ["wip1.txt"],
        "the new change holds exactly the split file"
    );
    assert_eq!(
        loaded_file_paths(&view, cx),
        ["wip2.txt"],
        "the working copy keeps the other file"
    );
    // jj split gives the remainder a fresh change id.
    assert_ne!(wc_change_id(&view, cx), old_wc_id);
    // SwiftUI parity: split paths are unmarked on the pre-split change id.
    assert!(view.read_with(cx, |view, _| {
        !view.is_reviewed(&old_wc_id, "wip1.txt", &wip1_identity)
    }));
    assert!(view.read_with(cx, |view, cx| view.multi_selected_file_paths(cx).is_empty()));
}

#[gpui::test]
fn split_modal_requires_a_description(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);
    let changes_before = view.read_with(cx, |view, cx| {
        view.view_model().read(cx).graph.changes.len()
    });

    let action = menu_action(&view, cx, "wip1.txt", is_split);
    view.update_in(cx, |view, _, cx| view.dispatch_context_action(action, cx));
    view.update_in(cx, |view, _, cx| view.submit_text_modal(cx));
    settle_visual(cx);

    assert!(
        view.read_with(cx, |view, _| view.has_text_modal()),
        "an empty description must keep the modal open instead of splitting"
    );
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.view_model().read(cx).graph.changes.len(),
            changes_before
        );
    });
}

#[gpui::test]
fn commit_selected_file_uses_the_commit_box_message(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);
    let summary = view.read_with(cx, |view, _| view.summary_input());
    summary.update(cx, |input, cx| input.set_text("commit wip1 only", cx));

    let action = menu_action(&view, cx, "wip1.txt", is_commit_files);
    view.update_in(cx, |view, _, cx| view.dispatch_context_action(action, cx));
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "commit errored: {:?}", vm.error);
        assert!(vm.selected_change().is_some_and(|c| c.is_working_copy));
    });
    assert_eq!(change_paths(&view, cx, "commit wip1 only"), ["wip1.txt"]);
    assert_eq!(loaded_file_paths(&view, cx), ["wip2.txt"]);
    assert_eq!(
        summary.read_with(cx, |input, _| input.text()),
        "",
        "the commit box clears once its message is committed"
    );
}

/// Parents (commit ids) of the change with `description`, straight from the loaded graph.
fn change_parents(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    description: &str,
) -> Vec<String> {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|c| c.description.trim() == description)
            .unwrap_or_else(|| panic!("change '{description}' present in graph"))
            .parents
            .clone()
    })
}

/// Split modal opened via the context menu carries a "Parallel split" checkbox (default off) and the sorted selected paths, wired to the same core split mutation as the plain split.
#[gpui::test]
fn split_modal_carries_the_parallel_checkbox_and_sorted_file_list(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);

    let action = menu_action(&view, cx, "wip1.txt", is_split);
    view.update_in(cx, |view, _, cx| view.dispatch_context_action(action, cx));

    assert_eq!(
        view.read_with(cx, |view, _| view.text_modal_checkbox_checked()),
        Some(false),
        "Parallel split defaults to off"
    );
    let paths = view
        .read_with(cx, |view, _| view.text_modal_file_list())
        .expect("split modal carries the file list");
    assert_eq!(
        paths.iter().map(|p| p.to_string()).collect::<Vec<_>>(),
        vec!["wip1.txt".to_owned()]
    );

    view.update_in(cx, |view, _, cx| view.toggle_text_modal_checkbox(cx));
    assert_eq!(
        view.read_with(cx, |view, _| view.text_modal_checkbox_checked()),
        Some(true)
    );
}

/// Core behavioral difference the checkbox controls: `--parallel` makes the split-off change and the remainder siblings sharing the original parents, instead of parent/child.
#[gpui::test]
fn parallel_split_creates_a_sibling_instead_of_a_child(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);
    let original_parents = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .selected_change()
            .expect("working copy selected")
            .parents
            .clone()
    });

    let action = menu_action(&view, cx, "wip1.txt", is_split);
    view.update_in(cx, |view, _, cx| view.dispatch_context_action(action, cx));
    view.update_in(cx, |view, _, cx| view.toggle_text_modal_checkbox(cx));

    let input = view
        .read_with(cx, |view, _| view.text_modal_input())
        .expect("split modal input");
    cx.focus(&input);
    cx.simulate_input("parallel split wip1");
    view.update_in(cx, |view, _, cx| view.submit_text_modal(cx));
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert!(
            view.view_model().read(cx).error.is_none(),
            "parallel split errored: {:?}",
            view.view_model().read(cx).error
        );
    });
    let split_parents = change_parents(&view, cx, "parallel split wip1");
    let remainder_parents = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .selected_change()
            .expect("remainder is the new working copy")
            .parents
            .clone()
    });
    assert_eq!(
        split_parents, original_parents,
        "the split-off change keeps the original working copy's parents"
    );
    assert_eq!(
        remainder_parents, original_parents,
        "a parallel split makes the remainder a sibling of the split-off change, not its child"
    );
}

/// The header's quick-split button (SwiftUI parity) targets files marked reviewed, not the row multi-selection; it stays hidden until at least one file is reviewed.
#[gpui::test]
fn header_split_button_targets_reviewed_files_not_the_row_selection(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);

    // Multi-selecting both files without reviewing them must not surface the header's quick-split control.
    click(&view, cx, "wip1.txt", Modifiers::default());
    click(&view, cx, "wip2.txt", Modifiers::secondary_key());
    assert!(
        cx.debug_bounds("file-split-reviewed").is_none(),
        "quick-split targets reviewed files, not the multi-selection"
    );

    let (change_id, identity) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let change_id = vm
            .selected_change()
            .expect("working copy selected")
            .change_id
            .id
            .clone();
        let identity = vm
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .find(|h| h.path == "wip1.txt")
            .expect("wip1.txt loaded")
            .review_identity
            .clone();
        (change_id, identity)
    });
    view.update_in(cx, |view, _, cx| {
        view.toggle_reviewed(change_id, "wip1.txt".to_owned(), identity, cx);
    });
    settle_visual(cx);

    let bounds = cx
        .debug_bounds("file-split-reviewed")
        .expect("quick-split button appears once a file is reviewed");
    cx.simulate_click(bounds.center(), Modifiers::default());
    settle_visual(cx);

    // The row multi-selection still holds both files, but the modal must carry only the reviewed one.
    let paths = view
        .read_with(cx, |view, _| view.text_modal_file_list())
        .expect("quick-split opens the split modal");
    assert_eq!(
        paths.iter().map(|p| p.to_string()).collect::<Vec<_>>(),
        vec!["wip1.txt".to_owned()]
    );
}

#[gpui::test]
fn commit_selected_file_with_empty_commit_box_is_rejected(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_repo(cx);
    let changes_before = view.read_with(cx, |view, cx| {
        view.view_model().read(cx).graph.changes.len()
    });

    let action = menu_action(&view, cx, "wip1.txt", is_commit_files);
    view.update_in(cx, |view, _, cx| view.dispatch_context_action(action, cx));
    settle_visual(cx);

    assert!(
        view.read_with(cx, |view, _| view.toast().is_some()),
        "an empty commit box must surface the same 'Summary required' feedback as a full commit"
    );
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.view_model().read(cx).graph.changes.len(),
            changes_before
        );
    });
}
