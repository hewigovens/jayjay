use gpui::{AppContext, TestAppContext};
use jayjay_gpui::diff::{DiffSelection, SbsSide};
use jayjay_gpui::log::{ActivePane, LogView};
use jayjay_gpui::repo::view_model::RepoViewModel;
use jj_test_fixtures::LinearFixture;

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
