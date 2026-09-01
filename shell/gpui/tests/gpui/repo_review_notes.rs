use std::fs;

use crate::harness::{install_test_globals, load_selected_change_files, settle_visual};
use gpui::{
    Entity, EntityInputHandler, KeyBinding, Modifiers, TestAppContext, VisualContext,
    VisualTestContext,
};
use jayjay_core::{DiffHunk, DiffProjection, DiffProjectionMode, DiffRenderKind};
use jayjay_gpui::app::actions::SaveNoteComposer;
use jayjay_gpui::diff::{DiffRenderRow, DiffViewMode, NoteDotKind};
use jayjay_gpui::repo::{RepoWindow, revset};
use jayjay_gpui::ui::context_menu::{ContextAction, ContextMenuItem};
use jj_test::{LinearFixture, run_jj_in};

#[gpui::test]
fn add_review_note_via_menu_creates_row_dot_and_badge(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);

    // README.md's display lines are 0 removed, 1 added "# Sample project", 2 added "Edited in GPUI test".
    let items = view.update_in(cx, |view, _, cx| view.build_diff_gutter_menu(&hunk, 2, cx));
    let add_action = find_action(&items, is_add_note).expect("Add Review Note on a changed line");

    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(add_action, cx);
    });
    assert!(view.read_with(cx, |view, _| view.has_text_modal()));
    settle_visual(cx);

    let context_input = view
        .read_with(cx, |view, _| view.text_modal_context_input())
        .expect("selectable highlighted code");
    cx.focus(&context_input);
    cx.simulate_keystrokes(&format!("{}-a", jayjay_gpui::platform::MOD_KEY));
    let selected_context = context_input.update_in(cx, |input, window, cx| {
        let selection = input
            .selected_text_range(false, window, cx)
            .expect("text area selection");
        let mut actual_range = None;
        input.text_for_range(selection.range, &mut actual_range, window, cx)
    });
    assert_eq!(
        selected_context,
        Some("# Sample project\n# Sample project\nEdited in GPUI test".to_owned()),
        "review note header should select the full context"
    );
    cx.simulate_keystrokes("backspace");
    assert_eq!(
        context_input.read_with(cx, |input, _| input.text()),
        "# Sample project\n# Sample project\nEdited in GPUI test",
        "selectable review-note context must remain read-only"
    );

    let input = view
        .read_with(cx, |view, _| view.text_modal_input())
        .expect("composer input present once the overlay opens");
    cx.focus(&input);
    cx.simulate_input("check this line");
    view.update_in(cx, |view, _, cx| view.submit_text_modal(cx));
    settle_visual(cx);

    assert!(!view.read_with(cx, |view, _| view.has_text_modal()));
    let rendered = rows(&view, cx);
    assert!(
        rendered.rows.iter().any(|row| matches!(
            row,
            DiffRenderRow::NoteText { text, .. } if text.as_ref() == "check this line"
        )),
        "saved note must appear as a row right after its anchor line"
    );
    assert_eq!(rendered.dots.get(&2), Some(&NoteDotKind::Active));

    let counts = active_note_counts(&view, cx);
    assert_eq!(
        counts.get("README.md"),
        Some(&1),
        "the file badge count must include the new note"
    );
}

#[gpui::test]
fn review_note_composer_defers_fs_refresh_until_saved(cx: &mut TestAppContext) {
    let (fixture, view, cx, hunk) = open_repo_and_select_readme(cx);
    let items = view.update_in(cx, |view, _, cx| view.build_diff_gutter_menu(&hunk, 2, cx));
    let add_action = find_action(&items, is_add_note).expect("Add Review Note present");
    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(add_action, cx)
    });

    fs::write(
        fixture.path.join("while-writing-note.txt"),
        "external edit\n",
    )
    .expect("write external working-copy edit");
    // Even inside the mutation-echo window the event must be remembered, not dropped.
    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, _| {
            vm.last_internal_mutation_at = Some(std::time::Instant::now());
        });
        view.handle_fs_event(cx);
    });
    assert!(
        !view.read_with(cx, |view, cx| view.view_model().read(cx).loading.refreshing),
        "the composer should keep its diff and draft stable"
    );

    let input = view
        .read_with(cx, |view, _| view.text_modal_input())
        .expect("composer input present");
    cx.focus(&input);
    cx.simulate_input("finish this note first");
    view.update_in(cx, |view, _, cx| view.submit_text_modal(cx));
    settle_visual(cx);

    assert!(view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .is_some_and(|files| {
                files
                    .iter()
                    .any(|file| file.path == "while-writing-note.txt")
            })
    }));
}

/// Regression: submitting an empty (or whitespace-only) body must leave the composer open instead of silently discarding the note.
#[gpui::test]
fn save_review_note_with_empty_body_keeps_composer_open(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);

    let items = view.update_in(cx, |view, _, cx| view.build_diff_gutter_menu(&hunk, 2, cx));
    let add_action = find_action(&items, is_add_note).expect("Add Review Note present");
    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(add_action, cx);
    });
    assert!(view.read_with(cx, |view, _| view.has_text_modal()));

    view.update_in(cx, |view, _, cx| view.submit_text_modal(cx));
    settle_visual(cx);

    assert!(
        view.read_with(cx, |view, _| view.has_text_modal()),
        "an empty save must leave the composer open instead of silently discarding it"
    );
    let rendered = rows(&view, cx);
    assert!(
        !rendered
            .rows
            .iter()
            .any(|row| matches!(row, DiffRenderRow::NoteText { .. })),
        "no note should be created from an empty body"
    );
}

/// Exercises the real key binding + `"NoteComposer"` key-context wiring, not just `submit_text_modal` called directly.
#[gpui::test]
fn save_review_note_via_mod_enter_keybinding(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);
    cx.update(|_window, cx| {
        cx.bind_keys([KeyBinding::new(
            "cmd-enter",
            SaveNoteComposer,
            Some("NoteComposer"),
        )]);
    });

    let items = view.update_in(cx, |view, _, cx| view.build_diff_gutter_menu(&hunk, 2, cx));
    let add_action = find_action(&items, is_add_note).expect("Add Review Note present");
    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(add_action, cx);
    });

    let input = view
        .read_with(cx, |view, _| view.text_modal_input())
        .expect("composer input present");
    cx.focus(&input);
    cx.simulate_input("saved via keybinding");
    cx.simulate_keystrokes("cmd-enter");
    settle_visual(cx);

    assert!(
        !view.read_with(cx, |view, _| view.has_text_modal()),
        "mod+Return must save and close the composer"
    );
    let rendered = rows(&view, cx);
    assert!(rendered.rows.iter().any(|row| matches!(
        row,
        DiffRenderRow::NoteText { text, .. } if text.as_ref() == "saved via keybinding"
    )));
}

#[gpui::test]
fn edit_review_note_via_menu_updates_body(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);
    add_note_and_save(&view, cx, &hunk, "original body");

    let items = view.update_in(cx, |view, _, cx| view.build_diff_gutter_menu(&hunk, 2, cx));
    assert!(
        find_action(&items, is_add_note).is_none(),
        "Add must not appear once an active note exists on the line"
    );
    let edit_action = find_action(&items, is_edit_note).expect("Edit Review Note present");
    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(edit_action, cx);
    });
    let input = view
        .read_with(cx, |view, _| view.text_modal_input())
        .expect("edit composer input present");
    assert_eq!(
        input.read_with(cx, |input, _| input.text()),
        "original body"
    );

    cx.focus(&input);
    input.update(cx, |input, cx| input.clear(cx));
    cx.simulate_input("updated body");
    view.update_in(cx, |view, _, cx| view.submit_text_modal(cx));
    settle_visual(cx);

    let rendered = rows(&view, cx);
    assert!(rendered.rows.iter().any(|row| matches!(
        row,
        DiffRenderRow::NoteText { text, .. } if text.as_ref() == "updated body"
    )));
    assert!(!rendered.rows.iter().any(|row| matches!(
        row,
        DiffRenderRow::NoteText { text, .. } if text.as_ref() == "original body"
    )));
}

#[gpui::test]
fn resolve_review_note_via_menu_dims_dot_drops_row_and_badge(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);
    add_note_and_save(&view, cx, &hunk, "resolve me");

    let items = view.update_in(cx, |view, _, cx| view.build_diff_gutter_menu(&hunk, 2, cx));
    let resolve_action = find_action(&items, is_resolve_note).expect("Resolve Review Note present");
    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(resolve_action, cx);
    });
    settle_visual(cx);

    let rendered = rows(&view, cx);
    assert!(
        !rendered
            .rows
            .iter()
            .any(|row| matches!(row, DiffRenderRow::NoteText { .. })),
        "a resolved note must not keep an in-diff row"
    );
    assert_eq!(rendered.dots.get(&2), Some(&NoteDotKind::Resolved));
    assert!(
        !active_note_counts(&view, cx).contains_key("README.md"),
        "a resolved note must not count toward the active badge"
    );
}

#[gpui::test]
fn resolved_note_dot_menu_offers_delete(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);
    add_note_and_save(&view, cx, &hunk, "resolved leftover");
    let items = view.update_in(cx, |view, _, cx| view.build_diff_gutter_menu(&hunk, 2, cx));
    let resolve_action = find_action(&items, is_resolve_note).expect("Resolve Review Note present");
    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(resolve_action, cx);
    });
    settle_visual(cx);

    let items = view.update_in(cx, |view, _, cx| view.build_diff_gutter_menu(&hunk, 2, cx));
    assert!(
        find_action(&items, is_add_note).is_some(),
        "a resolved-only line still allows adding a fresh note"
    );
    let delete_action = find_action(&items, is_delete_note)
        .expect("dimmed dot must offer Delete for the resolved note");
    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(delete_action, cx);
    });
    settle_visual(cx);

    assert!(
        rows(&view, cx).dots.is_empty(),
        "deleting the resolved note must clear its dimmed dot"
    );
}

#[gpui::test]
fn delete_review_note_via_menu_clears_row_dot_and_note(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);
    add_note_and_save(&view, cx, &hunk, "delete me");

    let items = view.update_in(cx, |view, _, cx| view.build_diff_gutter_menu(&hunk, 2, cx));
    let delete_action = find_action(&items, is_delete_note).expect("Delete Review Note present");
    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(delete_action, cx);
    });
    settle_visual(cx);

    let rendered = rows(&view, cx);
    assert!(rendered.dots.is_empty());
    assert!(
        !rendered
            .rows
            .iter()
            .any(|row| matches!(row, DiffRenderRow::NoteText { .. }))
    );
    let notes = view.read_with(cx, |view, cx| {
        view.view_model().read(cx).review_notes.clone()
    });
    assert!(
        notes.is_empty(),
        "a deleted note must not linger in any status"
    );

    let items = view.update_in(cx, |view, _, cx| view.build_diff_gutter_menu(&hunk, 2, cx));
    assert!(find_action(&items, is_add_note).is_some());
}

#[gpui::test]
fn add_review_note_absent_on_context_line(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);

    // README.md has no context line in this fixture, so use an out-of-range index instead — `line_anchor` must reject both the same way.
    let items = view.update_in(cx, |view, _, cx| view.build_diff_gutter_menu(&hunk, 99, cx));
    assert!(no_note_items(&items));
}

#[gpui::test]
fn add_review_note_absent_on_non_working_copy_change(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);

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
        view.build_diff_gutter_menu(&hunk, 2, cx)
    });
    assert!(no_note_items(&items));
}

#[gpui::test]
fn add_review_note_absent_in_compare_mode(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);

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
        view.build_diff_gutter_menu(&hunk, 2, cx)
    });
    assert!(no_note_items(&items));
}

#[gpui::test]
fn add_review_note_absent_on_projected_hunk(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);
    let projected = DiffHunk {
        projection: Some(DiffProjection {
            plugin_id: "notebook".to_owned(),
            plugin_label: "Notebook".to_owned(),
            plugin_version: 1,
            mode: DiffProjectionMode::Raw,
            render_kind: DiffRenderKind::Markdown,
            virtual_path: "README.md.ipynb.md".to_owned(),
            diagnostics: Vec::new(),
        }),
        ..hunk
    };

    let items = view.update_in(cx, |view, _, cx| {
        view.build_diff_gutter_menu(&projected, 2, cx)
    });
    assert!(no_note_items(&items));
}

#[gpui::test]
fn noted_files_filter_shows_only_files_with_active_notes(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);
    add_note_and_save(&view, cx, &hunk, "only readme has notes");

    assert!(!view.read_with(cx, |view, _| view.notes_only_files()));
    let all_paths = view.read_with(cx, |view, cx| view.visible_file_paths(cx));
    assert!(all_paths.contains(&"feature.txt".to_owned()));

    view.update_in(cx, |view, _, cx| view.toggle_notes_only_files(cx));
    assert!(view.read_with(cx, |view, _| view.notes_only_files()));
    let filtered_paths = view.read_with(cx, |view, cx| view.visible_file_paths(cx));
    assert_eq!(filtered_paths, vec!["README.md".to_owned()]);
}

#[gpui::test]
fn notes_only_filter_auto_clears_when_the_last_active_note_resolves(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);
    add_note_and_save(&view, cx, &hunk, "temporary");
    view.update_in(cx, |view, _, cx| view.toggle_notes_only_files(cx));
    assert!(view.read_with(cx, |view, _| view.notes_only_files()));

    let items = view.update_in(cx, |view, _, cx| view.build_diff_gutter_menu(&hunk, 2, cx));
    let resolve_action = find_action(&items, is_resolve_note).expect("resolve item present");
    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(resolve_action, cx);
    });
    settle_visual(cx);

    assert!(
        !view.read_with(cx, |view, _| view.notes_only_files()),
        "resolving the last active note must drop the filter, or the list would pin to empty"
    );
}

/// Regression: `sync_review_notes` must clear `vm.review_notes`, not just leave it stale, once the notes gate turns off, or `active_note_counts` would keep serving the working-copy change's notes for a different change.
#[gpui::test]
fn active_note_counts_clear_after_switching_away_from_the_noted_change(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);
    add_note_and_save(&view, cx, &hunk, "only relevant on the working copy");
    assert_eq!(active_note_counts(&view, cx).get("README.md"), Some(&1));

    view.update_in(cx, |view, _, cx| {
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
    });
    settle_visual(cx);

    assert!(
        active_note_counts(&view, cx).is_empty(),
        "switching to a non-working-copy change must drop the previous change's note counts"
    );
}

/// Same gate, entered via compare mode instead of a plain change switch.
#[gpui::test]
fn active_note_counts_clear_when_entering_compare_mode(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) = open_repo_and_select_readme(cx);
    add_note_and_save(&view, cx, &hunk, "only relevant outside compare mode");
    assert_eq!(active_note_counts(&view, cx).get("README.md"), Some(&1));

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
    });
    settle_visual(cx);

    assert!(
        active_note_counts(&view, cx).is_empty(),
        "entering compare mode must drop the note counts computed for the plain diff"
    );
}

#[gpui::test]
fn sbs_note_banner_absent_without_notes_present_and_flips_mode_with_notes(cx: &mut TestAppContext) {
    let (_fixture, view, cx, _hunk) = open_repo_and_select_readme(cx);
    view.update_in(cx, |view, _, cx| view.toggle_view_mode(cx));
    settle_visual(cx);
    assert_eq!(
        view.read_with(cx, |view, cx| view.view_model().read(cx).view_mode),
        DiffViewMode::SideBySide
    );
    assert!(
        cx.debug_bounds("sbs-notes-banner").is_none(),
        "no banner without an active note on this file"
    );

    view.update_in(cx, |view, _, cx| view.toggle_view_mode(cx));
    settle_visual(cx);
    let hunk = hunk_for_path(&view, cx, "README.md");
    add_note_and_save(&view, cx, &hunk, "shown in the sbs banner");
    view.update_in(cx, |view, _, cx| view.toggle_view_mode(cx));
    settle_visual(cx);

    let show_unified = cx
        .debug_bounds("sbs-notes-show-unified")
        .expect("show-in-unified action present once a note exists");
    cx.simulate_click(show_unified.center(), Modifiers::default());
    settle_visual(cx);

    assert_eq!(
        view.read_with(cx, |view, cx| view.view_model().read(cx).view_mode),
        DiffViewMode::Unified,
        "\"Show in Unified\" must flip the view mode back"
    );
}

fn is_add_note(action: &ContextAction) -> bool {
    matches!(action, ContextAction::OpenAddReviewNote(_))
}

fn is_edit_note(action: &ContextAction) -> bool {
    matches!(action, ContextAction::OpenEditReviewNote(_))
}

fn is_resolve_note(action: &ContextAction) -> bool {
    matches!(action, ContextAction::ResolveReviewNote(_))
}

fn is_delete_note(action: &ContextAction) -> bool {
    matches!(action, ContextAction::DeleteReviewNote(_))
}

fn no_note_items(items: &[ContextMenuItem]) -> bool {
    find_action(items, is_add_note).is_none()
        && find_action(items, is_edit_note).is_none()
        && find_action(items, is_resolve_note).is_none()
        && find_action(items, is_delete_note).is_none()
}

fn find_action(
    items: &[ContextMenuItem],
    pred: impl Fn(&ContextAction) -> bool,
) -> Option<ContextAction> {
    items
        .iter()
        .find(|item| pred(&item.action))
        .map(|item| item.action.clone())
}

fn rows(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
) -> std::sync::Arc<jayjay_gpui::diff::DiffRenderRows> {
    view.read_with(cx, |view, cx| view.diff_render_rows(cx))
        .expect("rows for the loaded diff")
}

fn active_note_counts(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
) -> std::collections::HashMap<String, usize> {
    view.read_with(cx, |view, cx| {
        (*view.view_model().read(cx).active_note_counts()).clone()
    })
}

/// Adds a note through the real menu + composer flow, not a direct store write, so setup still exercises the composer wiring.
fn add_note_and_save(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    hunk: &DiffHunk,
    body: &str,
) {
    let items = view.update_in(cx, |view, _, cx| view.build_diff_gutter_menu(hunk, 2, cx));
    let add_action = find_action(&items, is_add_note).expect("Add Review Note present");
    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(add_action, cx);
    });
    let input = view
        .read_with(cx, |view, _| view.text_modal_input())
        .expect("composer input present");
    cx.focus(&input);
    cx.simulate_input(body);
    view.update_in(cx, |view, _, cx| view.submit_text_modal(cx));
    settle_visual(cx);
}

fn file_index(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, path: &str) -> usize {
    view.update_in(cx, |view, _, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .position(|h| h.path == path)
            .unwrap_or_else(|| panic!("{path} hunk present"))
    })
}

fn hunk_for_path(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, path: &str) -> DiffHunk {
    let ix = file_index(view, cx, path);
    view.update_in(cx, |view, _, cx| {
        view.view_model().read(cx).files.as_ref().unwrap()[ix].clone()
    })
}

fn open_repo_and_select_readme(
    cx: &mut TestAppContext,
) -> (
    LinearFixture,
    Entity<RepoWindow>,
    &mut VisualTestContext,
    DiffHunk,
) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let ix = file_index(&view, cx, "README.md");
    view.update_in(cx, |view, _, cx| view.select_file(ix, cx));
    settle_visual(cx);
    let hunk = hunk_for_path(&view, cx, "README.md");
    (fixture, view, cx, hunk)
}

/// Regression: a diff refresh must re-reconcile notes even when the store is untouched — a note under a just-edited line flips Stale with no store write.
#[gpui::test]
fn diff_refresh_reconciles_notes_without_store_write(cx: &mut TestAppContext) {
    let (fixture, view, cx, hunk) = open_repo_and_select_readme(cx);
    add_note_and_save(&view, cx, &hunk, "goes stale");
    assert_eq!(rows(&view, cx).dots.get(&2), Some(&NoteDotKind::Active));

    std::fs::write(
        fixture.path.join("README.md"),
        "# Sample project\nCompletely different content now\n",
    )
    .expect("edit noted file");
    // The suppressed fs-watcher would normally snapshot + refresh; do both by hand.
    run_jj_in(&fixture.path, &["st"]);
    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, cx| vm.refresh(true, cx))
    });
    settle_visual(cx);
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    view.update_in(cx, |view, _, cx| view.sync_review_notes(cx));
    settle_visual(cx);

    let stale = view.read_with(cx, |view, cx| {
        view.view_model().read(cx).stale_or_orphaned_notes().len()
    });
    assert_eq!(
        stale, 1,
        "note under the edited line must reconcile stale/orphaned from the diff refresh alone"
    );
    assert!(
        rows(&view, cx).dots.is_empty(),
        "a no-longer-current note keeps no gutter dot"
    );
}
