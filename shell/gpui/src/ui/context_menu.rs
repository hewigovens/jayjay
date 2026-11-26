//! Right-click context menu, overlaid via `gpui::deferred` + `gpui::anchored` with a transparent backdrop that dismisses it on outside clicks.

mod keyboard;

use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::{
    Anchor, AnyElement, App, Entity, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Pixels, Point, SharedString, StatefulInteractiveElement, Styled, anchored,
    canvas, deferred, div, point, px, rgb,
};
use jayjay_core::compare::{BookmarkDiffRequest, CompareState};

use crate::app::fonts::ui_text_width;
use crate::app::theme::{Theme, ui_size};
use crate::repo::window::{
    AbandonSelectedLinesRequest, AddNoteRequest, ChangeAction, ConflictResolution, FileBatchAction,
    RepoWindow,
};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::HelpTag;

const MENU_MIN_WIDTH: f32 = 140.;
const MENU_MAX_WIDTH: f32 = 420.;
const MENU_ROW_HEIGHT: f32 = 28.;
const MENU_SEPARATOR_HEIGHT: f32 = 9.;
const MENU_EDGE_MARGIN: f32 = 6.;
const MENU_FONT: f32 = 13.;
const MENU_ICON_SIZE: f32 = 13.;
const MENU_CARET_SIZE: f32 = 10.;
const MENU_ROW_GAP: f32 = 6.;
const MENU_ROW_PADDING_X: f32 = 12.;

#[derive(Clone)]
pub enum ContextAction {
    Noop,
    CopyText(SharedString),
    OpenUrl(SharedString),
    CreateBookmark(SharedString),
    CreateTag(SharedString),
    PushTag(SharedString),
    DeleteTag(SharedString),
    DeleteRemoteTag(SharedString),
    OpenStackedPr(SharedString),
    ResolveDivergence(SharedString),
    UploadChange(SharedString),
    UpdateFromGerrit(SharedString),
    MoveBookmark {
        name: SharedString,
        to_rev: SharedString,
    },
    PushBookmark(SharedString),
    PullBookmark(SharedString),
    RenameBookmark(SharedString),
    DeleteBookmark {
        name: SharedString,
        rev: Option<SharedString>,
    },
    OpenPRForBookmark(SharedString),
    NewChangeOnTop(SharedString),
    Change(Arc<ChangeAction>),
    AbandonChange(SharedString),
    EditDescription {
        rev: SharedString,
        description: SharedString,
    },
    OpenEvologFor(SharedString),
    ShowAncestors(SharedString),
    RestoreOperation(SharedString),
    UpdateWorkspace,
    CommitWorkingCopy(SharedString),
    OpenFileHistoryFor(SharedString),
    ToggleAnnotateFor(SharedString),
    ShowBookmarkDiff(BookmarkDiffRequest),
    ShowCompare(CompareState),
    FilterRevset(SharedString),
    TrackBookmark {
        name: String,
        remote: String,
    },
    RevealChange(SharedString),
    OpenInEditor {
        path: SharedString,
        line: Option<u32>,
    },
    ShowInFileManager(SharedString),
    OpenWorkspaceAt(SharedString),
    SetRepositoryPinned {
        path: SharedString,
        pinned: bool,
    },
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
    EditFileAtLine(u32),
    AbandonSelectedLines(Arc<AbandonSelectedLinesRequest>),
    FileBatch(Arc<FileBatchAction>),
    OpenAddReviewNote(Arc<AddNoteRequest>),
    OpenEditReviewNote(SharedString),
    ResolveReviewNote(SharedString),
    DeleteReviewNote(SharedString),
    ResolveConflict(ConflictResolution),
}

#[derive(Clone)]
pub struct ContextMenuItem {
    pub label: SharedString,
    glyph: &'static str,
    pub action: ContextAction,
    is_separator: bool,
    pub enabled: bool,
    tooltip: Option<SharedString>,
    submenu: Option<Vec<ContextMenuItem>>,
    shortcut: Option<char>,
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
            shortcut: action.shortcut(),
            action,
            is_separator: false,
            enabled: true,
            tooltip: None,
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
            shortcut: None,
            action: ContextAction::Noop,
            is_separator: false,
            enabled: true,
            tooltip: None,
            submenu: Some(items),
        }
    }

    pub(crate) fn separator() -> Self {
        Self {
            label: SharedString::default(),
            glyph: "",
            shortcut: None,
            action: ContextAction::Noop,
            is_separator: true,
            enabled: true,
            tooltip: None,
            submenu: None,
        }
    }

    pub(crate) fn with_shortcut(mut self, shortcut: char) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub(crate) fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub(crate) fn with_disabled_reason(mut self, reason: Option<SharedString>) -> Self {
        self.enabled &= reason.is_none();
        self.tooltip = reason;
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
    pub(crate) selected_index: Option<usize>,
    pub(crate) submenu_selected_index: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum MenuLevel {
    Main,
    Submenu,
}

pub(crate) fn render_context_menu(
    state: &ContextMenuState,
    viewport_width: Pixels,
    t: &Theme,
    view: &Entity<RepoWindow>,
    cx: &App,
) -> AnyElement {
    let width = menu_width(&state.items, t, cx);
    let margin = px(t.scaled_font_size(MENU_EDGE_MARGIN));
    let origin = menu_origin(state.anchor, width, viewport_width, margin);
    let backdrop_view = view.clone();
    let backdrop = div()
        .id("context-menu-backdrop")
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .occlude()
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
        .position(origin)
        .snap_to_window_with_margin(margin)
        .child(menu_panel(
            &state.items,
            width,
            MenuLevel::Main,
            state.selected_index,
            t,
            view,
        ));

    let submenu = state.submenu_index.and_then(|index| {
        let items = state.items.get(index)?.submenu_items()?;
        let submenu_width = menu_width(items, t, cx);
        let x = if origin.x < state.anchor.x {
            origin.x - submenu_width + px(2.)
        } else {
            origin.x + width - px(2.)
        };
        Some(
            anchored()
                .anchor(Anchor::TopLeft)
                .position(point(x, origin.y + submenu_offset(state, index, t)))
                .snap_to_window_with_margin(margin)
                .child(menu_panel(
                    items,
                    submenu_width,
                    MenuLevel::Submenu,
                    state.submenu_selected_index,
                    t,
                    view,
                )),
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

/// Opens leftward from the pointer when the menu would cross the trailing edge, as native menus do.
fn menu_origin(
    anchor: Point<Pixels>,
    width: Pixels,
    viewport_width: Pixels,
    margin: Pixels,
) -> Point<Pixels> {
    if anchor.x + width + margin > viewport_width && anchor.x - width >= margin {
        point(anchor.x - width, anchor.y)
    } else {
        anchor
    }
}

fn submenu_offset(state: &ContextMenuState, index: usize, t: &Theme) -> Pixels {
    state
        .items
        .iter()
        .take(index)
        .fold(px(t.scaled_font_size(4.)), |offset, item| {
            offset
                + if item.is_separator {
                    px(t.scaled_font_size(MENU_SEPARATOR_HEIGHT - 1.) + 1.)
                } else {
                    px(t.scaled_font_size(MENU_ROW_HEIGHT))
                }
        })
}

/// Fits the longest label, the shortcut letters and the submenu caret at the current zoom.
fn menu_width(items: &[ContextMenuItem], t: &Theme, cx: &App) -> Pixels {
    let size = px(t.scaled_font_size(MENU_FONT));
    let label = items
        .iter()
        .map(|item| ui_text_width(cx, &item.label, size))
        .fold(px(0.), Pixels::max);
    let shortcut = items
        .iter()
        .filter_map(|item| item.shortcut)
        .map(|shortcut| {
            ui_text_width(cx, &shortcut.to_ascii_uppercase().to_string(), size)
                + px(t.scaled_font_size(MENU_ROW_GAP))
        })
        .fold(px(0.), Pixels::max);
    let caret = if items.iter().any(|item| item.submenu.is_some()) {
        px(t.scaled_font_size(MENU_ROW_GAP + MENU_CARET_SIZE))
    } else {
        px(0.)
    };
    let chrome =
        px(t.scaled_font_size(2. * MENU_ROW_PADDING_X + MENU_ICON_SIZE + MENU_ROW_GAP) + 2.);
    let scale = t.scaled_font_size(1.);
    (chrome + label + shortcut + caret)
        .clamp(px(MENU_MIN_WIDTH * scale), px(MENU_MAX_WIDTH * scale))
}

fn menu_panel(
    items: &[ContextMenuItem],
    width: Pixels,
    level: MenuLevel,
    selected: Option<usize>,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let mut col = div()
        .flex()
        .flex_col()
        .w(width)
        .py(ui_size(4.))
        .bg(rgb(t.detail_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_sm();

    for (ix, item) in items.iter().enumerate() {
        if item.is_separator {
            col = col.child(div().h(px(1.)).my(ui_size(4.)).bg(rgb(t.border)));
        } else {
            col = col.child(menu_row(ix, item, level, selected == Some(ix), t, view));
        }
    }
    col.into_any_element()
}

fn menu_row(
    ix: usize,
    item: &ContextMenuItem,
    level: MenuLevel,
    selected: bool,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let action = item.action.clone();
    let action_view = view.clone();
    let selector = format!("context-menu-{}", item.label);
    let has_submenu = item.submenu.is_some();
    let enabled = item.enabled;

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
        .gap(ui_size(MENU_ROW_GAP))
        .px(ui_size(MENU_ROW_PADDING_X))
        .py(ui_size(6.))
        .text_size(ui_size(MENU_FONT))
        .text_color(rgb(t.fg))
        .when(selected && enabled, |row| row.bg(rgb(t.selected_bg)));
    let hover_view = view.clone();
    let row = row.on_hover(move |hovered, _, cx| {
        if *hovered {
            hover_view.update(cx, |this, cx| {
                this.hover_context_item(level, ix, cx);
            });
        }
    });
    let row = if enabled {
        row.cursor_pointer()
            .hover(|s| s.bg(rgb(t.selected_bg)))
            .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _, cx| {
                cx.stop_propagation();
                action_view.update(cx, |this, cx| {
                    if has_submenu && level == MenuLevel::Main {
                        this.hover_context_item(level, ix, cx);
                    } else {
                        this.dispatch_context_action(action.clone(), cx);
                    }
                });
            })
    } else {
        row.opacity(0.45)
    };
    let mut row = row
        .when_some(item.tooltip.clone(), |row, tooltip| row.menu_help(tooltip))
        .child(icons::icon(item.glyph, MENU_ICON_SIZE, t.fg_dim))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .child(item.label.clone()),
        );
    if let Some(shortcut) = item.shortcut {
        row = row.child(
            div()
                .text_color(rgb(t.fg_dim))
                .child(shortcut.to_ascii_uppercase().to_string()),
        );
    }
    if has_submenu {
        row = row.child(icons::icon(glyph::CARET_RIGHT, MENU_CARET_SIZE, t.fg_dim));
    }
    row.into_any_element()
}

// Menu placement is read on key down, so capturing a row never needs another frame.
pub(crate) fn row_bounds(slot: crate::repo::window::PanelBoundsSlot) -> impl IntoElement {
    canvas(move |bounds, _, _| slot.set(Some(bounds)), |_, _, _, _| {})
        .absolute()
        .inset_0()
}
