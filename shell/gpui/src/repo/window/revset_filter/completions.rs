use gpui::{
    AnyElement, App, Bounds, Entity, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    MouseDownEvent, ParentElement, Pixels, ScrollHandle, StatefulInteractiveElement, Styled,
    anchored, deferred, div, point, px, rgb,
};
use jayjay_core::utf16::{byte_offset, utf16_offset};
use jayjay_core::{RevsetCompletion, RevsetCompletionKind, RevsetVocabulary, revset_completions};

use super::super::{RepoWindow, picker};
use crate::app::fonts::CodeText as _;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::input::LineInput;
use crate::ui::navigation::{ListNavKeys, list_nav_from_key, move_index};

pub(super) fn completions_at_caret(
    input: &LineInput,
    vocabulary: &RevsetVocabulary,
) -> Vec<RevsetCompletion> {
    let cursor = utf16_offset(input.text(), input.edit().cursor_offset());
    revset_completions(input.text(), cursor as u32, vocabulary)
}

pub(super) fn insert_completion(input: &mut LineInput, entry: &RevsetCompletion) {
    let start = byte_offset(input.text(), entry.start as usize);
    let end = byte_offset(input.text(), (entry.start + entry.len) as usize);
    input.replace_range(start..end, &entry.text);
}

pub(super) enum CompletionOutcome {
    Consumed,
    Dismiss,
    Accept(usize),
}

/// Candidates for the symbol the revset editor's caret sits in.
pub(crate) struct RevsetCompletionState {
    entries: Vec<RevsetCompletion>,
    /// No row is picked until the caret moves into the list, so Return keeps applying the revset.
    selected: Option<usize>,
    scroll: ScrollHandle,
}

impl RevsetCompletionState {
    pub(super) fn new(entries: Vec<RevsetCompletion>) -> Option<Self> {
        (!entries.is_empty()).then(|| Self {
            entries,
            selected: None,
            scroll: ScrollHandle::new(),
        })
    }

    pub(super) fn entry(&self, index: usize) -> Option<&RevsetCompletion> {
        self.entries.get(index)
    }

    /// `None` when the key belongs to the revset editor.
    pub(super) fn handle_key(&mut self, event: &KeyDownEvent) -> Option<CompletionOutcome> {
        match event.keystroke.key.as_str() {
            "escape" => Some(CompletionOutcome::Dismiss),
            // Tab belongs to the window's focus cycle, so a picked row is what Return takes.
            "enter" => self.selected.map(CompletionOutcome::Accept),
            _ => {
                let direction = list_nav_from_key(event, ListNavKeys::COMMAND_PALETTE)?;
                let selected = self.selected.map_or(0, |current| {
                    move_index(Some(current), self.entries.len(), direction).unwrap_or(current)
                });
                self.selected = Some(selected);
                self.scroll.scroll_to_item(selected);
                Some(CompletionOutcome::Consumed)
            }
        }
    }
}

/// No backdrop: a click outside has to reach the editor or whatever takes focus from it.
pub(crate) fn render_revset_completions(
    state: &RevsetCompletionState,
    bar: Bounds<Pixels>,
    t: &Theme,
    view: &Entity<RepoWindow>,
) -> AnyElement {
    let list = div()
        .id("revset-completions")
        .debug_selector(|| "revset-completions".to_owned())
        .flex()
        .flex_col()
        .w(bar.size.width)
        .max_h(px(176.))
        .py(px(4.))
        .bg(rgb(t.detail_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_lg()
        .overflow_y_scroll()
        .track_scroll(&state.scroll)
        .occlude()
        .children(state.entries.iter().enumerate().map(|(index, entry)| {
            let view = view.clone();
            completion_row(index, entry, state.selected == Some(index), t, move |cx| {
                view.update(cx, |view, cx| view.accept_revset_completion(index, cx));
            })
        }));
    deferred(
        anchored()
            .position(point(bar.origin.x, bar.bottom() + px(4.)))
            .snap_to_window_with_margin(px(6.))
            .child(list),
    )
    .with_priority(3)
    .into_any_element()
}

pub(super) fn completion_row(
    index: usize,
    entry: &RevsetCompletion,
    selected: bool,
    t: &Theme,
    on_pick: impl Fn(&mut App) + 'static,
) -> AnyElement {
    picker::row(format!("revset-completion-{index}"), selected, 24., t)
        .gap(px(8.))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, _, cx| {
            cx.stop_propagation();
            on_pick(cx);
        })
        .child(
            div()
                .code_text(12.)
                .text_color(rgb(t.fg))
                .child(entry.text.clone()),
        )
        .child(
            div()
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_faint))
                .child(match entry.kind {
                    RevsetCompletionKind::Function => "function",
                    RevsetCompletionKind::Alias => "alias",
                    RevsetCompletionKind::Bookmark => "bookmark",
                    RevsetCompletionKind::Tag => "tag",
                }),
        )
        .into_any_element()
}
