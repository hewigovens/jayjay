use gpui::ScrollStrategy;
use gpui::{
    AnyElement, Context, Entity, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, MouseDownEvent, ParentElement, Pixels, Point, SharedString, Styled, div, px, rgb,
};
use jayjay_core::{BookmarkFilterTarget, BookmarkInfo};

mod entry;
mod rows;

use super::RepoWindow;
use super::picker::{self, PickerOutcome, PickerQuery, picker_actions, picker_items};
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::{self, glyph};
use crate::ui::input::LineInput;
use rows::{bookmark_row, bookmark_sections};

pub(crate) struct BookmarkPickerState {
    pub(super) anchor: Point<Pixels>,
    pub(super) query: PickerQuery,
}

impl RepoWindow {
    pub(crate) fn open_bookmark_picker(&mut self, anchor: Point<Pixels>, cx: &mut Context<Self>) {
        #[cfg(not(target_os = "macos"))]
        {
            self.app_menu = None;
        }
        self.context_menu = None;
        self.close_repo_switcher(cx);
        self.bookmark_picker = Some(BookmarkPickerState {
            anchor,
            query: PickerQuery::new(),
        });
        LineInput::show_for_owner(self, cx, Self::bookmark_picker_input);
        cx.notify();
    }

    fn bookmark_picker_query(view: &mut Self) -> Option<&mut PickerQuery> {
        view.bookmark_picker.as_mut().map(|state| &mut state.query)
    }

    fn bookmark_picker_input(view: &mut Self) -> Option<&mut LineInput> {
        Self::bookmark_picker_query(view).map(|query| &mut query.input)
    }

    pub(crate) fn close_bookmark_picker(&mut self, cx: &mut Context<Self>) {
        if self.bookmark_picker.is_some() {
            LineInput::hide_for_owner(self, cx, Self::bookmark_picker_input);
            self.bookmark_picker = None;
            cx.notify();
        }
    }

    pub(super) fn filter_bookmark_revset(&mut self, revset: &str, cx: &mut Context<Self>) {
        self.close_bookmark_picker(cx);
        self.apply_revset(revset, cx);
    }

    /// Selects the bookmark's exact commit where the graph shows it, so a divergent sibling is never picked; otherwise filters to its stack.
    pub(super) fn reveal_bookmark(
        &mut self,
        target: &BookmarkFilterTarget,
        cx: &mut Context<Self>,
    ) {
        self.close_bookmark_picker(cx);
        let vm = self.vm.read(cx);
        let commit_id = vm
            .repo
            .as_ref()
            .and_then(|repo| repo.log(&target.head).ok())
            .and_then(|changes| changes.into_iter().next())
            .map(|change| change.commit_id.id);
        let row = commit_id.as_ref().and_then(|commit_id| {
            vm.graph
                .changes
                .iter()
                .position(|change| &change.commit_id.id == commit_id)
        });
        match (row, commit_id) {
            (Some(ix), _) => {
                self.show_sidebar(cx);
                self.scrolls
                    .changes
                    .scroll_to_item(ix, ScrollStrategy::Center);
                self.select_change(ix, cx);
            }
            (None, selecting) => self.apply_revset_selecting(&target.revset, selecting, cx),
        }
    }

    pub(super) fn handle_bookmark_picker_key(
        &mut self,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(outcome) = self.drive_picker(
            event,
            Self::bookmark_picker_query,
            Self::bookmark_picker_input,
            |view, cx| view.bookmark_picker_actions(cx),
            cx,
        ) else {
            return false;
        };
        match outcome {
            PickerOutcome::Handled => {}
            PickerOutcome::Dismiss => self.close_bookmark_picker(cx),
            PickerOutcome::Activate(target) => self.reveal_bookmark(&target, cx),
        }
        true
    }

    fn bookmark_picker_query_edited(&mut self, cx: &mut Context<Self>) {
        self.picker_query_edited(
            Self::bookmark_picker_query,
            |view, cx| view.bookmark_picker_actions(cx),
            cx,
        );
    }

    fn bookmark_picker_actions(&self, cx: &gpui::App) -> Vec<(BookmarkFilterTarget, usize)> {
        let Some(state) = self.bookmark_picker.as_ref() else {
            return Vec::new();
        };
        let bookmarks = self.vm.read(cx).graph.bookmarks.clone();
        picker_actions(&bookmark_sections(state, &bookmarks))
    }
}

pub(crate) fn render_bookmark_picker(
    state: &BookmarkPickerState,
    bookmarks: &[BookmarkInfo],
    t: &Theme,
    view: &Entity<RepoWindow>,
    ime_focus: Option<FocusHandle>,
) -> AnyElement {
    let close_view = view.clone();
    picker::overlay(
        "bookmark-picker-backdrop",
        state.anchor,
        menu_panel(state, bookmarks, t, view, ime_focus),
        move |_: &MouseDownEvent, _, cx| {
            close_view.update(cx, |view, cx| view.close_bookmark_picker(cx));
        },
    )
}

fn menu_panel(
    state: &BookmarkPickerState,
    bookmarks: &[BookmarkInfo],
    t: &Theme,
    view: &Entity<RepoWindow>,
    ime_focus: Option<FocusHandle>,
) -> AnyElement {
    let new_view = view.clone();
    let header = picker::header(
        "bookmark-picker-filter",
        &state.query,
        LineInput::ime_layer(
            view.clone(),
            ime_focus,
            RepoWindow::bookmark_picker_input,
            RepoWindow::bookmark_picker_query_edited,
        ),
        [picker::header_button(
            "bookmark-picker-new",
            glyph::PLUS_CIRCLE,
            "New",
            t,
            move |_, cx| {
                new_view.update(cx, |view, cx| {
                    view.close_bookmark_picker(cx);
                    view.open_create_bookmark("@".to_owned(), cx);
                });
            },
        )],
        t,
    );

    let has_any_bookmarks = bookmarks.iter().any(|bookmark| !bookmark.is_deleted);
    let row_view = view.clone();
    picker::panel(
        "bookmark-picker-panel",
        280.,
        header,
        picker_items(
            bookmark_sections(state, bookmarks),
            state.query.selected,
            if has_any_bookmarks {
                "No matches"
            } else {
                "No bookmarks yet"
            },
        ),
        &state.query,
        t,
        move |entry, selected, t| bookmark_row(entry, selected, t, &row_view),
    )
}

/// Bookmarks the picker lists: live ones, plus deleted ones still on a remote the user does not track.
pub(crate) fn listed_bookmark_count(bookmarks: &[BookmarkInfo]) -> usize {
    bookmarks
        .iter()
        .filter(|bookmark| {
            !bookmark.is_deleted
                || bookmark
                    .available_remotes
                    .iter()
                    .any(|remote| !bookmark.tracked_remotes.contains(remote))
        })
        .count()
}

pub(crate) fn bookmarks_header_button(
    count: usize,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let button = div()
        .id(SharedString::from("bookmarks-button"))
        .debug_selector(move || format!("bookmarks-button-{count}"))
        .flex()
        .flex_none()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .h(px(t.scaled_control_height(26., 12.)))
        .px(px(6.))
        .rounded_md()
        .text_size(ui_font_size(12.))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(rgb(t.fg))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|view, ev: &MouseDownEvent, window, cx| {
                view.focus_handle.focus(window, cx);
                view.open_bookmark_picker(ev.position, cx);
            }),
        )
        .child(icons::icon(glyph::BOOKMARK, 12., t.fg))
        .child("Bookmarks");
    let mut button = picker::opener(
        button,
        |view| view.bookmark_picker.is_some(),
        RepoWindow::close_bookmark_picker,
        cx,
    );
    if count > 0 {
        button = button.child(
            div()
                .px(px(5.))
                .rounded_full()
                .bg(rgb(t.toggle_inactive_bg))
                .text_size(ui_font_size(11.))
                .font_weight(gpui::FontWeight::NORMAL)
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(count.to_string())),
        );
    }
    button
        .child(icons::icon(glyph::CARET_DOWN, 10., t.fg_dim))
        .into_any_element()
}
