use gpui::{Modifiers, MouseButton, TestAppContext};
use jayjay_core::BisectPhase;
use jj_test::LinearFixture;

use crate::harness::{open_fixture, selector, settle_visual};

#[gpui::test]
fn bisect_between_selects_the_result_and_restores_the_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    let (working_copy, feature_commit, feature_ix, initial_ix) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let working_copy = vm
            .graph
            .changes
            .iter()
            .find(|change| change.is_working_copy)
            .expect("working copy")
            .change_id
            .id
            .clone();
        let feature = vm
            .graph
            .changes
            .iter()
            .position(|change| change.description.contains("add feature"))
            .expect("add feature");
        let initial = vm
            .graph
            .changes
            .iter()
            .position(|change| change.description.starts_with("initial"))
            .expect("initial");
        (
            working_copy,
            vm.graph.changes[feature].commit_id.id.clone(),
            feature,
            initial,
        )
    });

    view.update_in(cx, |view, _, cx| {
        view.handle_change_row_click(feature_ix, Modifiers::default(), cx);
        view.handle_change_row_click(initial_ix, Modifiers::secondary_key(), cx);
    });
    settle_visual(cx);

    let row = cx
        .debug_bounds(selector(format!("dag-change-{feature_commit}")))
        .expect("feature row");
    cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(cx);
    let item = cx
        .debug_bounds("context-menu-Bisect between")
        .expect("bisect between");
    cx.simulate_mouse_down(item.center(), MouseButton::Left, Modifiers::default());
    settle_visual(cx);

    for _ in 0..6 {
        let finished = view.read_with(cx, |view, cx| {
            let vm = view.view_model().read(cx);
            assert!(vm.error.is_none(), "bisect failed: {:?}", vm.error);
            vm.bisect
                .state
                .as_ref()
                .is_some_and(|state| state.phase == BisectPhase::Finished)
        });
        if finished {
            break;
        }
        let good = cx.debug_bounds("bisect-good").expect("good button");
        cx.simulate_click(good.center(), Modifiers::default());
        settle_visual(cx);
    }

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let state = vm.bisect.state.as_ref().expect("bisect result");
        assert_eq!(state.phase, BisectPhase::Finished);
        let select = state
            .result
            .as_ref()
            .and_then(|result| result.bad.first())
            .map(|change| change.change_id.clone());
        let selected = vm
            .selected_change()
            .map(|change| change.change_id.id.clone());
        assert_eq!(selected, select, "the finished result should be selected");
        let restored = vm
            .graph
            .changes
            .iter()
            .find(|change| change.is_working_copy)
            .expect("restored working copy")
            .change_id
            .id
            .clone();
        assert_eq!(restored, working_copy, "@ should be restored");
    });
}

#[test]
fn dropping_the_view_model_stops_bisect() {
    use jayjay_gpui::repo::view_model::RepoViewModel;
    use jj_test::find_change;

    let fixture = LinearFixture::build();
    let vm = RepoViewModel::new(fixture.path.clone());
    let repo = vm.repo.clone().expect("open repo");
    let original = repo.log("@").expect("log @").remove(0).change_id.id;
    let good = find_change(&repo, "initial").change_id.id;
    let bad = find_change(&repo, "add feature").change_id.id;
    repo.start_bisect_between(&good, &bad).expect("start");

    drop(vm);

    assert!(repo.bisect_state().is_none());
    assert_eq!(
        repo.log("@").expect("log @").remove(0).change_id.id,
        original
    );
}
