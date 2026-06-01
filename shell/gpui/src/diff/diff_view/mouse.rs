use gpui::{
    Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels,
    Styled, canvas,
};

use crate::diff::SbsSide;
use crate::repo::window::{PanelBoundsSlot, RepoWindow};

// Absolute overlay canvas — captures parent bounds during prepaint.
pub(super) fn bounds_capture(slot: PanelBoundsSlot) -> impl IntoElement {
    canvas(
        move |bounds, window, _cx| {
            if slot.get() != Some(bounds) {
                slot.set(Some(bounds));
                window.refresh();
            }
        },
        |_, _, _, _| {},
    )
    .absolute()
    .size_full()
}

pub(super) fn pixel_to_col(slot: &PanelBoundsSlot, x: Pixels, advance: Pixels) -> usize {
    let Some(bounds) = slot.get() else {
        return 0;
    };
    let local = (f32::from(x) - f32::from(bounds.origin.x)).max(0.);
    (local / f32::from(advance)).floor() as usize
}

// mouse_down (single → start, double → word) + mouse_move (drag-extend).
// Bound to a single side so old/new sbs panels stay independent.
pub(super) fn attach_selection_handlers<E>(
    elem: E,
    ix: usize,
    side: SbsSide,
    advance: Pixels,
    col_offset: usize,
    bounds: PanelBoundsSlot,
    cx: &mut Context<RepoWindow>,
) -> E
where
    E: InteractiveElement + 'static,
{
    let down_bounds = bounds.clone();
    elem.on_mouse_down(
        MouseButton::Left,
        cx.listener(move |v, ev: &MouseDownEvent, _, cx| {
            let col = col_offset + pixel_to_col(&down_bounds, ev.position.x, advance);
            if ev.click_count >= 2 {
                v.select_word(ix, col, side, cx);
            } else {
                v.start_diff_selection(ix, col, side, cx);
            }
        }),
    )
    .on_mouse_move(cx.listener(move |v, ev: &MouseMoveEvent, _, cx| {
        let col = col_offset + pixel_to_col(&bounds, ev.position.x, advance);
        v.extend_diff_selection(ix, col, side, cx);
    }))
}
