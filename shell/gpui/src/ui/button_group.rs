use gpui::{
    AnyElement, Div, InteractiveElement, ParentElement, SharedString, Stateful,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::theme::Theme;
use crate::ui::primitives::{HelpTag, TOOLBAR_BUTTON_HEIGHT, TOOLBAR_BUTTON_WIDTH};

/// Leading/trailing get the capsule's fully rounded outer caps; inner segments stay flat-sided.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GroupEdge {
    Leading,
    Inner,
    Trailing,
}

/// No background or border of its own at rest — `button_group` supplies the shared fill; only a hover/press fill is scoped to this segment.
pub(crate) fn group_item(
    id: impl Into<SharedString>,
    tooltip: impl Into<SharedString>,
    edge: GroupEdge,
    theme: &Theme,
) -> Stateful<Div> {
    let tooltip = tooltip.into();
    let el = div()
        .id(id.into())
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .w(px(TOOLBAR_BUTTON_WIDTH))
        .h_full()
        .cursor_pointer()
        .hover(|s| s.bg(rgb(theme.row_alt_bg)))
        .active(|s| s.bg(rgb(theme.selected_bg)))
        .help(tooltip);
    match edge {
        GroupEdge::Leading => el.rounded_l_full(),
        GroupEdge::Trailing => el.rounded_r_full(),
        GroupEdge::Inner => el,
    }
}

/// Supplies the shared background for `group_item`s and rounds only the outer ends.
pub(crate) fn button_group(theme: &Theme, children: Vec<AnyElement>) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .h(px(TOOLBAR_BUTTON_HEIGHT))
        .rounded_full()
        .overflow_hidden()
        .bg(rgb(theme.toolbar_group_bg))
        .children(children)
}
