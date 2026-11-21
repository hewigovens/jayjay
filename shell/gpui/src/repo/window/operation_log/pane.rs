use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, Point, ScrollStrategy, Styled, UniformListScrollHandle, Window, div, px, rgb,
};
use jayjay_core::RecentOperations;

use super::rows::operation_list;
use crate::app::theme::Theme;
use crate::repo::window::{PanelBoundsSlot, RepoWindow};
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::glyph;
use crate::ui::loading::loading_status;
use crate::ui::primitives::placeholder;

pub(crate) struct OperationLogPane {
    pub(super) selected: Option<String>,
    /// An operation Undo was asked to undo after the repo moved past it; its parent is preselected instead.
    undoing: Option<String>,
    pub(super) seen: Option<Arc<RecentOperations>>,
    pub(super) focus: FocusHandle,
    focus_pending: bool,
    pub(super) scroll: UniformListScrollHandle,
    pub(super) selected_bounds: PanelBoundsSlot,
}

impl OperationLogPane {
    pub(crate) fn new(cx: &mut App) -> Self {
        Self {
            selected: None,
            undoing: None,
            seen: None,
            focus: cx.focus_handle(),
            focus_pending: false,
            scroll: UniformListScrollHandle::new(),
            selected_bounds: PanelBoundsSlot::default(),
        }
    }

    pub(crate) fn is_focused(&self, window: &Window) -> bool {
        self.focus.is_focused(window)
    }

    fn select_restore_target(&mut self) {
        self.selected = self
            .seen
            .as_ref()
            .and_then(|operations| operations.restore_target(self.undoing.as_deref()))
            .map(str::to_owned);
    }
}

impl RepoWindow {
    pub(crate) fn toggle_operation_log(&mut self, cx: &mut Context<Self>) {
        if self.layout.operation_log_visible() {
            crate::app::config::update(cx, |c| c.layout.show_operation_log = false);
            self.apply_operation_log_shown(false, cx);
        } else {
            self.show_operation_log(cx);
        }
    }

    fn show_operation_log(&mut self, cx: &mut Context<Self>) {
        crate::app::config::update(cx, |c| c.layout.show_operation_log = true);
        self.apply_operation_log_shown(true, cx);
        self.operation_log.focus_pending = true;
        let viewport_width = self.layout.viewport_width;
        if self
            .layout
            .operation_log_pane_width(viewport_width)
            .is_none()
        {
            self.show_toast(jayjay_i18n::oplog::no_room_for_operation_log(), cx);
        }
    }

    pub fn operation_log_toggle_title(&self) -> String {
        operation_log_title(self.layout.operation_log_visible())
    }

    pub(crate) fn show_operation_log_undoing(
        &mut self,
        operation_id: String,
        cx: &mut Context<Self>,
    ) {
        self.operation_log.undoing = Some(operation_id);
        self.operation_log.select_restore_target();
        if !self.layout.operation_log_visible() {
            self.show_operation_log(cx);
        }
        cx.notify();
    }

    pub(crate) fn apply_operation_log_shown(&mut self, shown: bool, cx: &mut Context<Self>) {
        if self.layout.operation_log_shown == shown {
            return;
        }
        self.layout.operation_log_shown = shown;
        self.layout.operation_log_slide += 1;
        self.layout.operation_log_closing = !shown;
        if !shown {
            self.finish_closing_slide(
                |layout| {
                    (
                        layout.operation_log_slide,
                        &mut layout.operation_log_closing,
                    )
                },
                cx,
            );
            self.operation_log.undoing = None;
            self.operation_log.select_restore_target();
        }
        cx.notify();
    }

    /// A refresh keeps the selected operation while the pane shows it; a hidden pane follows the restore target.
    pub(crate) fn sync_operation_log(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let operations = self.vm.read(cx).operations.clone();
        let pane = &mut self.operation_log;
        if pane.seen.as_ref().map(Arc::as_ptr) != operations.as_ref().map(Arc::as_ptr) {
            let first_load = pane.seen.is_none();
            pane.seen = operations;
            if first_load || !self.layout.operation_log_shown {
                pane.select_restore_target();
            } else {
                pane.selected = pane.selected.take().filter(|id| {
                    pane.seen.as_ref().is_some_and(|operations| {
                        operations
                            .entries()
                            .iter()
                            .any(|entry| entry.id.as_str() == id)
                    })
                });
            }
        }
        let visible = self.layout.operation_log_visible();
        if pane.focus_pending {
            pane.focus_pending = false;
            if visible {
                window.focus(&pane.focus, cx);
            }
        } else if !visible && pane.focus.is_focused(window) {
            window.focus(&self.focus_handle, cx);
        }
    }

    pub(super) fn select_operation(&mut self, operation_id: String, cx: &mut Context<Self>) {
        self.operation_log.selected = Some(operation_id);
        cx.notify();
    }

    fn restorable_operation(&self, operation_id: &str, cx: &App) -> bool {
        !self.vm.read(cx).restoring_operation
            && self.operation_log.seen.as_ref().is_some_and(|operations| {
                operations
                    .entries()
                    .iter()
                    .any(|entry| entry.id.as_str() == operation_id && !entry.is_current)
            })
    }

    pub(crate) fn selected_restorable_operation(&self, cx: &App) -> Option<String> {
        self.operation_log
            .selected
            .clone()
            .filter(|id| self.layout.operation_log_visible() && self.restorable_operation(id, cx))
    }

    pub(crate) fn restore_operation(&mut self, operation_id: String, cx: &mut Context<Self>) {
        if !self.restorable_operation(&operation_id, cx) {
            return;
        }
        let Some(task) = self
            .vm
            .update(cx, |vm, cx| vm.restore_operation(operation_id, cx))
        else {
            return;
        };
        self.operation_log.undoing = None;
        Self::spawn_ok(cx, task, |view, operation, cx| {
            view.present_toast(
                jayjay_i18n::oplog::restored_operation().into(),
                operation,
                cx,
            );
        });
    }

    pub(super) fn open_operation_menu(
        &mut self,
        operation_id: String,
        position: Point<gpui::Pixels>,
        cx: &mut Context<Self>,
    ) {
        let restorable = self.restorable_operation(&operation_id, cx);
        self.select_operation(operation_id.clone(), cx);
        self.open_context_menu(
            position,
            vec![
                ContextMenuItem::new(
                    jayjay_i18n::repo::restore(),
                    glyph::HISTORY,
                    ContextAction::RestoreOperation(operation_id.into()),
                )
                .with_enabled(restorable),
            ],
            cx,
        );
    }

    fn handle_operation_log_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) -> bool {
        let key = &event.keystroke;
        let modifiers = &key.modifiers;
        let plain = !modifiers.modified();
        let opens_menu = match key.key.as_str() {
            "enter" => modifiers.control && !modifiers.shift && !modifiers.alt,
            "menu" => plain,
            "f10" => modifiers.shift && !modifiers.control && !modifiers.alt,
            _ => false,
        };
        if opens_menu {
            if let (Some(selected), Some(bounds)) = (
                self.operation_log.selected.clone(),
                self.operation_log.selected_bounds.get(),
            ) {
                self.open_operation_menu(selected, bounds.bottom_left(), cx);
            }
            return true;
        }
        let step: isize = match key.key.as_str() {
            "up" if plain => -1,
            "down" if plain => 1,
            _ => return false,
        };
        let Some(operations) = self.operation_log.seen.clone() else {
            return true;
        };
        let entries = operations.entries();
        let current = self.operation_log.selected.as_deref().and_then(|selected| {
            entries
                .iter()
                .position(|entry| entry.id.as_str() == selected)
        });
        let next = match current {
            Some(ix) => ix
                .saturating_add_signed(step)
                .min(entries.len().saturating_sub(1)),
            None => 0,
        };
        if let Some(entry) = entries.get(next) {
            self.operation_log
                .scroll
                .scroll_to_item(next, ScrollStrategy::Nearest);
            self.select_operation(entry.id.id.clone(), cx);
        }
        true
    }
}

pub(crate) fn operation_log_title(shown: bool) -> String {
    if shown {
        jayjay_i18n::menu::hide_operation_log()
    } else {
        jayjay_i18n::menu::show_operation_log()
    }
}

pub(crate) fn operation_log_pane(
    view: &RepoWindow,
    width: f32,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let pane = &view.operation_log;
    let body = match pane.seen.as_deref() {
        None => loading_status(jayjay_i18n::oplog::loading_operations_ellipsis(), t),
        Some(RecentOperations::Failed(message)) => placeholder(message.clone(), t),
        Some(RecentOperations::Entries(entries)) if entries.is_empty() => {
            placeholder(jayjay_i18n::repo::no_operations_found(), t)
        }
        Some(RecentOperations::Entries(_)) => operation_list(view, t.clone(), cx),
    };
    div()
        .id("operation-log-pane")
        .debug_selector(|| "operation-log-pane".to_owned())
        .track_focus(&pane.focus)
        .on_key_down(cx.listener(|view, event: &KeyDownEvent, _, cx| {
            if view.context_menu.is_none() && view.handle_operation_log_key(event, cx) {
                cx.stop_propagation();
            }
        }))
        .flex()
        .flex_col()
        .flex_none()
        .w(px(width))
        .h_full()
        .overflow_hidden()
        .bg(rgb(t.sidebar_bg))
        .child(body)
        .into_any_element()
}
