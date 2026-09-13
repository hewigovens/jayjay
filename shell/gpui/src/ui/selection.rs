use gpui::Modifiers;
use jayjay_core::dag::SelectionClick;

pub(crate) fn click_from_modifiers(modifiers: &Modifiers) -> SelectionClick {
    if modifiers.secondary() {
        SelectionClick::Toggle
    } else if modifiers.shift {
        SelectionClick::Extend
    } else {
        SelectionClick::Replace
    }
}
