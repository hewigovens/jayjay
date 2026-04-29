//! Right-click context menu primitive.
//!
//! `LogView` owns an `Option<ContextMenuState>` and renders the menu via
//! [`render_context_menu`]. The menu is overlaid on top of the rest of the
//! window using [`gpui::deferred`] + [`gpui::anchored`]. A full-window
//! transparent backdrop catches clicks outside the menu and dismisses it.

use gpui::{
    Anchor, AnyElement, Entity, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Pixels, Point, SharedString, Styled, anchored, deferred, div, px, rgb,
};

use crate::app::theme::Theme;
use crate::log::LogView;
use crate::ui::icons;

#[derive(Clone)]
pub enum ContextAction {
    CopyText(SharedString),
    OpenUrl(SharedString),
    OpenEvologFor(SharedString),
    OpenFileHistoryFor(SharedString),
    ToggleAnnotateFor(SharedString),
    RevealChange(SharedString),
    OpenInEditor(SharedString),
    #[allow(unused)]
    OpenInTerminal,
}

#[derive(Clone)]
pub struct ContextMenuItem {
    pub label: SharedString,
    pub glyph: &'static str,
    pub action: ContextAction,
}

impl ContextMenuItem {
    pub fn new(label: impl Into<SharedString>, glyph: &'static str, action: ContextAction) -> Self {
        Self {
            label: label.into(),
            glyph,
            action,
        }
    }
}

#[derive(Clone)]
pub struct ContextMenuState {
    pub anchor: Point<Pixels>,
    pub items: Vec<ContextMenuItem>,
}

pub fn render_context_menu(
    state: &ContextMenuState,
    t: &Theme,
    view: &Entity<LogView>,
) -> AnyElement {
    let backdrop_view = view.clone();
    let backdrop = div()
        .id("context-menu-backdrop")
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .on_mouse_down(MouseButton::Left, {
            let v = backdrop_view.clone();
            move |_: &MouseDownEvent, _, cx| {
                v.update(cx, |this, cx| this.close_context_menu(cx));
            }
        })
        .on_mouse_down(MouseButton::Right, {
            let v = backdrop_view;
            move |_: &MouseDownEvent, _, cx| {
                v.update(cx, |this, cx| this.close_context_menu(cx));
            }
        });

    let menu = anchored()
        .anchor(Anchor::TopLeft)
        .position(state.anchor)
        .snap_to_window_with_margin(px(6.))
        .child(menu_panel(&state.items, t, view));

    deferred(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .child(backdrop)
            .child(menu),
    )
    .with_priority(2)
    .into_any_element()
}

fn menu_panel(items: &[ContextMenuItem], t: &Theme, view: &Entity<LogView>) -> AnyElement {
    let mut col = div()
        .flex()
        .flex_col()
        .min_w(px(180.))
        .py(px(4.))
        .bg(rgb(t.detail_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_sm();

    for (ix, item) in items.iter().enumerate() {
        col = col.child(menu_row(ix, item, t, view));
    }
    col.into_any_element()
}

fn menu_row(ix: usize, item: &ContextMenuItem, t: &Theme, view: &Entity<LogView>) -> AnyElement {
    let action = item.action.clone();
    let view = view.clone();

    div()
        .id(("context-menu-row", ix))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(10.))
        .py(px(5.))
        .text_size(px(12.))
        .text_color(rgb(t.fg))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.selected_bg)))
        .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _, cx| {
            // Stop the event before it bubbles to the file/change row sitting
            // visually under the menu — otherwise picking the second menu item
            // would also select whichever row is beneath that item.
            cx.stop_propagation();
            let action = action.clone();
            view.update(cx, |this, cx| {
                this.dispatch_context_action(action, cx);
            });
        })
        .child(icons::icon(item.glyph, 12., t.fg_dim))
        .child(item.label.clone())
        .into_any_element()
}
