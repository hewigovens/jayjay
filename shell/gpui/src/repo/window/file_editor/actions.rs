use gpui::{AppContext as _, Context};
use jayjay_core::diff::{ConflictLineKind, highlight_file};
use jayjay_core::{DiffHunk, HunkType};

use crate::ui::text_area::TextArea;

use super::super::RepoWindow;
use super::FileEditorState;

impl RepoWindow {
    pub fn file_editor_active(&self) -> bool {
        self.file_editor.active
    }

    pub fn set_file_editor_content(&mut self, content: String, cx: &mut Context<Self>) {
        if let Some(editor) = self.file_editor.editor.as_ref() {
            editor.update(cx, |editor, cx| editor.set_text(content, cx));
            cx.notify();
        }
    }

    pub fn file_editor_has_syntax_highlights(&self, cx: &gpui::App) -> bool {
        self.file_editor
            .editor
            .as_ref()
            .is_some_and(|editor| editor.read(cx).has_syntax_highlights())
    }

    pub fn file_editor_scroll_offset_y(&self, cx: &gpui::App) -> gpui::Pixels {
        self.file_editor
            .editor
            .as_ref()
            .map_or(gpui::px(0.), |editor| editor.read(cx).scroll_offset_y())
    }

    pub(crate) fn can_edit_selected_working_copy_file(&self, cx: &Context<Self>) -> bool {
        let vm = self.vm.read(cx);
        vm.selected_change()
            .is_some_and(|change| change.is_working_copy)
            && vm.compare.is_none()
            && vm.selected_hunk().is_some_and(hunk_supports_file_editor)
            && vm.current_diff_supports_file_editor
            && !selected_file_has_conflict(vm)
    }

    pub(crate) fn enter_selected_file_editor(&mut self, cx: &mut Context<Self>) {
        if self.file_editor.preparing || self.conflict_editor.preparing {
            return;
        }
        let path = {
            let vm = self.vm.read(cx);
            vm.selected_hunk().map(|hunk| hunk.path.clone())
        };
        let Some(path) = path.filter(|_| self.can_edit_selected_working_copy_file(cx)) else {
            self.show_toast(
                "Only regular text files in the working copy can be edited",
                cx,
            );
            return;
        };
        self.enter_file_editor(path, cx);
    }

    fn enter_file_editor(&mut self, path: String, cx: &mut Context<Self>) {
        self.file_editor.session = self.file_editor.session.wrapping_add(1);
        let session = self.file_editor.session;
        self.file_editor.active = false;
        self.file_editor.preparing = true;
        self.file_editor.focus_pending = false;
        self.file_editor.path = path.clone();
        self.file_editor.data = None;
        self.file_editor.editor = None;
        self.file_editor.saving = false;
        let task = self
            .vm
            .update(cx, |vm, cx| vm.load_working_copy_file_editor(path, cx));
        cx.spawn(async move |this, cx| match task.await {
            Ok(data) => {
                let path = data.path.clone();
                let content = data.content.clone();
                let highlighted_lines = cx
                    .background_spawn(async move { highlight_file(&path, &content) })
                    .await;
                let _ = this.update(cx, move |view, cx| {
                    if !view.file_editor.preparing || view.file_editor.session != session {
                        return;
                    }
                    let editor = cx.new(|cx| {
                        TextArea::prepared_code_editor(
                            data.content.clone(),
                            data.path.clone(),
                            "File content",
                            520.,
                            highlighted_lines,
                            cx,
                        )
                        .full_bleed_pane()
                        .starting_at_top()
                    });
                    TextArea::subscribe_updates(&editor, cx);
                    view.file_editor.active = true;
                    view.file_editor.preparing = false;
                    view.file_editor.focus_pending = true;
                    view.file_editor.data = Some(data);
                    view.file_editor.editor = Some(editor);
                    cx.notify();
                });
            }
            Err(_) => {
                let _ = this.update(cx, move |view, cx| {
                    if view.file_editor.session == session {
                        view.exit_file_editor(cx);
                    }
                });
            }
        })
        .detach();
        cx.notify();
    }

    pub(crate) fn save_file_editor(&mut self, cx: &mut Context<Self>) {
        let (Some(data), Some(editor)) = (
            self.file_editor.data.clone(),
            self.file_editor.editor.as_ref(),
        ) else {
            return;
        };
        let content = editor.read(cx).text();
        if content == data.content || self.file_editor.saving {
            return;
        }
        let session = self.file_editor.session;
        self.file_editor.saving = true;
        let task = self.vm.update(cx, |vm, cx| {
            vm.apply_working_copy_file_editor(data, content, cx)
        });
        cx.spawn(async move |this, cx| {
            let saved = task.await.is_ok();
            let _ = this.update(cx, move |view, cx| {
                if view.file_editor.session != session {
                    return;
                }
                if saved {
                    view.exit_file_editor(cx);
                } else {
                    view.file_editor.saving = false;
                    cx.notify();
                }
            });
        })
        .detach();
        cx.notify();
    }

    pub(crate) fn exit_file_editor(&mut self, cx: &mut Context<Self>) {
        let session = self.file_editor.session;
        self.file_editor = FileEditorState {
            session,
            ..Default::default()
        };
        cx.notify();
    }

    pub(crate) fn sync_file_editor_selection(&mut self, cx: &mut Context<Self>) {
        if !self.file_editor.active && !self.file_editor.preparing {
            return;
        }
        let remains_current = {
            let vm = self.vm.read(cx);
            let selected_path = vm.selected_hunk().map(|hunk| hunk.path.as_str());
            let selected_change = vm.selected_change();
            selected_path == Some(self.file_editor.path.as_str())
                && selected_change.is_some_and(|change| change.is_working_copy)
                && vm.compare.is_none()
                && self.file_editor.data.as_ref().is_none_or(|data| {
                    selected_change.is_some_and(|change| change.change_id.id == data.change_id)
                })
        };
        if !remains_current {
            self.exit_file_editor(cx);
        }
    }
}

fn hunk_supports_file_editor(hunk: &DiffHunk) -> bool {
    hunk.hunk_type != HunkType::Removed
        && hunk.projection.is_none()
        && hunk.new.preview.is_none()
        && !hunk.is_conflict_only_placeholder()
}

fn selected_file_has_conflict(vm: &crate::repo::view_model::RepoViewModel) -> bool {
    vm.selected_hunk()
        .is_some_and(DiffHunk::is_conflict_only_placeholder)
        || vm.current_diff.as_ref().is_some_and(|diff| {
            diff.lines
                .iter()
                .any(|line| line.conflict_kind != ConflictLineKind::None)
        })
}
