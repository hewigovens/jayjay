//! Right-click context menu, overlaid via `gpui::deferred` + `gpui::anchored` with a transparent backdrop that dismisses it on outside clicks.

use std::sync::Arc;

use gpui::{
    Anchor, AnyElement, Entity, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Pixels, Point, SharedString, Styled, anchored, deferred, div, px, rgb,
};

use crate::app::theme::Theme;
use crate::repo::revset::BookmarkDiffRequest;
use crate::repo::window::{
    AbandonSelectedLinesRequest, AddNoteRequest, ChangeAction, FileBatchAction, RepoWindow,
};
use crate::ui::primitives::icon_label;

#[derive(Clone)]
pub enum ContextAction {
    Noop,
    CopyText(SharedString),
    OpenUrl(SharedString),
    CreateBookmark(SharedString),
    OpenStackedPr(SharedString),
    MoveBookmark {
        name: SharedString,
        to_rev: SharedString,
    },
    PushBookmark(SharedString),
    DeleteBookmark {
        name: SharedString,
        rev: Option<SharedString>,
    },
    OpenPRForBookmark(SharedString),
    NewChangeOnTop(SharedString),
    Change(Arc<ChangeAction>),
    AbandonChange(SharedString),
    OpenEvologFor(SharedString),
    OpenFileHistoryFor(SharedString),
    ToggleAnnotateFor(SharedString),
    ShowBookmarkDiff(BookmarkDiffRequest),
    FilterByBookmark(SharedString),
    RevealChange(SharedString),
    OpenInEditor(SharedString),
    ShowInFileManager(SharedString),
    #[allow(unused)]
    OpenInTerminal,
    OpenWorkspaceAt(SharedString),
    ForgetWorkspace(SharedString),
    CreateWorkspace,
    OpenDiffEdit,
    AbandonSelectedLines(Arc<AbandonSelectedLinesRequest>),
    FileBatch(Arc<FileBatchAction>),
    OpenAddReviewNote(Arc<AddNoteRequest>),
    OpenEditReviewNote(SharedString),
    ResolveReviewNote(SharedString),
    DeleteReviewNote(SharedString),
}

#[derive(Clone)]
pub struct ContextMenuItem {
    pub label: SharedString,
    glyph: &'static str,
    pub action: ContextAction,
}

impl ContextMenuItem {
    pub(crate) fn new(
        label: impl Into<SharedString>,
        glyph: &'static str,
        action: ContextAction,
    ) -> Self {
        Self {
            label: label.into(),
            glyph,
            action,
        }
    }
}

#[derive(Clone)]
pub struct ContextMenuState {
    pub(crate) anchor: Point<Pixels>,
    pub(crate) items: Vec<ContextMenuItem>,
}

pub(crate) fn render_context_menu(
    state: &ContextMenuState,
    t: &Theme,
    view: &Entity<RepoWindow>,
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
                cx.stop_propagation();
                v.update(cx, |this, cx| this.close_context_menu(cx));
            }
        })
        .on_mouse_down(MouseButton::Right, {
            let v = backdrop_view;
            move |_: &MouseDownEvent, _, cx| {
                cx.stop_propagation();
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
    .with_priority(4)
    .into_any_element()
}

fn menu_panel(items: &[ContextMenuItem], t: &Theme, view: &Entity<RepoWindow>) -> AnyElement {
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

fn menu_row(ix: usize, item: &ContextMenuItem, t: &Theme, view: &Entity<RepoWindow>) -> AnyElement {
    let action = item.action.clone();
    let view = view.clone();
    let selector = format!("context-menu-{}", item.label);

    div()
        .id(("context-menu-row", ix))
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
            // Stop propagation so the click doesn't also select the row sitting under the menu.
            cx.stop_propagation();
            let action = action.clone();
            view.update(cx, |this, cx| {
                this.dispatch_context_action(action, cx);
            });
        })
        .child(icon_label(item.glyph, item.label.clone(), 12., t.fg_dim))
        .into_any_element()
}
