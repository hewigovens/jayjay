use gpui::{
    AnyElement, Div, InteractiveElement, ParentElement, SharedString, Stateful,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::theme::Theme;
use crate::ui::icons;
use crate::ui::primitives::{
    TOOLBAR_BUTTON_HEIGHT, TOOLBAR_BUTTON_WIDTH, TOOLBAR_ICON_SIZE, text_tooltip,
};

/// Leading/trailing get the capsule's fully rounded outer caps; inner segments stay flat-sided.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GroupEdge {
    Leading,
    Inner,
    Trailing,
}

/// No background or border of its own at rest — `button_group` supplies the shared fill; only a hover/press fill is scoped to this segment.
pub fn group_item(
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
        .tooltip(text_tooltip(tooltip));
    match edge {
        GroupEdge::Leading => el.rounded_l_full(),
        GroupEdge::Trailing => el.rounded_r_full(),
        GroupEdge::Inner => el,
    }
}

pub fn group_icon_item(
    id: impl Into<SharedString>,
    glyph_str: &'static str,
    tooltip: impl Into<SharedString>,
    edge: GroupEdge,
    theme: &Theme,
) -> Stateful<Div> {
    group_item(id, tooltip, edge, theme).child(icons::icon(
        glyph_str,
        TOOLBAR_ICON_SIZE,
        theme.fg_dim,
    ))
}

/// Supplies the shared background for `group_item`s and rounds only the outer ends.
pub fn button_group(theme: &Theme, children: Vec<AnyElement>) -> Div {
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
