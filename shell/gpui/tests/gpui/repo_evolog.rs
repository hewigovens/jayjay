use crate::harness::*;
use gpui::{Modifiers, TestAppContext, VisualTestContext};
use jayjay_gpui::repo::revset;
use jayjay_gpui::ui::context_menu::ContextAction;
use jayjay_gpui::windows::evolog::EvologView;
use jj_test::{LinearFixture, run_jj_in};

fn open_evolog(fixture: &LinearFixture, cx: &mut TestAppContext) -> VisualTestContext {
    let (view, repo_cx) = open_fixture(fixture, cx);
    let rev = view.read_with(repo_cx, |view, cx| {
        revset::change_revision(
            view.view_model()
                .read(cx)
                .selected_change()
                .expect("working copy"),
        )
    });
    view.update_in(repo_cx, |view, _, cx| {
        view.dispatch_context_action(ContextAction::OpenEvologFor(rev.into()), cx);
    });
    settle_visual(repo_cx);
    let window = repo_cx
        .cx
        .windows()
        .into_iter()
        .find(|window| window.downcast::<EvologView>().is_some())
        .expect("evolog window");
    let mut evolog_cx = VisualTestContext::from_window(window, &repo_cx.cx);
    settle_visual(&mut evolog_cx);
    evolog_cx
}

fn snapshot_working_copy(fixture: &LinearFixture, contents: &str) {
    std::fs::write(fixture.path.join("wip1.txt"), contents).expect("write working copy");
    run_jj_in(&fixture.path, &["st"]);
}

fn collapsed_run_selector(cx: &mut VisualTestContext) -> Option<&'static str> {
    for start in 0..8 {
        for count in 2..24 {
            let id = selector(format!("evolog-snapshot-run-{start}-{count}"));
            if cx.debug_bounds(id).is_some() {
                return Some(id);
            }
        }
    }
    None
}

#[gpui::test]
fn evolog_hides_consecutive_snapshots_until_toggled_or_expanded(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["describe", "-m", "described"]);
    for index in 0..12 {
        snapshot_working_copy(&fixture, &format!("snap {index}\n"));
    }

    let mut evolog_cx = open_evolog(&fixture, cx);
    assert!(
        evolog_cx.debug_bounds("evolog-hide-snapshots").is_some(),
        "hide snapshots toggle"
    );

    collapsed_run_selector(&mut evolog_cx).expect("collapsed snapshot run");
    let toggle = evolog_cx
        .debug_bounds("evolog-hide-snapshots")
        .expect("hide snapshots toggle");
    evolog_cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(&mut evolog_cx);
    assert!(
        collapsed_run_selector(&mut evolog_cx).is_none(),
        "turning hide off should show every snapshot row"
    );

    let toggle = evolog_cx
        .debug_bounds("evolog-hide-snapshots")
        .expect("hide snapshots toggle");
    evolog_cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(&mut evolog_cx);
    let collapsed = collapsed_run_selector(&mut evolog_cx).expect("collapsed run after hide on");
    let expand = selector(format!("{collapsed}-label"));
    let expand_bounds = evolog_cx.debug_bounds(expand).expect("collapsed run label");
    evolog_cx.simulate_click(expand_bounds.center(), Modifiers::default());
    settle_visual(&mut evolog_cx);
    assert!(
        evolog_cx.debug_bounds(collapsed).is_none(),
        "clicking a collapsed run should expand it"
    );
}
