use gpui::{Context, InteractiveElement, MouseButton, MouseDownEvent, MouseMoveEvent};

use crate::repo::window::RepoWindow;

pub(super) fn attach_gutter_selection_handlers<E>(
    elem: E,
    path: String,
    line_ix: usize,
    cx: &mut Context<RepoWindow>,
) -> E
where
    E: InteractiveElement + 'static,
{
    let elem = with_click_to_select(elem, path.clone(), line_ix, cx);
    let elem = with_drag_extend(elem, path.clone(), line_ix, cx);
    with_right_click_menu(elem, path, line_ix, cx)
}

fn with_click_to_select<E>(elem: E, path: String, line_ix: usize, cx: &mut Context<RepoWindow>) -> E
where
    E: InteractiveElement + 'static,
{
    elem.on_mouse_down(
        MouseButton::Left,
        cx.listener(move |v, ev: &MouseDownEvent, _, cx| {
            if ev.modifiers.shift {
                v.shift_extend_gutter_selection(path.clone(), line_ix, cx);
            } else {
                v.start_gutter_selection(path.clone(), line_ix, cx);
            }
        }),
    )
}

fn with_drag_extend<E>(elem: E, path: String, line_ix: usize, cx: &mut Context<RepoWindow>) -> E
where
    E: InteractiveElement + 'static,
{
    elem.on_mouse_move(cx.listener(move |v, ev: &MouseMoveEvent, _, cx| {
        if ev.pressed_button == Some(MouseButton::Left) {
            v.extend_gutter_selection(&path, line_ix, cx);
        }
    }))
}

fn with_right_click_menu<E>(
    elem: E,
    path: String,
    line_ix: usize,
    cx: &mut Context<RepoWindow>,
) -> E
where
    E: InteractiveElement + 'static,
{
    elem.on_mouse_down(
        MouseButton::Right,
        cx.listener(move |v, ev: &MouseDownEvent, _, cx| {
            v.open_gutter_context_menu(path.clone(), line_ix, ev.position, cx);
        }),
    )
}
