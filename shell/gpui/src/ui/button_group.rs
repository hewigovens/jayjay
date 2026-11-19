use gpui::{
    AnyElement, Div, InteractiveElement, MouseButton, ParentElement, SharedString, Stateful,
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

/// No background or border of its own at rest — `button_group` supplies the shared fill; hover and press light a pill inset in this segment.
pub(crate) fn group_item(
    id: impl Into<SharedString>,
    tooltip: impl Into<SharedString>,
    edge: GroupEdge,
    theme: &Theme,
) -> Stateful<Div> {
    let id = id.into();
    let hover = theme.toolbar_hover_bg;
    let pressed = theme.selected_bg;
    let el = div()
        .id(id.clone())
        .group(id.clone())
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .w(px(TOOLBAR_BUTTON_WIDTH))
        .h_full()
        .cursor_pointer()
        .child(
            div()
                .id("hover")
                .absolute()
                .inset(px(3.))
                .rounded_full()
                .group_hover(id.clone(), move |s| s.bg(rgb(hover)))
                .group_active(id, move |s| s.bg(rgb(pressed))),
        )
        .help(tooltip.into())
        // Keeps a double click from zooming the window and, like a native toolbar button, leaves focus put.
        .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default());
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
