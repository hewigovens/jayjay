//! Canvas-based bounds capture shared by the diff panes (wrap-column math) and the
//! Markdown table renderer (content-based column widths) — both need a panel's live
//! pixel width, which isn't otherwise observable at element-build time.
use gpui::{IntoElement, Styled, canvas};

use crate::repo::window::PanelBoundsSlot;

/// Absolute overlay canvas — captures parent bounds during prepaint.
pub(crate) fn bounds_capture(slot: PanelBoundsSlot) -> impl IntoElement {
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
