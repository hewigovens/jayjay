//! Bookmark drag-and-drop: drag a bookmark chip onto another DAG change to
//! move the bookmark there. Mirrors the SwiftUI `DAGView+RebaseDrag` bookmark
//! drag flow (chip → drop on row → `move_bookmark(name, dest_rev)`).

use gpui::{Context, IntoElement, Render, Styled, Window, rgb};

use super::RepoWindow;
use crate::app::theme::{FONT_TAG, theme};
use crate::ui::icons::glyph;
use crate::ui::primitives::icon_chip;

/// Payload carried by an in-flight bookmark drag. `'static` so GPUI can box it.
#[derive(Clone)]
pub(super) struct BookmarkDrag {
    pub(super) name: String,
}

/// The floating chip rendered under the cursor while dragging a bookmark.
pub(super) struct BookmarkDragGhost {
    name: String,
}

impl BookmarkDragGhost {
    pub(super) fn new(name: String) -> Self {
        Self { name }
    }
}

impl Render for BookmarkDragGhost {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx);
        icon_chip(
            glyph::BOOKMARK,
            self.name.clone(),
            t.tag_bookmark_bg,
            t.tag_bookmark_fg,
            t.tag_bookmark_icon,
            FONT_TAG,
        )
        .border_1()
        .border_color(rgb(t.toggle_active_bg))
        .opacity(0.9)
    }
}

impl RepoWindow {
    /// Drop handler: move `name` onto the change at `rev`. No-op when the bookmark
    /// already points only there (a self-drop). Conflicted bookmarks are listed on
    /// every target, so dropping onto one of those commits is a resolve, not a no-op.
    pub(super) fn drop_bookmark_on_rev(
        &mut self,
        name: String,
        rev: String,
        cx: &mut Context<Self>,
    ) {
        let already_here = !self.bookmark_is_conflicted(&name, cx)
            && self.vm.read(cx).graph.changes.iter().any(|c| {
                crate::repo::revset::change_revision(c) == rev
                    && c.bookmarks.iter().any(|b| b.as_str() == name)
            });
        if already_here {
            return;
        }
        let task = self
            .vm
            .update(cx, |vm, cx| vm.move_bookmark(name.clone(), rev, cx));
        cx.spawn(async move |this, cx| {
            if task.await.is_ok() {
                let _ = this.update(cx, move |view, cx| {
                    view.show_toast(format!("Moved {name}"), cx);
                });
            }
        })
        .detach();
    }
}
