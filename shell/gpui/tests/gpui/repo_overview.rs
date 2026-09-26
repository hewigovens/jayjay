use crate::harness::*;
use gpui::{TestAppContext, VisualContext, VisualTestContext};
use jayjay_gpui::app::actions::{OpenFind, OpenOverview};
use jayjay_gpui::windows::overview::OverviewView;
use jj_test::{LinearFixture, run_jj_in};

fn fixture_with_lanes() -> LinearFixture {
    let fixture = LinearFixture::build();
    for (key, value) in [
        ("revset-aliases.'trunk()'", "main"),
        ("revset-aliases.'immutable_heads()'", "main"),
    ] {
        run_jj_in(&fixture.path, &["config", "set", "--repo", key, value]);
    }
    run_jj_in(&fixture.path, &["describe", "-m", "billing: wip"]);
    run_jj_in(&fixture.path, &["new", "-m", "search: normalize", "main"]);
    run_jj_in(&fixture.path, &["new", "-m", "search: rank"]);
    run_jj_in(&fixture.path, &["edit", "subject(\"billing: wip\")"]);
    fixture
}

fn lane_selector(prefix: String) -> &'static str {
    selector(format!("overview-lane-{prefix}"))
}

fn open_overview(
    view: &gpui::Entity<jayjay_gpui::repo::RepoWindow>,
    repo_cx: &mut VisualTestContext,
) -> VisualTestContext {
    repo_cx.focus(view);
    repo_cx.dispatch_action(OpenOverview);
    settle_visual(repo_cx);
    let window = repo_cx
        .cx
        .windows()
        .into_iter()
        .find(|window| window.downcast::<OverviewView>().is_some())
        .expect("overview window");
    let mut overview_cx = VisualTestContext::from_window(window, &repo_cx.cx);
    settle_visual(&mut overview_cx);
    overview_cx
}

#[gpui::test]
fn overview_filters_lanes_and_reveals_the_keyboard_selection_in_the_graph(cx: &mut TestAppContext) {
    let fixture = fixture_with_lanes();
    let (view, repo_cx) = open_fixture(&fixture, cx);
    let billing = change_with_subject(&view, repo_cx, "billing: wip");
    let rank = change_with_subject(&view, repo_cx, "search: rank");
    let billing_lane = lane_selector(billing.change_id.unique_prefix());
    let search_lane = lane_selector(rank.change_id.unique_prefix());

    let mut overview_cx = open_overview(&view, repo_cx);
    assert!(overview_cx.debug_bounds(billing_lane).is_some());
    assert!(overview_cx.debug_bounds(search_lane).is_some());

    overview_cx.dispatch_action(OpenFind);
    overview_cx.simulate_input("rank");
    settle_visual(&mut overview_cx);
    assert!(overview_cx.debug_bounds(billing_lane).is_none());
    assert!(overview_cx.debug_bounds(search_lane).is_some());

    overview_cx.simulate_keystrokes("enter down");
    settle_visual(&mut overview_cx);
    assert!(
        overview_cx.debug_bounds("overview-change-panel").is_some(),
        "down from the card selects the lane head and opens its details"
    );

    overview_cx.simulate_keystrokes("enter");
    settle_visual(&mut overview_cx);
    settle_visual(repo_cx);
    view.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(
            vm.revset.as_ref(),
            jayjay_core::ancestors_revset(&rank.change_id.id)
        );
        let selected = vm
            .selected
            .map(|ix| vm.graph.changes[ix].commit_id.id.clone());
        assert_eq!(selected.as_deref(), Some(rank.commit_id.id.as_str()));
    });
}

#[gpui::test]
fn change_panel_description_wraps_and_copies_as_selected_text(cx: &mut TestAppContext) {
    let fixture = fixture_with_lanes();
    let body = "Rank results by recency and relevance so the freshest matching products surface first, even when older listings share more keywords with the query.";
    run_jj_in(
        &fixture.path,
        &[
            "describe",
            "-m",
            "search: rank",
            "-m",
            body,
            "subject(\"search: rank\")",
        ],
    );
    let (view, repo_cx) = open_fixture(&fixture, cx);
    let mut overview_cx = open_overview(&view, repo_cx);

    overview_cx.simulate_keystrokes("right down");
    settle_visual(&mut overview_cx);
    let label = overview_cx
        .debug_bounds("overview-panel-body")
        .expect("description body");
    assert!(
        label.size.height > gpui::px(30.),
        "a long body wraps onto several lines, got {:?}",
        label.size.height
    );

    overview_cx.simulate_click(label.center(), gpui::Modifiers::default());
    overview_cx.simulate_keystrokes(&format!("{0}-a {0}-c", jayjay_gpui::platform::MOD_KEY));
    settle_visual(&mut overview_cx);
    let copied = overview_cx
        .cx
        .read_from_clipboard()
        .and_then(|item| item.text());
    assert_eq!(copied.as_deref(), Some(body));
}

#[gpui::test]
fn clicking_a_lane_card_opens_its_panel_and_escape_closes_it(cx: &mut TestAppContext) {
    let fixture = fixture_with_lanes();
    let (view, repo_cx) = open_fixture(&fixture, cx);
    let rank = change_with_subject(&view, repo_cx, "search: rank");
    let search_lane = lane_selector(rank.change_id.unique_prefix());
    let mut overview_cx = open_overview(&view, repo_cx);
    assert!(overview_cx.debug_bounds("overview-lane-panel").is_none());

    let card = overview_cx.debug_bounds(search_lane).expect("search lane");
    overview_cx.simulate_click(card.center(), gpui::Modifiers::default());
    settle_visual(&mut overview_cx);
    assert!(overview_cx.debug_bounds("overview-lane-panel").is_some());

    overview_cx.simulate_keystrokes("down");
    settle_visual(&mut overview_cx);
    assert!(overview_cx.debug_bounds("overview-change-panel").is_some());

    overview_cx.simulate_keystrokes("escape");
    settle_visual(&mut overview_cx);
    assert!(
        overview_cx.debug_bounds("overview-lane-panel").is_some(),
        "clearing the change falls back to the lane panel"
    );

    overview_cx.simulate_keystrokes("escape");
    settle_visual(&mut overview_cx);
    assert!(overview_cx.debug_bounds("overview-lane-panel").is_none());
}
