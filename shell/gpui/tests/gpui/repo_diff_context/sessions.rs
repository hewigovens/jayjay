use gpui::{Focusable, Modifiers, TestAppContext, VisualTestContext};
use jayjay_gpui::repo::RepoWindow;

use super::fixtures::*;
use super::harness::*;

#[gpui::test]
fn switching_files_discards_an_in_flight_expansion_and_restarts_collapsed(cx: &mut TestAppContext) {
    let fixture = context_fixture();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    let context_ix = select_file(&view, "context.txt", cx);
    let original = largest_region(&view, cx);
    let other_ix = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .position(|hunk| hunk.path == "other.txt")
            .expect("other.txt hunk")
    });

    view.update_in(cx, |view, _, cx| {
        view.expand_context(
            original.id,
            jayjay_core::diff::ContextExpansion::ShowMore { line_count: 10 },
            cx,
        );
        view.select_file(other_ix, cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.view_model()
                .read(cx)
                .current_diff
                .as_ref()
                .map(|diff| diff.path.as_str()),
            Some("other.txt"),
            "the stale completion must not overwrite the newly selected file"
        );
    });

    view.update_in(cx, |view, _, cx| view.select_file(context_ix, cx));
    settle_visual(cx);
    assert_eq!(
        largest_region(&view, cx).line_count,
        original.line_count,
        "reselecting the file starts from the cached collapsed basis"
    );
}

#[gpui::test]
fn rapid_show_more_then_show_all_applies_the_latest_request(cx: &mut TestAppContext) {
    let fixture = context_fixture();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    select_file(&view, "context.txt", cx);
    let original = largest_region(&view, cx);

    view.update_in(cx, |view, _, cx| {
        view.expand_context(
            original.id,
            jayjay_core::diff::ContextExpansion::ShowMore { line_count: 10 },
            cx,
        );
        view.expand_context(
            original.id,
            jayjay_core::diff::ContextExpansion::ShowAll,
            cx,
        );
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert!(
            view.view_model()
                .read(cx)
                .current_diff
                .as_ref()
                .expect("fully expanded diff")
                .lines
                .iter()
                .all(|line| !line
                    .context_region
                    .is_some_and(|region| region.id == original.id)),
            "the queued Show all request must run after Show 10"
        );
    });
}

#[gpui::test]
fn expansion_recomputes_find_matches_over_revealed_context(cx: &mut TestAppContext) {
    let fixture = context_fixture();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    select_file(&view, "context.txt", cx);
    view.update_in(cx, |view, window, cx| {
        view.focus_handle(cx).focus(window, cx);
        view.open_find(cx);
    });
    cx.simulate_keystrokes("l i n e space 4");
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.find_match_count(),
            0,
            "every 'line 4x' row is inside the collapsed region"
        );
    });

    let region = largest_region(&view, cx);
    let show_all_id = selector(format!("diff-context-unified-{}-show-all", region.id));
    let show_all = cx.debug_bounds(show_all_id).expect("Show all control");
    cx.simulate_click(show_all.center(), Modifiers::default());
    settle_visual(cx);

    view.read_with(cx, |view, _| {
        assert_eq!(
            view.find_match_count(),
            10,
            "revealed context rows join the match set"
        );
    });
}
