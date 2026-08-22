use gpui::{App, Context, KeyDownEvent, ScrollHandle, point, px};

use crate::repo::window::RepoWindow;
use crate::ui::input::LineInput;
use crate::ui::navigation::{ListNav, ListNavKeys, list_nav_from_key};

pub(crate) enum PickerOutcome<A> {
    Handled,
    Dismiss,
    Activate(A),
}

impl RepoWindow {
    /// `None` means the picker `query` selects is closed, so the key is not the picker's.
    pub(in crate::repo::window) fn drive_picker<A: Clone>(
        &mut self,
        event: &KeyDownEvent,
        query: fn(&mut Self) -> Option<&mut PickerQuery>,
        input: fn(&mut Self) -> Option<&mut LineInput>,
        actions: impl Fn(&Self, &App) -> Vec<(A, usize)>,
        cx: &mut Context<Self>,
    ) -> Option<PickerOutcome<A>> {
        let current = actions(self, cx);
        let key = query(self)?.handle_key(event, current.len(), cx);
        let outcome = match key {
            PickerKeyAction::Dismiss => PickerOutcome::Dismiss,
            PickerKeyAction::Activate(index) => match current.get(index) {
                Some((action, _)) => PickerOutcome::Activate(action.clone()),
                None => PickerOutcome::Handled,
            },
            PickerKeyAction::Edited => {
                let count = actions(self, cx).len();
                query(self)?.reset_selection_after_edit(count);
                PickerOutcome::Handled
            }
            PickerKeyAction::Consumed => PickerOutcome::Handled,
        };
        if matches!(outcome, PickerOutcome::Handled) {
            let current = actions(self, cx);
            query(self)?.reveal_selected(&current);
            LineInput::show_for_owner(self, cx, input);
            cx.notify();
        }
        Some(outcome)
    }
}

enum PickerKeyAction {
    Consumed,
    Dismiss,
    Activate(usize),
    Edited,
}

pub(crate) struct PickerQuery {
    pub(crate) input: LineInput,
    pub(crate) scroll: ScrollHandle,
    pub(crate) selected: Option<usize>,
}

impl PickerQuery {
    pub(crate) fn new() -> Self {
        Self {
            input: LineInput::new(""),
            scroll: ScrollHandle::new(),
            selected: None,
        }
    }

    fn reveal_selected<T>(&self, actions: &[(T, usize)]) {
        let item_index = self
            .selected
            .and_then(|index| actions.get(index).map(|(_, item_index)| *item_index));
        if let Some(item_index) = item_index {
            self.scroll.scroll_to_item(item_index);
        } else {
            let offset = self.scroll.offset();
            self.scroll.set_offset(point(offset.x, px(0.)));
        }
    }

    pub(crate) fn reset_selection_after_edit(&mut self, action_count: usize) {
        self.selected = if self.input.text().trim().is_empty() || action_count == 0 {
            None
        } else {
            Some(0)
        };
    }

    fn move_selection(&mut self, direction: ListNav, action_count: usize) {
        if action_count == 0 {
            self.selected = None;
            return;
        }
        self.selected = Some(match self.selected {
            None => 0,
            Some(index) => match direction {
                ListNav::Previous => index.saturating_sub(1),
                ListNav::Next => (index + 1).min(action_count - 1),
            },
        });
    }

    fn handle_key<T>(
        &mut self,
        event: &KeyDownEvent,
        action_count: usize,
        cx: &mut Context<T>,
    ) -> PickerKeyAction {
        match event.keystroke.key.as_str() {
            "escape" => PickerKeyAction::Dismiss,
            "enter" => self
                .selected
                .map(PickerKeyAction::Activate)
                .unwrap_or(PickerKeyAction::Consumed),
            _ => {
                if let Some(direction) = list_nav_from_key(event, ListNavKeys::COMMAND_PALETTE) {
                    self.move_selection(direction, action_count);
                    return PickerKeyAction::Consumed;
                }
                if self.input.handle_key(event, cx).changed {
                    PickerKeyAction::Edited
                } else {
                    PickerKeyAction::Consumed
                }
            }
        }
    }
}
