use gpui::{AnyElement, Context, Entity, KeyDownEvent, MouseDownEvent, Pixels, Point};
use jayjay_core::BookmarkInfo;

mod rows;

use super::RepoWindow;
use super::picker::{self, PickerOutcome, PickerQuery, picker_actions, render_sections};
use crate::app::theme::Theme;
use crate::repo::revset;
use crate::ui::icons::glyph;
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

    pub(super) fn filter_by_bookmark(&mut self, name: &str, cx: &mut Context<Self>) {
        self.close_bookmark_picker(cx);
        let symbol = revset::quoted_symbol(name);
        let revset = if self.bookmark_is_conflicted(name, cx) {
            format!("bookmarks(exact:{symbol})")
        } else {
            symbol
        };
        if let Some(input) = self.revset_filter.as_mut() {
            input.set_text(revset.clone());
        }
        self.vm.update(cx, |vm, cx| vm.apply_revset(&revset, cx));
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
            PickerOutcome::Activate(name) => self.filter_by_bookmark(&name, cx),
        }
        true
    }

    fn bookmark_picker_actions(&self, cx: &gpui::App) -> Vec<(String, usize)> {
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
) -> AnyElement {
    let close_view = view.clone();
    picker::overlay(
        "bookmark-picker-backdrop",
        state.anchor,
        menu_panel(state, bookmarks, t, view),
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
) -> AnyElement {
    let new_view = view.clone();
    let header = picker::header(
        "bookmark-picker-filter",
        &state.query,
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

    let sections = bookmark_sections(state, bookmarks);
    let has_any_bookmarks = bookmarks
        .iter()
        .any(|bookmark| !bookmark.is_deleted && bookmark.has_local_target);
    let rows = if sections.is_empty() {
        vec![picker::empty(
            if has_any_bookmarks {
                "No matches"
            } else {
                "No bookmarks yet"
            },
            t,
        )]
    } else {
        render_sections(sections, state.query.selected, t, |bookmark, selected| {
            bookmark_row(bookmark, selected, t, view)
        })
    };
    picker::panel(
        "bookmark-picker-panel",
        280.,
        header,
        rows,
        &state.query.scroll,
        t,
    )
}
