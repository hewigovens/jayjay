mod support;

use gpui::{Entity, Focusable, TestAppContext, VisualTestContext};
use jayjay_core::DiffHunk;
use jayjay_gpui::diff::{DiffRenderRow, NoteDotKind};
use jayjay_gpui::repo::{RepoWindow, revset};
use jayjay_review::{NoteAnchor, NoteEntry, NoteSide};
use jj_test::LinearFixture;
use support::{
    add_tracked_working_copy_edits, install_test_globals, load_selected_change_files, settle_visual,
};

#[gpui::test]
fn add_note_creates_a_note_text_row_right_after_its_anchor_line(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) =
        open_repo_and_select(cx, add_tracked_working_copy_edits, "README.md");
    let change_id = selected_change_id(&view, cx);

    // README.md's single line renders as remove+add, not context (a jj-diff quirk for single-line files); display lines are 0 removed, 1 added, 2 added "Edited in GPUI test" — the note anchors to index 2.
    let note = add_note_via_store(
        cx,
        &change_id,
        &hunk,
        NoteSide::New,
        2,
        "Edited in GPUI test",
        "check this",
    );
    view.update_in(cx, |view, _, cx| view.refresh_review_notes(cx));
    settle_visual(cx);

    let rendered = view
        .read_with(cx, |view, cx| view.diff_render_rows(cx))
        .expect("rows for the loaded diff");
    assert_eq!(
        rendered.rows,
        vec![
            DiffRenderRow::Line(0),
            DiffRenderRow::Line(1),
            DiffRenderRow::Line(2),
            DiffRenderRow::NoteText {
                note_id: note.id.clone().into(),
                text: "check this".into(),
                is_first: true,
                is_last: true,
            },
        ]
    );
    assert_eq!(rendered.dots.get(&2), Some(&NoteDotKind::Active));
}

#[gpui::test]
fn resolved_note_keeps_a_dimmed_dot_and_drops_its_row(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) =
        open_repo_and_select(cx, add_tracked_working_copy_edits, "README.md");
    let change_id = selected_change_id(&view, cx);

    let note = add_note_via_store(
        cx,
        &change_id,
        &hunk,
        NoteSide::New,
        2,
        "Edited in GPUI test",
        "check this",
    );
    view.update_in(cx, |view, _, cx| view.refresh_review_notes(cx));
    settle_visual(cx);

    resolve_note_via_store(cx, &note.id);
    view.update_in(cx, |view, _, cx| view.refresh_review_notes(cx));
    settle_visual(cx);

    let rendered = view
        .read_with(cx, |view, cx| view.diff_render_rows(cx))
        .expect("rows for the loaded diff");
    assert_eq!(
        rendered.rows,
        vec![
            DiffRenderRow::Line(0),
            DiffRenderRow::Line(1),
            DiffRenderRow::Line(2)
        ],
        "a resolved note must not keep an in-diff row"
    );
    assert_eq!(
        rendered.dots.get(&2),
        Some(&NoteDotKind::Resolved),
        "a resolved note keeps a dimmed dot"
    );
}

#[gpui::test]
fn notes_on_another_file_never_appear_on_the_selected_files_rows(cx: &mut TestAppContext) {
    let (_fixture, view, cx, readme_hunk) =
        open_repo_and_select(cx, add_tracked_working_copy_edits, "README.md");
    let change_id = selected_change_id(&view, cx);
    let feature_hunk = hunk_for_path(&view, cx, "feature.txt");

    // A note on feature.txt's added line 2, while README.md (also with an added line 2) is selected — a path/identity mismatch must gate this out.
    add_note_via_store(
        cx,
        &change_id,
        &feature_hunk,
        NoteSide::New,
        2,
        "Edited in GPUI test",
        "wrong file",
    );
    view.update_in(cx, |view, _, cx| view.refresh_review_notes(cx));
    settle_visual(cx);

    let rendered = view
        .read_with(cx, |view, cx| view.diff_render_rows(cx))
        .expect("rows for the loaded diff");
    assert_eq!(
        rendered.rows,
        vec![
            DiffRenderRow::Line(0),
            DiffRenderRow::Line(1),
            DiffRenderRow::Line(2)
        ]
    );
    assert!(rendered.dots.is_empty());
    assert_eq!(readme_hunk.path, "README.md");
}

#[gpui::test]
fn notes_rows_absent_in_compare_mode(cx: &mut TestAppContext) {
    let (_fixture, view, cx, hunk) =
        open_repo_and_select(cx, add_tracked_working_copy_edits, "README.md");
    let change_id = selected_change_id(&view, cx);
    add_note_via_store(
        cx,
        &change_id,
        &hunk,
        NoteSide::New,
        2,
        "Edited in GPUI test",
        "check this",
    );
    view.update_in(cx, |view, _, cx| view.refresh_review_notes(cx));
    settle_visual(cx);

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

    let rendered = view
        .read_with(cx, |view, cx| view.diff_render_rows(cx))
        .expect("rows still render in compare mode, just without notes");
    assert_eq!(
        rendered.rows,
        vec![
            DiffRenderRow::Line(0),
            DiffRenderRow::Line(1),
            DiffRenderRow::Line(2)
        ]
    );
    assert!(rendered.dots.is_empty());
}

#[gpui::test]
fn note_resolved_by_an_external_review_store_write_drops_from_rows(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_tracked_working_copy_edits(&fixture);
    install_test_globals(cx);
    let store_dir = tempfile::tempdir().expect("tempdir");
    let store_path = store_dir.path().join("review_store.json");
    cx.update(|cx| {
        jayjay_gpui::repo::window::install_review_store_from_path(cx, store_path.clone());
    });

    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    let readme_ix = file_index(&view, cx, "README.md");
    view.update_in(cx, |view, _, cx| view.select_file(readme_ix, cx));
    settle_visual(cx);

    let change_id = selected_change_id(&view, cx);
    let hunk = hunk_for_path(&view, cx, "README.md");
    let note = add_note_via_store(
        cx,
        &change_id,
        &hunk,
        NoteSide::New,
        2,
        "Edited in GPUI test",
        "check this",
    );
    view.update_in(cx, |view, _, cx| view.refresh_review_notes(cx));
    settle_visual(cx);
    assert!(
        rows_contain_note(&view, cx, &note.id),
        "note visible before the external write"
    );

    // A second, independent `ReviewStore` writing to the same file simulates the CLI's `jayjay review resolve-note` running while this window stays open.
    let mut external = jayjay_review::ReviewStore::load_from(store_path.clone());
    external.resolve_note(&note.id);

    // Exercise the render-time staleness check directly, so the test doesn't depend on GPUI's scheduler deciding to re-render on its own.
    view.update_in(cx, |view, _, cx| view.sync_review_notes(cx));
    settle_visual(cx);

    assert!(
        !rows_contain_note(&view, cx, &note.id),
        "resolved-externally note must drop out of the row list without a restart"
    );
}

/// Compares the applied scroll offset with vs without the note, using a file long enough that centering genuinely requires scrolling in both cases — `scroll_to_item`'s non-strict mode no-ops when already visible, so a shifted target must move the offset by exactly one row (18px).
#[gpui::test]
fn find_jump_scrolls_to_the_row_shifted_by_a_note_above_the_match(cx: &mut TestAppContext) {
    let without_note = jump_to_findme_offset(cx, false);
    let with_note = jump_to_findme_offset(cx, true);
    // GPUI's scroll offset grows more negative the further down the list is scrolled.
    assert_eq!(
        without_note - with_note,
        gpui::px(18.),
        "the note above \"findme\" must shift its scroll target by exactly one row"
    );
}

fn jump_to_findme_offset(cx: &mut TestAppContext, add_note: bool) -> gpui::Pixels {
    let (_fixture, view, cx, hunk) = open_repo_and_select(cx, add_long_findable_file, "long.txt");
    if add_note {
        let change_id = selected_change_id(&view, cx);
        // Anchored far above "findme" so the shift isn't swallowed by edge clamping near either end of the list.
        add_note_via_store(
            cx,
            &change_id,
            &hunk,
            NoteSide::New,
            5,
            "line 5",
            "look here",
        );
        view.update_in(cx, |view, _, cx| view.refresh_review_notes(cx));
        settle_visual(cx);
    }

    view.update_in(cx, |view, window, cx| {
        view.open_find(cx);
        view.focus_handle(cx).focus(window, cx);
    });
    cx.simulate_input("findme");

    view.read_with(cx, |view, _| view.diff_scroll_offset_y())
}

/// A very long file with a uniquely-findable line far enough down that centering it always requires scrolling, unlike a target near the top which `scroll_to_item`'s non-strict no-op would swallow.
fn add_long_findable_file(fixture: &LinearFixture) {
    let mut lines: Vec<String> = (1..=200).map(|n| format!("line {n}")).collect();
    lines[99] = "findme".to_owned();
    std::fs::write(fixture.path.join("long.txt"), lines.join("\n") + "\n").expect("write long.txt");
    jj_test::run_jj_in(&fixture.path, &["st"]);
}

fn rows_contain_note(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, note_id: &str) -> bool {
    let rendered = view
        .read_with(cx, |view, cx| view.diff_render_rows(cx))
        .expect("rows for the loaded diff");
    rendered.rows.iter().any(
        |row| matches!(row, DiffRenderRow::NoteText { note_id: id, .. } if id.as_ref() == note_id),
    )
}

fn selected_change_id(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) -> String {
    view.update_in(cx, |view, _, cx| {
        view.view_model()
            .read(cx)
            .selected_change()
            .expect("selected working copy")
            .change_id
            .id
            .clone()
    })
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

/// Reconciliation matches a note to a change group by `(side, line, anchor_excerpt)`, so `anchor_excerpt` must be the exact anchored line text, not a placeholder, or the note reconciles `Stale` instead of `Current`.
fn add_note_via_store(
    cx: &mut VisualTestContext,
    change_id: &str,
    hunk: &DiffHunk,
    side: NoteSide,
    line: u32,
    anchor_excerpt: &str,
    body: &str,
) -> NoteEntry {
    let anchor = NoteAnchor {
        change_id: change_id.to_owned(),
        path: hunk.path.clone(),
        identity: hunk.review_identity.clone(),
        side,
        line,
        anchor_excerpt: anchor_excerpt.to_owned(),
        anchor_context: Vec::new(),
        ignore_whitespace: false,
    };
    cx.update(|_window, cx| {
        let store = jayjay_gpui::repo::window::shared_review_store(cx);
        store.borrow_mut().add_note(anchor, body)
    })
}

fn resolve_note_via_store(cx: &mut VisualTestContext, id: &str) {
    cx.update(|_window, cx| {
        let store = jayjay_gpui::repo::window::shared_review_store(cx);
        store.borrow_mut().resolve_note(id);
    });
}

fn open_repo_and_select<'a>(
    cx: &'a mut TestAppContext,
    setup: impl FnOnce(&LinearFixture),
    path: &str,
) -> (
    LinearFixture,
    Entity<RepoWindow>,
    &'a mut VisualTestContext,
    DiffHunk,
) {
    let fixture = LinearFixture::build();
    setup(&fixture);
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let ix = file_index(&view, cx, path);
    view.update_in(cx, |view, _, cx| view.select_file(ix, cx));
    settle_visual(cx);
    let hunk = hunk_for_path(&view, cx, path);
    (fixture, view, cx, hunk)
}
