use gpui::{Context, InteractiveElement, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels};

use crate::diff::SbsSide;
use crate::diff::line::CODE_INSET;
use crate::repo::window::{PanelBoundsSlot, RepoWindow};

fn pixel_to_col(slot: &PanelBoundsSlot, x: Pixels, advance: Pixels) -> usize {
    let Some(bounds) = slot.get() else {
        return 0;
    };
    let local = (f32::from(x) - f32::from(bounds.origin.x) - CODE_INSET).max(0.);
    (local / f32::from(advance)).floor() as usize
}

/// The line a content cell shows, and how a pointer position in it maps to a column.
pub(super) struct SelectionCell {
    pub(super) ix: usize,
    pub(super) side: SbsSide,
    pub(super) advance: Pixels,
    pub(super) col_offset: usize,
    pub(super) bounds: PanelBoundsSlot,
    /// Off for a stacked file the selection does not belong to, so a drag out of another file leaves it alone.
    pub(super) extends: bool,
}

// mouse_down (single → start, double → word) + mouse_move (drag-extend); bound to a single side so old/new sbs panels stay independent.
pub(super) fn attach_selection_handlers<E>(
    elem: E,
    cell: SelectionCell,
    cx: &mut Context<RepoWindow>,
) -> E
where
    E: InteractiveElement + 'static,
{
    let SelectionCell {
        ix,
        side,
        advance,
        col_offset,
        bounds,
        extends,
    } = cell;
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
        if !extends || ev.pressed_button != Some(MouseButton::Left) {
            return;
        }
        let col = col_offset + pixel_to_col(&bounds, ev.position.x, advance);
        v.extend_diff_selection(ix, col, side, cx);
    }))
}
