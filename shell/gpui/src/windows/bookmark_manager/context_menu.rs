use gpui::{
    Anchor, AnyElement, Entity, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Pixels, Point, SharedString, Styled, anchored, deferred, div, px, rgb,
};
use jayjay_core::BookmarkInfo;

use super::BookmarkManagerView;
use crate::app::theme::Theme;
use crate::ui::icons::glyph;
use crate::ui::primitives::icon_label;

#[derive(Clone)]
pub(super) enum BookmarkContextAction {
    Reveal(String),
    ShowDiff(BookmarkInfo),
    CopyName(String),
    Track { name: String, remote: String },
}

#[derive(Clone)]
pub(super) struct BookmarkContextMenuItem {
    label: SharedString,
    glyph: &'static str,
    action: BookmarkContextAction,
}

#[derive(Clone)]
pub(super) struct BookmarkContextMenuState {
    pub anchor: Point<Pixels>,
    pub items: Vec<BookmarkContextMenuItem>,
}

pub(super) fn bookmark_menu_items(bookmark: &BookmarkInfo) -> Vec<BookmarkContextMenuItem> {
    let mut items = Vec::new();
    if !bookmark.change_id.is_empty() {
        items.push(BookmarkContextMenuItem::new(
            "Reveal",
            glyph::ARROW_CIRCLE_RIGHT,
            BookmarkContextAction::Reveal(bookmark.change_id.id.clone()),
        ));
        items.push(BookmarkContextMenuItem::new(
            "Diff",
            glyph::ARROWS_LEFT_RIGHT,
            BookmarkContextAction::ShowDiff(bookmark.clone()),
        ));
    }
    items.push(BookmarkContextMenuItem::new(
        "Copy Name",
        glyph::COPY,
        BookmarkContextAction::CopyName(bookmark.name.clone()),
    ));
    if !bookmark.is_tracking_remote
        && let Some(remote) = bookmark.available_remotes.first()
    {
        items.push(BookmarkContextMenuItem::new(
            format!("Track {remote}"),
            glyph::GIT_BRANCH,
            BookmarkContextAction::Track {
                name: bookmark.name.clone(),
                remote: remote.clone(),
            },
        ));
    }
    items
}

pub(super) fn render_context_menu(
    state: &BookmarkContextMenuState,
    t: &Theme,
    view: &Entity<BookmarkManagerView>,
) -> AnyElement {
    let backdrop_view = view.clone();
    let backdrop = div()
        .id("bookmark-menu-backdrop")
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .on_mouse_down(MouseButton::Left, {
            let view = backdrop_view.clone();
            move |_: &MouseDownEvent, _, cx| {
                view.update(cx, |this, cx| this.close_context_menu(cx));
            }
        })
        .on_mouse_down(MouseButton::Right, {
            let view = backdrop_view;
            move |_: &MouseDownEvent, _, cx| {
                view.update(cx, |this, cx| this.close_context_menu(cx));
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

impl BookmarkContextMenuItem {
    fn new(
        label: impl Into<SharedString>,
        glyph: &'static str,
        action: BookmarkContextAction,
    ) -> Self {
        Self {
            label: label.into(),
            glyph,
            action,
        }
    }
}

fn menu_panel(
    items: &[BookmarkContextMenuItem],
    t: &Theme,
    view: &Entity<BookmarkManagerView>,
) -> AnyElement {
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

fn menu_row(
    ix: usize,
    item: &BookmarkContextMenuItem,
    t: &Theme,
    view: &Entity<BookmarkManagerView>,
) -> AnyElement {
    let action = item.action.clone();
    let view = view.clone();

    div()
        .id(("bookmark-context-menu-row", ix))
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
            cx.stop_propagation();
            let action = action.clone();
            view.update(cx, |this, cx| {
                this.dispatch_context_action(action, cx);
            });
        })
        .child(icon_label(item.glyph, item.label.clone(), 12., t.fg_dim))
        .into_any_element()
}
