use gpui::{AppContext, TestAppContext, VisualContext, VisualTestContext};
use jayjay_gpui::app::config::{AppConfig, AppConfigStore};
use jayjay_gpui::app::theme::Theme;
use jayjay_gpui::diff::{DiffSelection, SbsSide};
use jayjay_gpui::log::{ActivePane, LogView};
use jayjay_gpui::repo::revset;
use jayjay_gpui::repo::view_model::RepoViewModel;
use jj_test_fixtures::LinearFixture;

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
    cx.update(|cx| {
        cx.set_global(AppConfigStore::new(AppConfig::default()));
        cx.set_global(Theme::light());
    });
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

fn settle_visual(cx: &mut VisualTestContext) {
    for _ in 0..8 {
        cx.run_until_parked();
        cx.cx.executor().run_until_parked();
    }
}
