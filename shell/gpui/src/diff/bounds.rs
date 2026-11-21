//! Canvas-based bounds capture shared by the diff panes (wrap-column math) and the
//! Markdown table renderer (content-based column widths) — both need a panel's live
//! pixel width, which isn't otherwise observable at element-build time.
use gpui::{Bounds, IntoElement, Pixels, Styled, canvas};

use crate::repo::window::PanelBoundsSlot;

/// Absolute overlay canvas — captures parent bounds during prepaint; a size change schedules the next frame because `refresh` is a no-op mid-draw.
pub(crate) fn bounds_capture(slot: PanelBoundsSlot) -> impl IntoElement {
    capture(slot, |old, new| old == new)
}

/// For panels stacked in one column that share a slot: only the extent they have in common counts, or each would replace the other's every frame.
pub(crate) fn column_bounds_capture(slot: PanelBoundsSlot) -> impl IntoElement {
    capture(slot, |old, new| {
        old.left() == new.left() && old.size.width == new.size.width
    })
}

fn capture(
    slot: PanelBoundsSlot,
    unchanged: fn(Bounds<Pixels>, Bounds<Pixels>) -> bool,
) -> impl IntoElement {
    canvas(
        move |bounds, window, _cx| {
            if !slot.get().is_some_and(|old| unchanged(old, bounds)) {
                slot.set(Some(bounds));
                window.request_animation_frame();
            }
        },
        |_, _, _, _| {},
    )
    .absolute()
    .size_full()
}
