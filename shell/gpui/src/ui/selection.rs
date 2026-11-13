use gpui::{App, Modifiers};
use jayjay_core::dag::SelectionClick;

pub(crate) fn click_from_modifiers(modifiers: &Modifiers, cx: &App) -> SelectionClick {
    if modifiers.secondary(cx) {
        SelectionClick::Toggle
    } else if modifiers.shift {
        SelectionClick::Extend
    } else {
        SelectionClick::Replace
    }
}
