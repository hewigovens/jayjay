//! Right-click context menu, overlaid via `gpui::deferred` + `gpui::anchored` with a transparent backdrop that dismisses it on outside clicks.

use std::sync::Arc;

use gpui::{
    Anchor, AnyElement, Entity, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Pixels, Point, SharedString, StatefulInteractiveElement, Styled, anchored,
    deferred, div, point, px, rgb,
};

use crate::app::theme::Theme;
use crate::repo::revset::BookmarkDiffRequest;
use crate::repo::window::{
    AbandonSelectedLinesRequest, AddNoteRequest, ChangeAction, FileBatchAction, RepoWindow,
};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::icon_label;

const MENU_MIN_WIDTH: f32 = 260.;
const MENU_MAX_WIDTH: f32 = 420.;
const MENU_ROW_HEIGHT: f32 = 28.;
const MENU_SEPARATOR_HEIGHT: f32 = 9.;

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
    ForgetWorkspace {
        name: SharedString,
        path: Option<SharedString>,
    },
    DeleteWorkspace {
        name: SharedString,
        path: SharedString,
    },
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
    is_separator: bool,
    pub enabled: bool,
    submenu: Option<Vec<ContextMenuItem>>,
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
            is_separator: false,
            enabled: true,
            submenu: None,
        }
    }

    pub(crate) fn submenu(
        label: impl Into<SharedString>,
        glyph: &'static str,
        items: Vec<ContextMenuItem>,
    ) -> Self {
        Self {
            label: label.into(),
            glyph,
            action: ContextAction::Noop,
            is_separator: false,
            enabled: true,
            submenu: Some(items),
        }
    }

    pub(crate) fn separator() -> Self {
        Self {
            label: SharedString::default(),
            glyph: "",
            action: ContextAction::Noop,
            is_separator: true,
            enabled: true,
            submenu: None,
        }
    }

    pub(crate) fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn submenu_items(&self) -> Option<&[ContextMenuItem]> {
        self.submenu.as_deref()
    }
}

#[derive(Clone)]
pub struct ContextMenuState {
    pub(crate) anchor: Point<Pixels>,
    pub(crate) items: Vec<ContextMenuItem>,
    pub(crate) submenu_index: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MenuLevel {
    Main,
    Submenu,
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
        .child(menu_panel(&state.items, MenuLevel::Main, t, view));

    let submenu = state.submenu_index.and_then(|index| {
        let items = state.items.get(index)?.submenu_items()?;
        Some(
            anchored()
                .anchor(Anchor::TopLeft)
                .position(submenu_position(state, index))
                .snap_to_window_with_margin(px(6.))
                .child(menu_panel(items, MenuLevel::Submenu, t, view)),
        )
    });

    let mut overlay = div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .child(backdrop)
        .child(menu);
    if let Some(submenu) = submenu {
        overlay = overlay.child(submenu);
    }
    deferred(overlay).with_priority(4).into_any_element()
}

fn submenu_position(state: &ContextMenuState, index: usize) -> Point<Pixels> {
    let y = state.items.iter().take(index).fold(px(4.), |offset, item| {
        offset
            + if item.is_separator {
                px(MENU_SEPARATOR_HEIGHT)
            } else {
                px(MENU_ROW_HEIGHT)
            }
    });
    point(
        state.anchor.x + menu_width(&state.items) - px(2.),
        state.anchor.y + y,
    )
}

fn menu_width(items: &[ContextMenuItem]) -> Pixels {
    let longest_label = items
        .iter()
        .map(|item| item.label.chars().count())
        .max()
        .unwrap_or_default() as f32;
    px((54. + longest_label * 7.).clamp(MENU_MIN_WIDTH, MENU_MAX_WIDTH))
}

fn menu_panel(
    items: &[ContextMenuItem],
    level: MenuLevel,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let mut col = div()
        .flex()
        .flex_col()
        .w(menu_width(items))
        .py(px(4.))
        .bg(rgb(t.detail_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_sm();

    for (ix, item) in items.iter().enumerate() {
        if item.is_separator {
            col = col.child(div().h(px(1.)).my(px(4.)).bg(rgb(t.border)));
        } else {
            col = col.child(menu_row(ix, item, level, t, view));
        }
    }
    col.into_any_element()
}

fn menu_row(
    ix: usize,
    item: &ContextMenuItem,
    level: MenuLevel,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let action = item.action.clone();
    let action_view = view.clone();
    let selector = format!("context-menu-{}", item.label);
    let has_submenu = item.submenu.is_some();

    let row = div()
        .id((
            if level == MenuLevel::Main {
                "context-menu-row"
            } else {
                "context-submenu-row"
            },
            ix,
        ))
        .debug_selector(move || selector.clone())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.))
        .px(px(12.))
        .py(px(6.))
        .text_size(px(13.))
        .text_color(rgb(t.fg));
    let row = if level == MenuLevel::Main {
        let hover_view = view.clone();
        row.on_hover(move |hovered, _, cx| {
            if *hovered {
                hover_view.update(cx, |this, cx| {
                    this.set_context_submenu(has_submenu.then_some(ix), cx);
                });
            }
        })
    } else {
        row
    };
    let row = if item.enabled {
        row.cursor_pointer()
            .hover(|s| s.bg(rgb(t.selected_bg)))
            .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _, cx| {
                cx.stop_propagation();
                action_view.update(cx, |this, cx| {
                    if has_submenu && level == MenuLevel::Main {
                        this.set_context_submenu(Some(ix), cx);
                    } else {
                        this.dispatch_context_action(action.clone(), cx);
                    }
                });
            })
    } else {
        row.opacity(0.45)
    };
    let mut row = row.child(icon_label(item.glyph, item.label.clone(), 13., t.fg_dim));
    if has_submenu {
        row = row
            .child(div().flex_1())
            .child(icons::icon(glyph::CARET_RIGHT, 10., t.fg_dim));
    }
    row.into_any_element()
}
