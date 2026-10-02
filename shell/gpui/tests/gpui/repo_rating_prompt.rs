use crate::harness::*;
use gpui::{Entity, Focusable, Modifiers, TestAppContext, VisualTestContext};
use jayjay_gpui::app::links::REPOSITORY_URL;
use jayjay_gpui::app::rating_prompt::{RatingPromptState, RatingPromptStore};
use jayjay_gpui::repo::RepoWindow;
use jj_test::LinearFixture;

fn raise_prompt(view: &Entity<RepoWindow>, message: &str, cx: &mut VisualTestContext) {
    cx.update(|_, cx| {
        cx.set_global(RatingPromptStore::in_memory(RatingPromptState {
            next_prompt_at: 0,
            ..RatingPromptState::default()
        }))
    });
    let message = message.to_owned();
    view.update_in(cx, |view, _, cx| {
        let vm = view.view_model();
        let rev = vm
            .read(cx)
            .selected_change()
            .expect("selected change")
            .selection_revision()
            .to_owned();
        vm.update(cx, |vm, cx| vm.describe_change(rev, message, cx))
            .detach();
    });
    settle_visual(cx);
    assert!(cx.debug_bounds("rating-prompt").is_some());
}

#[gpui::test]
fn the_prompt_is_modal_and_return_escape_or_dont_show_again_close_it(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, window, cx| {
        view.focus_handle(cx).focus(window, cx)
    });

    raise_prompt(&view, "first", cx);
    cx.simulate_keystrokes("enter");
    settle_visual(cx);
    assert_eq!(opened_url(cx).as_deref(), Some(REPOSITORY_URL));
    assert!(cx.debug_bounds("rating-prompt").is_none());

    cx.simulate_keystrokes("tab tab");
    settle_visual(cx);
    assert!(view.read_with(cx, |view, _| view.focused_control().is_some()));
    raise_prompt(&view, "second", cx);
    cx.simulate_keystrokes("escape");
    settle_visual(cx);
    assert!(cx.debug_bounds("rating-prompt").is_none());

    raise_prompt(&view, "third", cx);
    let dont_show = cx
        .debug_bounds("rating-prompt-dont-show")
        .expect("don't show again");
    cx.simulate_click(dont_show.center(), Modifiers::default());
    settle_visual(cx);
    assert!(cx.debug_bounds("rating-prompt").is_none());
    assert!(cx.update(|_, cx| cx.global::<RatingPromptStore>().state().dismissed));
}
