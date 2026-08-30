use gpui::{
    Anchor, AnyElement, Entity, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Pixels, Point, SharedString, Styled, anchored, deferred, div, px, rgb,
};
use jayjay_core::BookmarkInfo;

use super::BookmarkManagerView;
use crate::app::theme::Theme;
use crate::repo::revset;
use crate::ui::icons::glyph;
use crate::ui::primitives::icon_label;

#[derive(Clone)]
pub(super) enum BookmarkContextAction {
    Reveal(String),
    ShowDiff(BookmarkInfo),
    Track { name: String, remote: String },
    Push(String),
    Resolve(String),
    OpenPullRequest(String),
    Rename(String),
    Delete(String),
    Forget(String),
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

pub(super) fn bookmark_menu_items(
    bookmark: &BookmarkInfo,
    pr_host_name: Option<&str>,
) -> Vec<BookmarkContextMenuItem> {
    let mut items = Vec::new();
    if !bookmark.is_deleted && !bookmark.change_id.is_empty() {
        items.push(BookmarkContextMenuItem::new(
            "Reveal",
            glyph::ARROW_CIRCLE_RIGHT,
            BookmarkContextAction::Reveal(bookmark.change_id.id.clone()),
        ));
    }
    if !bookmark.is_deleted
        && !bookmark.is_conflicted
        && !bookmark.change_id.is_empty()
        && !revset::is_trunk_bookmark(&bookmark.name)
    {
        items.push(BookmarkContextMenuItem::new(
            "Diff",
            glyph::ARROWS_LEFT_RIGHT,
            BookmarkContextAction::ShowDiff(bookmark.clone()),
        ));
    }
    if bookmark.is_conflicted {
        items.push(BookmarkContextMenuItem::new(
            "Resolve conflict (set to @)",
            glyph::GIT_MERGE,
            BookmarkContextAction::Resolve(bookmark.name.clone()),
        ));
    }
    if !bookmark.is_deleted && !bookmark.has_local_target {
        for remote in &bookmark.available_remotes {
            items.push(BookmarkContextMenuItem::new(
                format!("Track {}@{remote}", bookmark.name),
                glyph::GIT_BRANCH,
                BookmarkContextAction::Track {
                    name: bookmark.name.clone(),
                    remote: remote.clone(),
                },
            ));
        }
    } else if bookmark.is_tracking_remote && !bookmark.is_deleted {
        items.push(BookmarkContextMenuItem::new(
            "Push",
            glyph::ARROW_UP,
            BookmarkContextAction::Push(bookmark.name.clone()),
        ));
    }
    if bookmark.is_tracking_remote
        && !bookmark.is_deleted
        && !revset::is_trunk_bookmark(&bookmark.name)
    {
        items.push(BookmarkContextMenuItem::new(
            pr_host_name
                .map(|host| format!("Pull Request on {host}"))
                .unwrap_or_else(|| "Pull Request".to_owned()),
            glyph::EXTERNAL_LINK,
            BookmarkContextAction::OpenPullRequest(bookmark.name.clone()),
        ));
    }
    if bookmark.is_deleted {
        items.push(BookmarkContextMenuItem::new(
            "Forget (clean up)",
            glyph::BOOKMARK,
            BookmarkContextAction::Forget(bookmark.name.clone()),
        ));
    } else if bookmark.has_local_target {
        items.push(BookmarkContextMenuItem::new(
            "Rename",
            glyph::PENCIL_CIRCLE,
            BookmarkContextAction::Rename(bookmark.name.clone()),
        ));
        items.push(BookmarkContextMenuItem::new(
            "Delete",
            glyph::X_CIRCLE,
            BookmarkContextAction::Delete(bookmark.name.clone()),
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
    let selector = format!("bookmark-context-{}", item.label);

    div()
        .id(("bookmark-context-menu-row", ix))
        .debug_selector(move || selector.clone())
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

#[cfg(test)]
mod tests {
    use jayjay_core::{BookmarkInfo, ShortId};

    use super::bookmark_menu_items;

    fn bookmark(name: &str) -> BookmarkInfo {
        BookmarkInfo {
            name: name.to_owned(),
            change_id: ShortId::new("abcdefghijkl".to_owned(), 3),
            description: String::new(),
            is_tracking_remote: true,
            is_deleted: false,
            is_conflicted: false,
            tracked_remotes: vec!["origin".to_owned()],
            available_remotes: vec!["origin".to_owned()],
            has_local_target: true,
            remote_targets: Vec::new(),
        }
    }

    #[test]
    fn deleted_bookmark_offers_only_cleanup() {
        let mut bookmark = bookmark("stale");
        bookmark.is_deleted = true;

        let labels: Vec<_> = bookmark_menu_items(&bookmark, Some("GitHub"))
            .into_iter()
            .map(|item| item.label.to_string())
            .collect();

        assert_eq!(labels, ["Forget (clean up)"]);
    }

    #[test]
    fn tracked_bookmark_matches_swiftui_actions() {
        let labels: Vec<_> = bookmark_menu_items(&bookmark("feature"), Some("GitHub"))
            .into_iter()
            .map(|item| item.label.to_string())
            .collect();

        assert_eq!(
            labels,
            [
                "Reveal",
                "Diff",
                "Push",
                "Pull Request on GitHub",
                "Rename",
                "Delete",
            ]
        );
    }
}
