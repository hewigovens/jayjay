use gpui::{Context, Div, Focusable, ParentElement, Window};

use super::super::RepoWindow;
use super::super::conflict_editor::conflict_editor_overlay;
use super::super::file_editor::file_editor_overlay;
use crate::app::theme::Theme;
use crate::ui::loading_hud::loading_hud;

impl RepoWindow {
    pub(super) fn sync_editors(&mut self, cx: &mut Context<Self>) {
        self.sync_conflict_editor_selection(cx);
        self.sync_file_editor_selection(cx);
    }

    pub(super) fn append_editor_overlays(
        &mut self,
        mut root: Div,
        t: &Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        if self.file_editor.focus_pending {
            if let Some(editor) = self.file_editor.editor.as_ref() {
                let handle = editor.read(cx).focus_handle(cx);
                window.focus(&handle, cx);
            }
            self.file_editor.focus_pending = false;
        }

        if self.file_editor.active {
            root = root.child(file_editor_overlay(self, t, cx));
        }

        if self.conflict_editor.focus_pending {
            if self.conflict_editor.show_raw
                && let Some(editor) = self.conflict_editor.result.as_ref()
            {
                let handle = editor.read(cx).focus_handle(cx);
                window.focus(&handle, cx);
            } else {
                window.focus(&self.focus_handle, cx);
            }
            self.conflict_editor.focus_pending = false;
        }
        if self.conflict_editor.preparing || self.file_editor.preparing {
            root = root.child(loading_hud(t));
        }
        if self.conflict_editor.active {
            root = root.child(conflict_editor_overlay(self, t, cx));
        }
        root
    }

    pub(super) fn dismiss_editor_overlay(&mut self, cx: &mut Context<Self>) -> bool {
        if self.file_editor.active || self.file_editor.preparing {
            self.exit_file_editor(cx);
        } else if self.conflict_editor.active || self.conflict_editor.preparing {
            self.exit_conflict_editor(cx);
        } else {
            return false;
        }
        true
    }

    pub(super) fn editor_input_focused(&self, window: &Window, cx: &gpui::App) -> bool {
        self.conflict_editor
            .result
            .as_ref()
            .is_some_and(|result| result.read(cx).focus_handle(cx).is_focused(window))
            || self
                .file_editor
                .editor
                .as_ref()
                .is_some_and(|editor| editor.read(cx).focus_handle(cx).is_focused(window))
    }
}
