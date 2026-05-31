use gpui::{
    AppContext, Entity, Focusable, Modifiers, TestAppContext, VisualContext, VisualTestContext,
};
use jayjay_gpui::app::config::{self, AppConfig, AppConfigStore, AppearanceMode};
use jayjay_gpui::app::theme::Theme;
use jayjay_gpui::diff::{DiffSelection, SbsSide};
use jayjay_gpui::log::{ActivePane, LogView};
use jayjay_gpui::repo::revset;
use jayjay_gpui::repo::view_model::RepoViewModel;
use jayjay_gpui::windows::command_palette::CommandPalette;
use jj_test_fixtures::{LinearFixture, run_jj_in};

fn settle(cx: &mut TestAppContext) {
    for _ in 0..8 {
        cx.run_until_parked();
        cx.executor().run_until_parked();
    }
}

#[gpui::test]
fn opens_linear_fixture_with_working_copy_selected(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();

    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    vm.read_with(cx, |vm, _| {
        assert!(vm.error.is_none(), "open errored: {:?}", vm.error);
        assert!(vm.repo.is_some(), "repo handle should be populated");
        assert!(
            vm.graph.entries.len() >= 4,
            "linear fixture should expose at least 4 changes (initial, hello, feature, wc), got {}",
            vm.graph.entries.len()
        );
        let selected_ix = vm.selected.expect("working copy should be selected");
        let selected = &vm.graph.entries[selected_ix].change;
        assert!(
            selected.is_working_copy,
            "selected change should be the working copy, got {:?}",
            selected.change_id
        );
    });
}

#[gpui::test]
fn reselecting_current_file_does_not_reset_diff_panel(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let view = cx.new(|cx| LogView::new(fixture.path.clone(), cx));

    view.update(cx, |view, cx| {
        view.vm.update(cx, |vm, _| {
            vm.selected_file_ix = Some(0);
        });
        view.active_pane = ActivePane::Sidebar;
        view.diff.selection = Some(DiffSelection::start(2, 3, SbsSide::Unified));

        view.select_file(0, cx);

        assert_eq!(view.active_pane, ActivePane::FileColumn);
        assert!(view.diff.selection.is_some());
    });
}

#[gpui::test]
fn describe_change_refreshes_graph(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    let rev = vm.read_with(cx, |vm, _| {
        revset::change_revision(vm.selected_change().expect("selected change"))
    });
    vm.update(cx, |vm, cx| {
        vm.describe_change(rev, "updated from gpui".to_owned(), cx);
    });
    settle(cx);

    vm.read_with(cx, |vm, _| {
        let selected = vm
            .selected_change()
            .expect("selected change after describe");
        assert_eq!(selected.description, "updated from gpui");
        assert!(vm.error.is_none(), "describe errored: {:?}", vm.error);
    });
}

#[gpui::test]
fn committing_working_copy_selects_new_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    vm.update(cx, |vm, cx| {
        vm.commit_working_copy("commit from gpui".to_owned(), cx);
    });
    settle(cx);

    vm.read_with(cx, |vm, _| {
        assert!(vm.error.is_none(), "commit errored: {:?}", vm.error);
        assert!(
            vm.graph
                .changes
                .iter()
                .any(|change| change.description.trim() == "commit from gpui"),
            "committed change should be visible in graph"
        );
        let selected = vm.selected_change().expect("selected change after commit");
        assert!(
            selected.is_working_copy,
            "new working copy should be selected after commit"
        );
    });
}

#[gpui::test]
fn commit_box_input_commits_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| LogView::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let input = view.read_with(cx, |view, _| view.commit_input.clone());
    cx.focus(&input);
    cx.simulate_input("commit from gpui commit box");
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.commit_input.read(cx).text(),
            "commit from gpui commit box"
        );
    });

    view.update_in(cx, |view, _, cx| {
        view.commit_working_copy_from_input(cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert_eq!(view.commit_input.read(cx).text(), "");
        let vm = view.vm.read(cx);
        assert!(vm.error.is_none(), "commit errored: {:?}", vm.error);
        assert!(
            vm.graph
                .changes
                .iter()
                .any(|change| change.description.trim() == "commit from gpui commit box"),
            "committed change should be visible in graph"
        );
        let selected = vm.selected_change().expect("selected change after commit");
        assert!(
            selected.is_working_copy,
            "new working copy should be selected after commit"
        );
    });
}

#[gpui::test]
fn commit_box_space_does_not_toggle_file_review(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_tracked_working_copy_edits(&fixture);
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| LogView::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    let (change_id, path, identity) = view.update_in(cx, |view, _, cx| {
        view.active_pane = ActivePane::FileColumn;
        let vm = view.vm.read(cx);
        let change = vm.selected_change().expect("selected change");
        let hunk = vm.selected_hunk().expect("selected hunk");
        let marker = (
            change.change_id.clone(),
            hunk.path.clone(),
            hunk.review_identity.clone(),
        );
        view.review_store.mark_unreviewed(&marker.0, &marker.1);
        marker
    });

    let input = view.read_with(cx, |view, _| view.commit_input.clone());
    cx.focus(&input);
    cx.simulate_keystrokes("space");

    view.read_with(cx, |view, cx| {
        assert_eq!(view.commit_input.read(cx).text(), " ");
        assert!(
            !view.is_reviewed(&change_id, &path, &identity),
            "space in commit input should not mark the selected file reviewed"
        );
    });
}

#[gpui::test]
fn ctrl_n_navigates_working_copy_file_list(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_tracked_working_copy_edits(&fixture);
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| LogView::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);

    view.update_in(cx, |view, window, cx| {
        view.active_pane = ActivePane::FileColumn;
        view.focus_handle(cx).focus(window, cx);
        let vm = view.vm.read(cx);
        assert!(
            vm.files.as_ref().map(|files| files.len()).unwrap_or(0) >= 2,
            "linear fixture should expose at least two working-copy files"
        );
        assert_eq!(vm.selected_file_ix, Some(0));
    });

    cx.simulate_keystrokes("ctrl-n");

    view.read_with(cx, |view, cx| {
        assert_eq!(view.vm.read(cx).selected_file_ix, Some(1));
    });
}

#[gpui::test]
fn command_palette_ctrl_n_enter_dispatches_selected_action(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| CommandPalette::open("".into(), None, cx));
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);

    palette_cx.simulate_input("theme");
    palette_cx.simulate_keystrokes("ctrl-n ctrl-n enter");

    assert_eq!(
        palette_cx.cx.read(|cx| config::current(cx).appearance),
        AppearanceMode::Dark
    );
}

#[gpui::test]
fn command_palette_mouse_click_dispatches_action(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| CommandPalette::open("".into(), None, cx));
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);
    palette_cx.simulate_input("dark");

    let row = palette_cx
        .debug_bounds("command-palette-action-theme-dark")
        .expect("theme dark row bounds");
    palette_cx.simulate_click(row.center(), Modifiers::default());

    assert_eq!(
        palette_cx.cx.read(|cx| config::current(cx).appearance),
        AppearanceMode::Dark
    );
}

fn settle_visual(cx: &mut VisualTestContext) {
    for _ in 0..8 {
        cx.run_until_parked();
        cx.cx.run_until_parked();
        cx.cx.executor().run_until_parked();
    }
}

fn install_test_globals(cx: &mut TestAppContext) {
    cx.update(|cx| {
        cx.set_global(AppConfigStore::new(AppConfig::default()));
        cx.set_global(Theme::light());
    });
}

fn load_selected_change_files(view: &Entity<LogView>, cx: &mut VisualTestContext) {
    view.update_in(cx, |view, _, cx| {
        let selected = view.vm.read(cx).selected;
        if let Some(ix) = selected {
            view.vm.update(cx, |vm, cx| vm.select_change(ix, cx));
        }
    });
}

fn add_tracked_working_copy_edits(fixture: &LinearFixture) {
    std::fs::write(
        fixture.path.join("README.md"),
        "# Sample project\nEdited in GPUI test\n",
    )
    .expect("write README.md");
    std::fs::write(
        fixture.path.join("feature.txt"),
        "feature\nEdited in GPUI test\n",
    )
    .expect("write feature.txt");
    run_jj_in(&fixture.path, &["st"]);
}
