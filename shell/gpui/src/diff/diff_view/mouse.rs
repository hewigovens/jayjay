use gpui::{IntoElement, Pixels, Styled, canvas};

use crate::log::PanelBoundsSlot;

// Absolute size_full canvas — captures the parent's bounds via prepaint
// without consuming layout, so mouse handlers can convert window x to col.
pub(super) fn bounds_capture(slot: PanelBoundsSlot) -> impl IntoElement {
    canvas(
        move |bounds, _window, _cx| {
            slot.set(Some(bounds));
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
