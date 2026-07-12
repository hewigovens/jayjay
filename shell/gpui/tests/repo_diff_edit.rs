mod support;

use std::fs;

use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext};
use jayjay_core::diff::{DiffSpanStyle, compute_file_diff};
use jayjay_core::{DiffEditDestination, HunkType, Repo};
use jayjay_gpui::diff::DiffViewMode;
use jayjay_gpui::repo::view_model::LoadedDiff;
use jayjay_gpui::repo::window::{DiffEditCheckboxState, RepoWindow};
use jj_test::{LinearFixture, run_jj_in};
use support::{install_test_globals, load_selected_change_files, settle_visual};

#[gpui::test]
fn button_cancel_and_escape_preserve_view_mode(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_changed_repo(cx);
    view.update_in(cx, |view, _, cx| {
        view.view_model()
            .update(cx, |vm, _| vm.view_mode = DiffViewMode::SideBySide)
    });

    let edit = cx.debug_bounds("edit-diff").expect("Edit Diff button");
    cx.simulate_click(edit.center(), Modifiers::default());
    settle_visual(cx);
    assert!(view.read_with(cx, |view, _| view.diff_edit_active()));
    assert_eq!(
        view.read_with(cx, |view, cx| view.view_model().read(cx).view_mode),
        DiffViewMode::SideBySide
    );

    let cancel = cx.debug_bounds("diff-edit-cancel").expect("Cancel button");
    cx.simulate_click(cancel.center(), Modifiers::default());
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
    assert_eq!(
        view.read_with(cx, |view, cx| view.view_model().read(cx).view_mode),
        DiffViewMode::SideBySide
    );

    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    cx.simulate_keystrokes("escape");
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
}

#[gpui::test]
fn gutter_menu_enters_mode(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_changed_repo(cx);
    let action = view.update_in(cx, |view, _, cx| {
        let hunk = view.view_model().read(cx).selected_hunk().cloned().unwrap();
        view.build_diff_gutter_menu(&hunk, 0, cx)
            .into_iter()
            .find(|item| item.label == "Open Diff Edit Mode")
            .expect("menu item")
            .action
    });
    view.update_in(cx, |view, _, cx| view.dispatch_context_action(action, cx));
    assert!(view.read_with(cx, |view, _| view.diff_edit_active()));
}

#[gpui::test]
fn compare_mode_has_no_diff_edit_entry_and_cannot_enter(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_changed_repo(cx);
    let other_ix = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let selected = vm.selected.expect("selected change");
        (0..vm.graph.changes.len())
            .find(|ix| *ix != selected)
            .expect("another change")
    });
    view.update_in(cx, |view, _, cx| {
        view.select_or_compare_change(other_ix, true, cx);
    });
    settle_visual(cx);

    assert!(view.read_with(cx, |view, cx| view.view_model().read(cx).compare.is_some()));
    assert!(cx.debug_bounds("edit-diff").is_none());
    view.update_in(cx, |view, _, cx| {
        let hunk = view
            .view_model()
            .read(cx)
            .selected_hunk()
            .cloned()
            .expect("compare hunk");
        assert!(
            view.build_diff_gutter_menu(&hunk, 0, cx)
                .iter()
                .all(|item| item.label != "Open Diff Edit Mode")
        );
        view.enter_diff_edit(cx);
    });
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
}

#[gpui::test]
fn select_all_waits_for_uncached_files_and_selects_every_file(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fs::write(
        fixture.path.join("README.md"),
        "# Sample project\nchanged\n",
    )
    .unwrap();
    fs::write(fixture.path.join("feature.txt"), "feature\nchanged\n").unwrap();
    run_jj_in(&fixture.path, &["st"]);
    let (view, cx) = open_fixture(&fixture, cx);
    let paths = view.update_in(cx, |view, _, cx| {
        let vm = view.view_model().read(cx);
        let selected_path = vm.selected_hunk().expect("selected file").path.clone();
        let paths = vm
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .map(|hunk| hunk.path.clone())
            .collect::<Vec<_>>();
        view.view_model().update(cx, |vm, _| {
            vm.diff_cache
                .retain(|_, loaded| loaded.diff.path == selected_path);
        });
        view.enter_diff_edit(cx);
        view.toggle_diff_edit_all(cx);
        assert!(view.diff_edit_selecting_all());
        paths
    });

    settle_visual(cx);
    assert!(!view.read_with(cx, |view, _| view.diff_edit_selecting_all()));
    view.update_in(cx, |view, _, cx| {
        for path in paths {
            assert_eq!(
                view.diff_edit_file_state(&path, cx),
                DiffEditCheckboxState::All,
                "{path} must not be silently skipped"
            );
        }
    });
}

#[gpui::test]
fn divergent_change_preloads_uncached_files_for_select_all(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fs::write(
        fixture.path.join("README.md"),
        "# Sample project\nchanged\n",
    )
    .unwrap();
    fs::write(fixture.path.join("feature.txt"), "feature\nchanged\n").unwrap();
    run_jj_in(&fixture.path, &["st"]);
    run_jj_in(&fixture.path, &["describe", "-m", "side one"]);
    run_jj_in(
        &fixture.path,
        &["--at-operation", "@-", "describe", "-m", "side two"],
    );
    run_jj_in(&fixture.path, &["st"]);

    let (view, cx) = open_fixture(&fixture, cx);
    let (paths, uncached_path) = view.update_in(cx, |view, _, cx| {
        let vm = view.view_model().read(cx);
        let selected = vm.selected_change().expect("selected change");
        assert!(selected.is_divergent, "fixture change must be divergent");
        let selected_path = vm.selected_hunk().expect("selected file").path.clone();
        let hunks = vm.files.as_ref().expect("files loaded");
        let paths = hunks
            .iter()
            .map(|hunk| hunk.path.clone())
            .collect::<Vec<_>>();
        let uncached_path = paths
            .iter()
            .find(|path| **path != selected_path)
            .expect("second file")
            .clone();
        view.view_model().update(cx, |vm, _| {
            vm.diff_cache
                .retain(|_, loaded| loaded.diff.path == selected_path);
        });
        view.enter_diff_edit(cx);
        view.toggle_diff_edit_all(cx);
        (paths, uncached_path)
    });

    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        assert!(!view.diff_edit_selecting_all());
        let uncached = view
            .view_model()
            .read(cx)
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .find(|hunk| hunk.path == uncached_path)
            .expect("previously uncached file")
            .clone();
        assert!(view.diff_edit_file_supported(&uncached, cx));
        for path in paths {
            assert_eq!(
                view.diff_edit_file_state(&path, cx),
                DiffEditCheckboxState::All,
                "{path} must be selected"
            );
        }
    });
}

#[gpui::test]
fn uncached_file_is_hidden_until_entry_preload_finishes(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fs::write(
        fixture.path.join("README.md"),
        "# Sample project\nchanged\n",
    )
    .unwrap();
    fs::write(fixture.path.join("feature.txt"), "feature\nchanged\n").unwrap();
    run_jj_in(&fixture.path, &["st"]);
    let (view, cx) = open_fixture(&fixture, cx);
    let uncached_path = view.update_in(cx, |view, _, cx| {
        let vm = view.view_model().read(cx);
        let selected_path = vm.selected_hunk().expect("selected file").path.clone();
        let uncached = vm
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .find(|hunk| hunk.path != selected_path)
            .expect("second file")
            .clone();
        view.view_model().update(cx, |vm, _| {
            vm.diff_cache
                .retain(|_, loaded| loaded.diff.path == selected_path);
        });
        view.enter_diff_edit(cx);
        assert!(!view.diff_edit_has_known_unsupported(cx));
        assert!(!view.diff_edit_file_supported(&uncached, cx));
        uncached.path
    });

    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        let hunk = view
            .view_model()
            .read(cx)
            .files
            .as_ref()
            .unwrap()
            .iter()
            .find(|hunk| hunk.path == uncached_path)
            .unwrap()
            .clone();
        assert!(view.diff_edit_file_supported(&hunk, cx));
        view.toggle_diff_edit_file(&uncached_path, cx);
        assert_eq!(
            view.diff_edit_file_state(&uncached_path, cx),
            DiffEditCheckboxState::All
        );
    });
}

#[gpui::test]
fn renamed_file_is_excluded_while_supported_edits_apply(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fs::create_dir(fixture.path.join("moved")).unwrap();
    fs::rename(
        fixture.path.join("README.md"),
        fixture.path.join("moved/README.md"),
    )
    .unwrap();
    let renamed = "# Sample project\nrenamed edit\n";
    fs::write(fixture.path.join("moved/README.md"), renamed).unwrap();
    fs::write(fixture.path.join("feature.txt"), "feature\nedited\n").unwrap();
    run_jj_in(&fixture.path, &["st"]);

    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| {
        let vm = view.view_model().read(cx);
        let renamed_hunk = vm
            .files
            .as_ref()
            .unwrap()
            .iter()
            .find(|hunk| hunk.path == "moved/README.md")
            .expect("renamed hunk");
        assert_eq!(renamed_hunk.hunk_type, HunkType::Renamed);
        assert!(!view.diff_edit_file_supported(renamed_hunk, cx));
        view.enter_diff_edit(cx);
        view.toggle_diff_edit_file("moved/README.md", cx);
        assert!(view.diff_edit_selected("moved/README.md").is_empty());
        view.toggle_diff_edit_all(cx);
    });
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        assert!(!view.diff_edit_selecting_all());
        assert!(view.diff_edit_selected("moved/README.md").is_empty());
        assert_eq!(
            view.diff_edit_file_state("feature.txt", cx),
            DiffEditCheckboxState::All
        );
        view.start_diff_edit_apply(DiffEditDestination::RemoveFromSource, cx);
    });
    settle_visual(cx);
    settle_visual(cx);

    assert_eq!(
        fs::read_to_string(fixture.path.join("feature.txt")).unwrap(),
        "feature\nedited\n"
    );
    assert!(!fixture.path.join("README.md").exists());
    assert_eq!(
        fs::read_to_string(fixture.path.join("moved/README.md")).unwrap(),
        renamed
    );
}

#[gpui::test]
fn line_group_file_and_select_all_update_full_diff_selection(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_changed_repo(cx);
    let (path, changed) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let diff = vm.current_diff.as_ref().unwrap();
        let changed = diff
            .lines
            .iter()
            .enumerate()
            .filter(|(_, line)| matches!(line.style, DiffSpanStyle::Added | DiffSpanStyle::Removed))
            .map(|(ix, _)| ix)
            .collect::<Vec<_>>();
        (vm.selected_hunk().unwrap().path.clone(), changed)
    });
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        view.toggle_diff_edit_display_line(&path, changed[0] as u32 + 1, cx);
        assert_eq!(view.diff_edit_selected(&path).len(), 1);
        view.toggle_diff_edit_display_line(&path, changed[0] as u32 + 1, cx);
        assert!(view.diff_edit_selected(&path).is_empty());
        view.select_diff_edit_display_group(&path, changed[0] as u32 + 1, cx);
        assert!(!view.diff_edit_selected(&path).is_empty());
        view.select_diff_edit_display_group(&path, changed[0] as u32 + 1, cx);
        assert!(!view.diff_edit_selected(&path).is_empty());
        view.toggle_diff_edit_display_line(&path, changed[0] as u32 + 1, cx);
        assert_eq!(
            view.diff_edit_file_state(&path, cx),
            DiffEditCheckboxState::Some
        );
        view.toggle_diff_edit_file(&path, cx);
        assert_eq!(
            view.diff_edit_file_state(&path, cx),
            DiffEditCheckboxState::All
        );
        view.toggle_diff_edit_file(&path, cx);
        assert_eq!(
            view.diff_edit_file_state(&path, cx),
            DiffEditCheckboxState::None
        );
        view.toggle_diff_edit_all(cx);
        assert_eq!(
            view.diff_edit_file_state(&path, cx),
            DiffEditCheckboxState::All
        );
    });
}

#[gpui::test]
fn collapsed_display_line_maps_to_full_diff_index(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_changed_repo(cx);
    let old = (1..=40)
        .map(|line| format!("line {line}\n"))
        .collect::<String>();
    let new = old.replace("line 20\n", "changed 20\n");
    let display = std::sync::Arc::new(compute_file_diff("README.md", &old, &new, false));
    let changed_display_ix = display
        .lines
        .iter()
        .position(|line| line.style == DiffSpanStyle::Removed)
        .unwrap();
    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, _| {
            vm.current_diff = Some(display.clone());
            vm.current_diff_old_content = Some(old.clone().into());
            vm.current_diff_new_content = Some(new.clone().into());
            vm.diff_cache.clear();
            vm.diff_cache.insert(
                "collapsed".into(),
                LoadedDiff {
                    diff: display,
                    projection: None,
                    svg_preview: None,
                    markdown_preview: None,
                    old_content: Some(old.into()),
                    new_content: Some(new.into()),
                },
            );
        });
        view.enter_diff_edit(cx);
    });
    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        view.toggle_diff_edit_display_line("README.md", changed_display_ix as u32 + 1, cx);
        let selected = view.diff_edit_selected("README.md");
        assert_eq!(selected.len(), 1);
        assert!(selected.first().copied().unwrap() > changed_display_ix as u32 + 1);
    });
}

#[gpui::test]
fn group_selection_unions(cx: &mut TestAppContext) {
    let fixture = separated_edits_fixture(false);
    let (view, cx) = open_fixture(&fixture, cx);
    select_file_by_path(&view, cx, "edit.txt");
    let display_line = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .current_diff
            .as_ref()
            .unwrap()
            .lines
            .iter()
            .position(|line| line.style == DiffSpanStyle::Added && line.text() == "selected two")
            .unwrap() as u32
            + 1
    });
    view.update_in(cx, |view, _, cx| {
        view.enter_diff_edit(cx);
        view.toggle_diff_edit_display_line("edit.txt", display_line, cx);
        view.toggle_diff_edit_display_line("edit.txt", display_line, cx);
        assert!(view.diff_edit_selected("edit.txt").is_empty());
        view.select_diff_edit_display_group("edit.txt", display_line, cx);
    });
    let first = view.read_with(cx, |view, _| view.diff_edit_selected("edit.txt"));
    view.update_in(cx, |view, _, cx| {
        view.select_diff_edit_display_group("edit.txt", display_line, cx)
    });
    assert_eq!(
        view.read_with(cx, |view, _| view.diff_edit_selected("edit.txt")),
        first
    );
}

#[gpui::test]
fn switching_changes_clears_diff_edit_session(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_changed_repo(cx);
    view.update_in(cx, |view, _, cx| {
        view.enter_diff_edit(cx);
        let current = view.view_model().read(cx).selected.unwrap();
        let next = if current == 0 { 1 } else { 0 };
        view.select_change(next, cx);
    });
    settle_visual(cx);
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
}

#[gpui::test]
fn non_working_copy_shows_destinations_and_prefills_description(cx: &mut TestAppContext) {
    let fixture = separated_edits_fixture(true);
    let (view, cx) = open_fixture(&fixture, cx);
    select_change_by_description(&view, cx, "edit source");
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    view.read_with(cx, |view, _| {
        let snapshot = view.diff_edit_snapshot();
        assert!(!snapshot.working_copy);
        assert_eq!(snapshot.description.trim(), "edit source");
        assert_eq!(
            snapshot.destinations,
            vec![
                DiffEditDestination::NewChild,
                DiffEditDestination::NewParallel,
                DiffEditDestination::MoveToWorkingCopy,
                DiffEditDestination::RemoveFromSource,
            ]
        );
    });

    view.update_in(cx, |view, _, cx| view.exit_diff_edit(cx));
    select_change_by_description(&view, cx, "working child");
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    view.read_with(cx, |view, _| {
        let snapshot = view.diff_edit_snapshot();
        assert!(snapshot.working_copy);
        assert_eq!(
            snapshot.destinations,
            vec![DiffEditDestination::RemoveFromSource]
        );
    });
}

#[gpui::test]
fn remove_from_working_copy_exits_and_reselects_file(cx: &mut TestAppContext) {
    let (fixture, view, cx) = open_changed_repo(cx);
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    select_first_changed_line(&view, cx);
    view.update_in(cx, |view, _, cx| {
        view.start_diff_edit_apply(DiffEditDestination::RemoveFromSource, cx)
    });
    settle_visual(cx);
    settle_visual(cx);
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
    assert_eq!(
        view.read_with(cx, |view, cx| {
            view.view_model()
                .read(cx)
                .selected_hunk()
                .map(|hunk| hunk.path.clone())
        }),
        Some("README.md".to_owned())
    );
    let repo = Repo::open(&fixture.path).unwrap();
    assert_ne!(
        repo.file_content("@", "README.md").unwrap(),
        "# Sample project\nfirst edit\nsecond edit\n"
    );
}

#[gpui::test]
fn done_keeps_only_selected_lines(cx: &mut TestAppContext) {
    let fixture = two_file_edits_fixture();
    let (view, cx) = open_fixture(&fixture, cx);
    select_file_by_path(&view, cx, "edit.txt");
    enter_and_select_group(&view, cx, "selected two");
    view.update_in(cx, |view, _, cx| {
        view.start_diff_edit_apply(DiffEditDestination::RemoveFromSource, cx)
    });
    settle_visual(cx);
    settle_visual(cx);

    assert_eq!(
        fs::read_to_string(fixture.path.join("edit.txt")).unwrap(),
        "one\nselected two\nthree\nfour\n"
    );
    assert_eq!(
        fs::read_to_string(fixture.path.join("untouched.txt")).unwrap(),
        "alpha\nbeta\ngamma\n"
    );
}

#[gpui::test]
fn done_with_empty_selection_is_inert(cx: &mut TestAppContext) {
    let fixture = two_file_edits_fixture();
    let before_edit = fs::read_to_string(fixture.path.join("edit.txt")).unwrap();
    let before_untouched = fs::read_to_string(fixture.path.join("untouched.txt")).unwrap();
    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| {
        view.enter_diff_edit(cx);
        view.start_diff_edit_apply(DiffEditDestination::RemoveFromSource, cx);
    });

    assert!(view.read_with(cx, |view, _| view.diff_edit_active()));
    assert_eq!(
        view.read_with(cx, |view, _| view.toast().map(|toast| toast.to_string())),
        Some("Select at least one file, hunk, or line before applying diff edit.".into())
    );
    assert_eq!(
        fs::read_to_string(fixture.path.join("edit.txt")).unwrap(),
        before_edit
    );
    assert_eq!(
        fs::read_to_string(fixture.path.join("untouched.txt")).unwrap(),
        before_untouched
    );
}

#[gpui::test]
fn new_child_contains_exactly_the_selected_lines(cx: &mut TestAppContext) {
    let fixture = separated_edits_fixture(false);
    let (view, cx) = open_fixture(&fixture, cx);
    select_file_by_path(&view, cx, "edit.txt");
    let source_change_id = selected_change_id(&view, cx);
    enter_and_select_group(&view, cx, "selected two");
    apply_with_message(&view, cx, DiffEditDestination::NewChild, "selected child");

    let repo = Repo::open(&fixture.path).expect("open mutated repo");
    let child = change_by_description(&repo, "selected child");
    let source = change_by_id(&repo, &source_change_id);
    assert_eq!(child.parents, vec![source.commit_id.id.clone()]);
    assert_eq!(
        repo.file_content(&child.change_id, "edit.txt")
            .expect("child file")
            .trim_end(),
        "one\nselected two\nthree\nfour\nfive\nsix\nseven\nremaining eight\nnine\nten"
    );
    assert_eq!(
        repo.file_content(&source.change_id, "edit.txt")
            .expect("source file")
            .trim_end(),
        "one\ntwo\nthree\nfour\nfive\nsix\nseven\nremaining eight\nnine\nten"
    );
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
}

#[gpui::test]
fn new_parallel_creates_a_sibling_with_same_parents(cx: &mut TestAppContext) {
    let fixture = separated_edits_fixture(false);
    let (view, cx) = open_fixture(&fixture, cx);
    select_file_by_path(&view, cx, "edit.txt");
    let source_change_id = selected_change_id(&view, cx);
    enter_and_select_group(&view, cx, "selected two");
    apply_with_message(
        &view,
        cx,
        DiffEditDestination::NewParallel,
        "selected parallel",
    );

    let repo = Repo::open(&fixture.path).expect("open mutated repo");
    let parallel = change_by_description(&repo, "selected parallel");
    let source = change_by_id(&repo, &source_change_id);
    assert_eq!(parallel.parents, source.parents);
    assert_ne!(parallel.parents, vec![source.commit_id.id.clone()]);
    assert_eq!(
        repo.file_content(&parallel.change_id, "edit.txt")
            .expect("parallel file")
            .trim_end(),
        "one\nselected two\nthree\nfour\nfive\nsix\nseven\neight\nnine\nten"
    );
    assert_eq!(
        repo.file_content(&source.change_id, "edit.txt")
            .expect("source file")
            .trim_end(),
        "one\ntwo\nthree\nfour\nfive\nsix\nseven\nremaining eight\nnine\nten"
    );
}

#[gpui::test]
fn move_to_working_copy_from_a_parent_change(cx: &mut TestAppContext) {
    let fixture = separated_edits_fixture(true);
    let (view, cx) = open_fixture(&fixture, cx);
    select_change_by_description(&view, cx, "edit source");
    select_file_by_path(&view, cx, "edit.txt");
    let source_change_id = selected_change_id(&view, cx);
    enter_and_select_group(&view, cx, "selected two");
    view.update_in(cx, |view, _, cx| {
        view.start_diff_edit_apply(DiffEditDestination::MoveToWorkingCopy, cx)
    });
    settle_visual(cx);

    let repo = Repo::open(&fixture.path).expect("open mutated repo");
    let source = change_by_id(&repo, &source_change_id);
    assert_eq!(
        repo.file_content(&source.change_id, "edit.txt")
            .expect("source file")
            .trim_end(),
        "one\ntwo\nthree\nfour\nfive\nsix\nseven\nremaining eight\nnine\nten"
    );
    assert_eq!(
        repo.file_content("@", "edit.txt")
            .expect("working-copy file")
            .trim_end(),
        "one\nselected two\nthree\nfour\nfive\nsix\nseven\nremaining eight\nnine\nten"
    );
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
}

#[gpui::test]
fn stale_selection_rejection_surfaces_and_refreshes(cx: &mut TestAppContext) {
    let (fixture, view, cx) = open_changed_repo(cx);
    enter_and_select_line(&view, cx, "first edit");
    let changed = "# Sample project\nfirst edit\nsecond edit\nintervening edit\n";
    fs::write(fixture.path.join("README.md"), changed).expect("edit after mode entry");
    run_jj_in(&fixture.path, &["st"]);

    view.update_in(cx, |view, _, cx| {
        view.start_diff_edit_apply(DiffEditDestination::RemoveFromSource, cx)
    });
    settle_visual(cx);

    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
    assert_eq!(
        fs::read_to_string(fixture.path.join("README.md")).expect("read current file"),
        changed,
        "the rejected operation must not reconstruct the file from stale content"
    );
    assert!(!view.read_with(cx, |view, _| view.diff_edit_active()));
}

fn enter_and_select_line(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, text: &str) {
    let (path, line) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let line = vm
            .current_diff
            .as_ref()
            .expect("diff loaded")
            .lines
            .iter()
            .position(|line| line.style == DiffSpanStyle::Added && line.text() == text)
            .unwrap_or_else(|| panic!("added line '{text}' present"));
        (
            vm.selected_hunk().expect("hunk selected").path.clone(),
            line,
        )
    });
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        view.toggle_diff_edit_display_line(&path, line as u32 + 1, cx);
    });
}

fn enter_and_select_group(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, added_text: &str) {
    let (path, line) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let line = vm
            .current_diff
            .as_ref()
            .expect("diff loaded")
            .lines
            .iter()
            .position(|line| line.style == DiffSpanStyle::Added && line.text() == added_text)
            .unwrap_or_else(|| panic!("added line '{added_text}' present"));
        (
            vm.selected_hunk().expect("hunk selected").path.clone(),
            line,
        )
    });
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        view.select_diff_edit_display_group(&path, line as u32 + 1, cx);
    });
}

fn apply_with_message(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    destination: DiffEditDestination,
    message: &str,
) {
    view.update_in(cx, |view, _, cx| {
        view.set_diff_edit_message(message, cx);
        view.start_diff_edit_apply(destination, cx);
    });
    settle_visual(cx);
}

fn selected_change_id(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) -> String {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .selected_change()
            .expect("change selected")
            .change_id
            .id
            .clone()
    })
}

fn select_change_by_description(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    description: &str,
) {
    let ix = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .position(|change| change.description.trim() == description)
            .expect("fixture change present")
    });
    view.update_in(cx, |view, _, cx| view.select_change(ix, cx));
    settle_visual(cx);
}

fn select_file_by_path(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, path: &str) {
    let ix = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .position(|hunk| hunk.path == path)
            .unwrap_or_else(|| panic!("file '{path}' present"))
    });
    view.update_in(cx, |view, _, cx| view.select_file(ix, cx));
    settle_visual(cx);
}

fn separated_edits_fixture(with_child: bool) -> LinearFixture {
    let fixture = LinearFixture::build();
    let base = "one\ntwo\nthree\nfour\nfive\nsix\nseven\neight\nnine\nten\n";
    fs::write(fixture.path.join("edit.txt"), base).expect("write base file");
    run_jj_in(&fixture.path, &["describe", "-m", "edit base"]);
    run_jj_in(&fixture.path, &["new", "-m", "edit source"]);
    fs::write(
        fixture.path.join("edit.txt"),
        "one\nselected two\nthree\nfour\nfive\nsix\nseven\nremaining eight\nnine\nten\n",
    )
    .expect("write separated edits");
    run_jj_in(&fixture.path, &["st"]);
    if with_child {
        run_jj_in(&fixture.path, &["new", "-m", "working child"]);
        fs::write(fixture.path.join("working.txt"), "working edit\n")
            .expect("write working-copy edit");
        run_jj_in(&fixture.path, &["st"]);
    }
    fixture
}

fn two_file_edits_fixture() -> LinearFixture {
    let fixture = LinearFixture::build();
    fs::write(fixture.path.join("edit.txt"), "one\ntwo\nthree\nfour\n").unwrap();
    fs::write(fixture.path.join("untouched.txt"), "alpha\nbeta\ngamma\n").unwrap();
    run_jj_in(&fixture.path, &["describe", "-m", "base files"]);
    run_jj_in(&fixture.path, &["new"]);
    fs::write(
        fixture.path.join("edit.txt"),
        "one\nselected two\nthree\nunselected four\n",
    )
    .unwrap();
    fs::write(
        fixture.path.join("untouched.txt"),
        "alpha\nchanged beta\ngamma\n",
    )
    .unwrap();
    run_jj_in(&fixture.path, &["st"]);
    fixture
}

fn change_by_description(repo: &Repo, description: &str) -> jayjay_core::ChangeInfo {
    repo.log("all()")
        .expect("load graph")
        .into_iter()
        .find(|change| change.description.trim() == description)
        .unwrap_or_else(|| panic!("change '{description}' present"))
}

fn change_by_id(repo: &Repo, change_id: &str) -> jayjay_core::ChangeInfo {
    repo.log(change_id)
        .expect("load change")
        .into_iter()
        .next()
        .expect("change present")
}

fn select_first_changed_line(view: &gpui::Entity<RepoWindow>, cx: &mut VisualTestContext) {
    let (path, line) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let line = vm
            .current_diff
            .as_ref()
            .unwrap()
            .lines
            .iter()
            .position(|line| line.is_changed())
            .unwrap();
        (vm.selected_hunk().unwrap().path.clone(), line)
    });
    view.update_in(cx, |view, _, cx| {
        view.toggle_diff_edit_display_line(&path, line as u32 + 1, cx)
    });
}

fn open_changed_repo(
    cx: &mut TestAppContext,
) -> (
    LinearFixture,
    gpui::Entity<RepoWindow>,
    &mut VisualTestContext,
) {
    let fixture = LinearFixture::build();
    fs::write(
        fixture.path.join("README.md"),
        "# Sample project\nfirst edit\nsecond edit\n",
    )
    .unwrap();
    let (view, cx) = open_fixture(&fixture, cx);
    (fixture, view, cx)
}

fn open_fixture<'a>(
    fixture: &LinearFixture,
    cx: &'a mut TestAppContext,
) -> (Entity<RepoWindow>, &'a mut VisualTestContext) {
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    (view, cx)
}
