use crate::harness::*;
use gpui::{
    Entity, Keystroke, Modifiers, MouseButton, TestAppContext, VisualContext, VisualTestContext,
};
use jayjay_gpui::repo::{ActivePane, RepoWindow};
use jayjay_gpui::ui::context_menu::ContextAction;
use jj_test::{LinearFixture, run_jj_in};

#[gpui::test]
fn commit_box_prefills_from_working_copy_description(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(
        &fixture.path,
        &["describe", "-m", "existing summary\n\nexisting body"],
    );
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert_eq!(view.summary_input().read(cx).text(), "existing summary");
        assert_eq!(view.description_input().read(cx).text(), "existing body");
    });
}

#[gpui::test]
fn commit_box_preserves_draft_when_working_copy_changes(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let old_change_id = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| change.is_working_copy)
            .expect("working copy")
            .change_id
            .id
            .clone()
    });
    let summary = view.read_with(cx, |view, _| view.summary_input().clone());
    cx.focus(&summary);
    cx.simulate_input("keep this draft");

    view.update_in(cx, |view, _, cx| {
        view.dispatch_context_action(ContextAction::NewChangeOnTop("@".into()), cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let working_copy = view
            .view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| change.is_working_copy)
            .expect("new working copy");
        assert_ne!(working_copy.change_id.id, old_change_id);
        assert!(working_copy.description.is_empty());
        assert_eq!(view.summary_input().read(cx).text(), "keep this draft");
    });
}

#[gpui::test]
fn commit_box_input_commits_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let input = view.read_with(cx, |view, _| view.summary_input().clone());
    cx.focus(&input);
    cx.simulate_input("commit from gpui commit box");
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.summary_input().read(cx).text(),
            "commit from gpui commit box"
        );
    });

    view.update_in(cx, |view, _, cx| {
        view.commit_working_copy_from_input(cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert_eq!(view.summary_input().read(cx).text(), "");
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "commit errored: {:?}", vm.error);
        assert!(
            vm.graph
                .changes
                .iter()
                .any(|change| change.description.trim() == "commit from gpui commit box")
        );
        let selected = vm.selected_change().expect("selected change after commit");
        assert!(selected.is_working_copy);
    });
}

#[gpui::test]
fn describe_button_sets_description_without_new_change(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let (change_count, working_copy_id) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let wc = vm
            .graph
            .changes
            .iter()
            .find(|c| c.is_working_copy)
            .expect("working copy in graph");
        (vm.graph.changes.len(), wc.change_id.id.clone())
    });

    let summary = view.read_with(cx, |view, _| view.summary_input().clone());
    cx.focus(&summary);
    cx.simulate_input("describe from gpui");
    let description = view.read_with(cx, |view, _| view.description_input().clone());
    cx.focus(&description);
    cx.simulate_input("body details");

    view.update_in(cx, |view, _, cx| {
        view.describe_working_copy_from_input(cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "describe errored: {:?}", vm.error);
        assert_eq!(
            vm.graph.changes.len(),
            change_count,
            "describe must not create a new change"
        );
        let wc = vm
            .graph
            .changes
            .iter()
            .find(|c| c.is_working_copy)
            .expect("working copy after describe");
        assert_eq!(
            wc.change_id.id, working_copy_id,
            "@ keeps its change id under describe"
        );
        assert_eq!(wc.description.trim(), "describe from gpui\n\nbody details");
        let selected = vm.selected_change().expect("selection after describe");
        assert!(selected.is_working_copy, "selection stays on @");
        // The box mirrors @'s description, which the describe just set: the inputs round-trip through the refresh untouched.
        assert_eq!(view.summary_input().read(cx).text(), "describe from gpui");
        assert_eq!(view.description_input().read(cx).text(), "body details");
    });
}

#[gpui::test]
fn describe_with_empty_box_toasts_and_keeps_description(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        view.describe_working_copy_from_input(cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        // SwiftUI disables Describe on an empty draft; the GPUI equivalent of that gate is the toast, and @'s description must stay untouched either way.
        let toast = view.toast().expect("empty describe toast");
        assert!(
            toast.contains("Description required"),
            "unexpected toast: {toast}"
        );
        let vm = view.view_model().read(cx);
        let wc = vm
            .graph
            .changes
            .iter()
            .find(|c| c.is_working_copy)
            .expect("working copy in graph");
        assert_eq!(wc.description, "", "empty describe must not clear or set @");
    });
}

#[gpui::test]
fn commit_requires_a_summary_even_with_a_body(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let change_count = view.read_with(cx, |view, cx| {
        view.view_model().read(cx).graph.changes.len()
    });
    let description = view.read_with(cx, |view, _| view.description_input().clone());
    cx.focus(&description);
    cx.simulate_input("body without a summary");

    view.update_in(cx, |view, _, cx| {
        view.commit_working_copy_from_input(cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        // SwiftUI disables Commit while the summary is blank; a body-only draft may Describe but must never commit with an empty subject line.
        let toast = view.toast().expect("summary-required toast");
        assert!(
            toast.contains("Summary required"),
            "unexpected toast: {toast}"
        );
        let vm = view.view_model().read(cx);
        assert_eq!(
            vm.graph.changes.len(),
            change_count,
            "a summary-less commit must not create a change"
        );
        assert_eq!(
            view.description_input().read(cx).text(),
            "body without a summary",
            "the rejected draft stays in the box"
        );
    });
}

#[gpui::test]
fn commit_box_buttons_follow_swiftui_order(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let commit_box = cx.debug_bounds("commit-box-editor").expect("commit box");
    let commit_button = cx
        .debug_bounds("commit-working-copy")
        .expect("commit button");
    assert!(
        commit_button.size.width < commit_box.size.width * 0.5,
        "commit button should stay compact instead of filling the commit box"
    );
    assert!(
        commit_button.origin.x > commit_box.origin.x + commit_box.size.width * 0.5,
        "commit button should sit on the trailing side of the commit box"
    );
    let describe_button = cx
        .debug_bounds("describe-working-copy")
        .expect("describe button");
    let sparkle = cx
        .debug_bounds("commit-ai-generate")
        .expect("generate button");
    assert!(
        sparkle.origin.x < describe_button.origin.x
            && describe_button.origin.x < commit_button.origin.x,
        "SwiftUI button order is sparkle, Describe, Commit"
    );
}

#[gpui::test]
fn commit_clears_working_copy_review_marks(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let (change_id, files) = view.update_in(cx, |view, _, cx| {
        let (change_id, files) = {
            let vm = view.view_model().read(cx);
            let change = vm.selected_change().expect("selected working copy");
            assert!(change.is_working_copy);
            let files: Vec<_> = vm
                .files
                .as_ref()
                .expect("working copy files loaded")
                .iter()
                .map(|hunk| (hunk.path.clone(), hunk.review_identity.clone()))
                .collect();
            (change.change_id.id.clone(), files)
        };
        assert!(
            !files.is_empty(),
            "fixture should expose working copy files"
        );

        for (path, identity) in &files {
            view.toggle_reviewed(change_id.clone(), path.clone(), identity.clone(), cx);
            assert!(view.is_reviewed(&change_id, path, identity));
        }
        (change_id, files)
    });

    let input = view.read_with(cx, |view, _| view.summary_input().clone());
    cx.focus(&input);
    cx.simulate_input("commit reviewed working copy");
    view.update_in(cx, |view, _, cx| {
        view.commit_working_copy_from_input(cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(
            vm.graph
                .changes
                .iter()
                .any(|change| change.change_id.id == change_id && !change.is_working_copy),
            "committed change should keep the working copy change id"
        );
        for (path, identity) in &files {
            assert!(
                !view.is_reviewed(&change_id, path, identity),
                "committed file {path} should not inherit the working copy review mark"
            );
        }
    });
}

#[gpui::test]
fn commit_box_keeps_input_when_commit_fails(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let input = view.read_with(cx, |view, _| view.summary_input().clone());
    cx.focus(&input);
    cx.simulate_input("keep this message");

    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, _| {
            vm.repo = None;
        });
        view.commit_working_copy_from_input(cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert_eq!(view.summary_input().read(cx).text(), "keep this message");
        assert_eq!(
            view.view_model().read(cx).error.as_deref(),
            Some("Repository is not open")
        );
    });
}

#[gpui::test]
fn commit_box_space_does_not_toggle_file_review(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let (change_id, path, identity) = view.update_in(cx, |view, _, cx| {
        view.set_active_pane(ActivePane::FileColumn);
        let vm = view.view_model().read(cx);
        let change = vm.selected_change().expect("selected change");
        let hunk = vm.selected_hunk().expect("selected hunk");
        let marker = (
            change.change_id.id.clone(),
            hunk.path.clone(),
            hunk.review_identity.clone(),
        );
        view.mark_unreviewed(&marker.0, &marker.1);
        marker
    });

    let input = view.read_with(cx, |view, _| view.summary_input().clone());
    cx.focus(&input);
    cx.simulate_keystrokes("space");

    view.read_with(cx, |view, cx| {
        assert_eq!(view.summary_input().read(cx).text(), " ");
        assert!(!view.is_reviewed(&change_id, &path, &identity));
    });
}

#[gpui::test]
fn file_column_hide_reviewed_button_filters_reviewed_files(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let reviewed_ix = view.update_in(cx, |view, _, cx| {
        let (change_id, files) = {
            let vm = view.view_model().read(cx);
            let change = vm.selected_change().expect("selected working copy");
            assert!(change.is_working_copy);
            let files: Vec<_> = vm
                .files
                .as_ref()
                .expect("working copy files loaded")
                .iter()
                .map(|hunk| (hunk.path.clone(), hunk.review_identity.clone()))
                .collect();
            (change.change_id.id.clone(), files)
        };
        assert!(files.len() >= 2, "fixture should expose multiple files");
        view.view_model()
            .update(cx, |vm, _| vm.selected_file_ix = Some(0));
        let (path, identity) = files[0].clone();
        view.toggle_reviewed(change_id, path, identity, cx);
        0
    });
    settle_visual(cx);

    let toggle = cx
        .debug_bounds("file-hide-reviewed")
        .expect("hide reviewed button");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert!(view.hide_reviewed_files());
        assert_ne!(
            view.view_model().read(cx).selected_file_ix,
            Some(reviewed_ix),
            "hiding reviewed files should move selection off the hidden reviewed file"
        );
    });
}

#[gpui::test]
fn a_new_window_skips_a_hidden_reviewed_file_when_selecting_the_first_file(
    cx: &mut TestAppContext,
) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (first, vcx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    load_selected_change_files(&first, vcx);
    settle_visual(vcx);
    first.update_in(vcx, |view, _, cx| {
        let (change_id, path, identity) = {
            let vm = view.view_model().read(cx);
            let change = vm.selected_change().expect("selected working copy");
            let file = &vm.files.as_ref().expect("working copy files loaded")[0];
            (
                change.change_id.id.clone(),
                file.path.clone(),
                file.review_identity.clone(),
            )
        };
        view.toggle_reviewed(change_id, path, identity, cx);
    });
    settle_visual(vcx);
    cx.update(|cx| jayjay_gpui::app::config::update(cx, |c| c.diff.hide_reviewed_files = true));

    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.files.as_ref().is_some_and(|files| files.len() >= 2));
        assert!(
            vm.selected_file_ix.is_some_and(|ix| ix != 0),
            "the initial selection should skip the hidden reviewed file: {:?}",
            vm.selected_file_ix
        );
    });
}

#[gpui::test]
fn hide_reviewed_files_preference_hydrates_new_windows(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    cx.update(|cx| jayjay_gpui::app::config::update(cx, |c| c.diff.hide_reviewed_files = true));

    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    view.read_with(cx, |view, _| assert!(view.hide_reviewed_files()));
    assert!(
        cx.debug_bounds("file-hide-reviewed").is_some(),
        "the eye toggle stays visible while the filter is on, even with nothing reviewed yet"
    );
}

#[gpui::test]
fn review_marks_are_shared_across_windows(cx: &mut TestAppContext) {
    // Two windows on the same repo must share one process-wide review store, not per-window copies.
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);

    let window_b = cx
        .add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx))
        .0;
    let (window_a, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&window_a, cx);
    settle_visual(cx);

    let (change_id, path, identity) = window_a.update_in(cx, |view, _, cx| {
        let vm = view.view_model().read(cx);
        let change = vm.selected_change().expect("selected change");
        let hunk = vm.selected_hunk().expect("selected hunk");
        let marker = (
            change.change_id.id.clone(),
            hunk.path.clone(),
            hunk.review_identity.clone(),
        );
        view.mark_unreviewed(&marker.0, &marker.1);
        marker
    });

    window_a.update_in(cx, |view, _, cx| {
        view.toggle_reviewed(change_id.clone(), path.clone(), identity.clone(), cx);
        assert!(view.is_reviewed(&change_id, &path, &identity), "A marked");
    });

    window_b.read_with(cx, |view, _| {
        assert!(
            view.is_reviewed(&change_id, &path, &identity),
            "B should observe the mark A made"
        );
    });

    // Unmarking in B is visible in A — and leaves the on-disk store as found.
    window_b.update_in(cx, |view, _, cx| {
        view.toggle_reviewed(change_id.clone(), path.clone(), identity.clone(), cx);
    });
    window_a.read_with(cx, |view, _| {
        assert!(
            !view.is_reviewed(&change_id, &path, &identity),
            "A should observe the unmark B made"
        );
    });
}

fn working_copy_rev(view: &RepoWindow, cx: &gpui::App) -> gpui::SharedString {
    view.view_model()
        .read(cx)
        .graph
        .changes
        .iter()
        .find(|change| change.is_working_copy)
        .expect("working copy")
        .selection_revision()
        .to_owned()
        .into()
}

fn description_of(fixture: &LinearFixture, rev: &str) -> String {
    let output = run_jj_in(
        &fixture.path,
        &["log", "--no-graph", "-r", rev, "-T", "description"],
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

#[gpui::test]
fn a_described_working_copy_commits_once_from_its_change_menu(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let parent = description_of(&fixture, "@-");
    run_jj_in(&fixture.path, &["describe", "-m", "saved description"]);
    let (view, cx) = open_fixture(&fixture, cx);
    assert!(cx.debug_bounds("commit-box-editor").is_none());

    let commit_id = selected_commit_id(&view, cx);
    let row = cx
        .debug_bounds(selector(format!("dag-change-{commit_id}")))
        .expect("working-copy row");
    cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(cx);
    assert!(menu_shows(cx, &jayjay_i18n::repo::commit()));
    keys(cx, "escape");
    view.update(cx, |view, cx| {
        let rev = working_copy_rev(view, cx);
        view.dispatch_context_action(ContextAction::CommitWorkingCopy(rev.clone()), cx);
        view.dispatch_context_action(ContextAction::CommitWorkingCopy(rev), cx);
    });
    settle_visual(cx);

    assert_eq!(description_of(&fixture, "@-"), "saved description");
    assert_eq!(description_of(&fixture, "@--"), parent);
    assert!(cx.debug_bounds("commit-box-editor").is_some());
}

#[gpui::test]
fn committing_the_saved_description_from_the_sheet_keeps_a_typed_draft(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    let summary = view.read_with(cx, |view, _| view.summary_input().clone());
    cx.focus(&summary);
    cx.simulate_input("typed draft");
    run_jj_in(&fixture.path, &["describe", "-m", "saved description"]);
    view.update(cx, |view, cx| {
        view.view_model().update(cx, |vm, cx| vm.refresh(false, cx))
    });
    settle_visual(cx);
    assert!(cx.debug_bounds("commit-box-editor").is_some());

    click(cx, "edit-description");
    click(cx, "text-modal-commit");

    assert_eq!(description_of(&fixture, "@-"), "saved description");
    view.read_with(cx, |view, cx| {
        assert!(!view.has_text_modal());
        assert_eq!(view.summary_input().read(cx).text(), "typed draft");
    });
}

fn open_working_copy_sheet<'a>(
    fixture: &LinearFixture,
    cx: &'a mut TestAppContext,
) -> (Entity<RepoWindow>, &'a mut VisualTestContext) {
    run_jj_in(&fixture.path, &["describe", "-m", "saved description"]);
    let (view, cx) = open_fixture(fixture, cx);
    click(cx, "edit-description");
    (view, cx)
}

#[gpui::test]
fn a_landing_sheet_commit_leaves_a_sheet_opened_since_alone(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_working_copy_sheet(&fixture, cx);
    start_sheet_commit(cx);
    cx.update(|window, cx| {
        window.dispatch_keystroke(Keystroke::parse("escape").unwrap(), cx);
    });
    view.update(cx, |view, cx| {
        assert!(!view.has_text_modal());
        view.dispatch_context_action(ContextAction::CreateBookmark("@".into()), cx);
    });
    settle_visual(cx);

    assert_eq!(description_of(&fixture, "@-"), "saved description");
    assert!(view.read_with(cx, |view, _| view.has_text_modal()));
}

#[gpui::test]
fn saving_the_sheet_while_its_commit_runs_does_not_describe(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_working_copy_sheet(&fixture, cx);
    start_sheet_commit(cx);
    view.update(cx, |view, cx| {
        view.text_modal_input()
            .expect("summary")
            .update(cx, |input, cx| input.set_text("described meanwhile", cx));
        view.submit_text_modal(cx);
    });
    settle_visual(cx);

    assert_eq!(description_of(&fixture, "@-"), "saved description");
    assert_eq!(description_of(&fixture, "@"), "");
}

#[gpui::test]
fn a_commit_refuses_once_the_working_copy_moved_under_it(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_working_copy_sheet(&fixture, cx);
    let rev = view.read_with(cx, working_copy_rev);
    view.update_in(cx, |view, _, cx| {
        view.text_modal_input()
            .expect("summary")
            .update(cx, |input, cx| input.set_text("sheet message", cx));
    });
    run_jj_in(&fixture.path, &["new"]);
    view.update(cx, |view, cx| {
        view.view_model().update(cx, |vm, cx| vm.refresh(false, cx))
    });
    settle_visual(cx);
    let refused = Some(jayjay_i18n::repo::working_copy_moved_nothing_committed());

    click(cx, "text-modal-commit");
    assert_eq!(
        view.read_with(cx, |view, _| view.toast()).as_deref(),
        refused.as_deref()
    );
    assert!(view.read_with(cx, |view, _| view.has_text_modal()));

    keys(cx, "escape");
    view.update(cx, |view, cx| {
        view.dispatch_context_action(ContextAction::CommitWorkingCopy(rev), cx);
    });
    settle_visual(cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.toast()).as_deref(),
        refused.as_deref()
    );
    assert_eq!(description_of(&fixture, "@"), "");
    assert_eq!(description_of(&fixture, "@-"), "saved description");
}

#[gpui::test]
fn another_change_saves_its_description_while_the_working_copy_commits(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    let parent = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| !change.is_working_copy)
            .expect("parent")
            .selection_revision()
            .to_owned()
    });
    view.update(cx, |view, cx| {
        view.summary_input()
            .update(cx, |input, cx| input.set_text("committed", cx));
        view.commit_working_copy_from_input(cx);
        view.dispatch_context_action(
            ContextAction::EditDescription {
                rev: parent.clone().into(),
                description: "".into(),
            },
            cx,
        );
        view.text_modal_input()
            .expect("summary")
            .update(cx, |input, cx| input.set_text("described meanwhile", cx));
        view.submit_text_modal(cx);
    });
    settle_visual(cx);

    assert_eq!(description_of(&fixture, &parent), "described meanwhile");
    assert_eq!(description_of(&fixture, "@-"), "committed");
}
