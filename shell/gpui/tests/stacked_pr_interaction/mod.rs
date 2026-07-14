use gpui::{KeyBinding, TestAppContext, VisualContext, VisualTestContext};
use jayjay_gpui::app::actions::SubmitStackedPr;
use jayjay_gpui::repo::window::StackedPrSnapshot;

use crate::{open_stack, settle_visual};

fn bind_submit_key(cx: &mut VisualTestContext) {
    cx.update(|_window, cx| {
        cx.bind_keys([KeyBinding::new(
            "enter",
            SubmitStackedPr,
            Some("StackedPrPanel && !StackedPrInput"),
        )]);
    });
}

#[gpui::test]
fn enter_submits_when_names_are_valid(cx: &mut TestAppContext) {
    let (_fixture, view, provider, cx) = open_stack(cx);
    bind_submit_key(cx);
    cx.focus(&view);
    cx.simulate_keystrokes("enter");
    settle_visual(cx);
    assert_eq!(provider.submitted.lock().unwrap().len(), 1);
    assert!(matches!(
        view.read_with(cx, |view, _| view.stacked_pr_snapshot()),
        StackedPrSnapshot::Results { .. }
    ));
}

#[gpui::test]
fn enter_finishes_bookmark_edit_before_submitting(cx: &mut TestAppContext) {
    let (_fixture, view, provider, cx) = open_stack(cx);
    bind_submit_key(cx);
    view.update_in(cx, |view, _, cx| {
        view.edit_stacked_pr_name(0, "edited-base", cx)
    });
    cx.focus(&view);

    cx.simulate_keystrokes("enter");
    settle_visual(cx);
    assert!(provider.submitted.lock().unwrap().is_empty());

    cx.simulate_keystrokes("enter");
    settle_visual(cx);
    assert_eq!(provider.submitted.lock().unwrap().len(), 1);
    assert!(matches!(
        view.read_with(cx, |view, _| view.stacked_pr_snapshot()),
        StackedPrSnapshot::Results { .. }
    ));
}

#[gpui::test]
fn base_labels_track_edits_to_the_layer_below(cx: &mut TestAppContext) {
    let (_fixture, view, _provider, cx) = open_stack(cx);
    view.update_in(cx, |view, _, cx| {
        view.edit_stacked_pr_name(0, "edited-base", cx)
    });
    let StackedPrSnapshot::Preview { bases, .. } =
        view.read_with(cx, |view, _| view.stacked_pr_snapshot())
    else {
        panic!("expected preview");
    };
    assert_eq!(bases, ["main", "edited-base"]);
}

#[gpui::test]
fn escape_does_not_close_the_panel_while_submitting(cx: &mut TestAppContext) {
    let (_fixture, view, _provider, cx) = open_stack(cx);
    view.update_in(cx, |view, _, cx| {
        view.submit_stacked_pr(cx);
        view.close_stacked_pr(cx);
    });
    let snapshot = view.read_with(cx, |view, _| view.stacked_pr_snapshot());
    assert_eq!(snapshot, StackedPrSnapshot::Submitting);
    cx.simulate_keystrokes("escape");
    let snapshot = view.read_with(cx, |view, _| view.stacked_pr_snapshot());
    assert_ne!(snapshot, StackedPrSnapshot::Closed);
    settle_visual(cx);
    let snapshot = view.read_with(cx, |view, _| view.stacked_pr_snapshot());
    assert!(matches!(snapshot, StackedPrSnapshot::Results { .. }));
}
