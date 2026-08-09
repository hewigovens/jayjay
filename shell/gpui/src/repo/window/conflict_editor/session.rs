use gpui::{AppContext as _, Context};
use jayjay_core::diff::{highlight_file, highlight_file_against_base};
use jayjay_core::merge_hunk_display_diff;

use crate::ui::text_area::TextArea;

use super::super::RepoWindow;
use super::ConflictEditorState;

impl RepoWindow {
    pub fn conflict_editor_active(&self) -> bool {
        self.conflict_editor.active
    }

    pub fn conflict_editor_has_syntax_highlights(&self, cx: &gpui::App) -> bool {
        self.conflict_editor
            .sources
            .as_ref()
            .is_some_and(|sources| {
                sources
                    .iter()
                    .all(|source| source.read(cx).has_syntax_highlights())
            })
            && self
                .conflict_editor
                .result
                .as_ref()
                .is_some_and(|result| result.read(cx).has_syntax_highlights())
    }

    pub fn conflict_editor_has_diff_highlights(&self, cx: &gpui::App) -> bool {
        self.conflict_editor
            .sources
            .as_ref()
            .is_some_and(|sources| {
                sources[0].read(cx).has_diff_highlights()
                    && sources[2].read(cx).has_diff_highlights()
            })
    }

    pub(crate) fn sync_conflict_editor_selection(&mut self, cx: &mut Context<Self>) {
        if !self.conflict_editor.active && !self.conflict_editor.preparing {
            return;
        }
        let remains_current = {
            let vm = self.vm.read(cx);
            let selected_path = vm.selected_hunk().map(|hunk| hunk.path.as_str());
            let selected_revision = vm.selected_revision();
            // Change ids survive working-copy snapshots; keying on commit ids would discard open resolutions on refresh.
            let selected_change = vm
                .selected_change()
                .map(|change| change.change_id.id.as_str());
            let expected_change = self
                .conflict_editor
                .data
                .as_ref()
                .map(|data| data.change_id.as_str());
            selected_path == Some(self.conflict_editor.path.as_str())
                && vm.compare.is_none()
                && selected_revision.as_deref() == Some(self.conflict_editor.rev.as_str())
                && expected_change.is_none_or(|expected| selected_change == Some(expected))
        };
        if !remains_current {
            self.exit_conflict_editor(cx);
        }
    }

    pub(crate) fn enter_selected_conflict_editor(&mut self, cx: &mut Context<Self>) {
        if self.conflict_editor.preparing || self.file_editor.preparing {
            return;
        }
        let can_edit = {
            let vm = self.vm.read(cx);
            vm.compare.is_none()
                && vm
                    .selected_change()
                    .is_some_and(|change| !change.is_immutable)
                && vm
                    .selected_hunk()
                    .is_some_and(|hunk| hunk.supports_conflict_editor)
        };
        if !can_edit {
            self.show_toast("This conflict cannot be edited", cx);
            return;
        }
        let Some((rev, path)) = self.selected_resolution_target(cx) else {
            self.show_toast("No conflicted file selected", cx);
            return;
        };
        self.conflict_editor.session = self.conflict_editor.session.wrapping_add(1);
        let session = self.conflict_editor.session;
        self.conflict_editor.active = false;
        self.conflict_editor.preparing = true;
        self.conflict_editor.focus_pending = false;
        self.conflict_editor.show_base = false;
        self.conflict_editor.show_raw = false;
        self.conflict_editor.selected_hunk = 0;
        self.conflict_editor.rev = rev.clone();
        self.conflict_editor.path = path.clone();
        self.conflict_editor.data = None;
        self.conflict_editor.hunk_diffs.clear();
        self.conflict_editor.sources = None;
        self.conflict_editor.result = None;
        self.conflict_editor.selected_source = None;
        self.conflict_editor.saving = false;
        let task = self
            .vm
            .update(cx, |vm, cx| vm.load_conflict_editor(rev, path, cx));
        cx.spawn(async move |this, cx| match task.await {
            Ok(data) => {
                let path = data.path.clone();
                let left = data.left.clone();
                let base = data.base.clone();
                let right = data.right.clone();
                let result = data.result.clone();
                let hunks = data.hunks.clone();
                let highlighted = cx
                    .background_spawn(async move {
                        (
                            highlight_file_against_base(&path, &base, &left),
                            highlight_file(&path, &base),
                            highlight_file_against_base(&path, &base, &right),
                            highlight_file(&path, &result),
                            hunks
                                .iter()
                                .map(|hunk| merge_hunk_display_diff(&path, &result, hunk))
                                .collect::<Vec<_>>(),
                        )
                    })
                    .await;
                let _ = this.update(cx, move |view, cx| {
                    if !view.conflict_editor.preparing || view.conflict_editor.session != session {
                        return;
                    }
                    let sources = [
                        cx.new(|cx| {
                            TextArea::prepared_diff_highlighted_code_block(
                                data.left.clone(),
                                data.path.clone(),
                                highlighted.0.clone(),
                                cx,
                            )
                            .full_bleed_pane()
                        }),
                        cx.new(|cx| {
                            TextArea::prepared_highlighted_code_block(
                                data.base.clone(),
                                data.path.clone(),
                                highlighted.1.clone(),
                                cx,
                            )
                            .full_bleed_pane()
                        }),
                        cx.new(|cx| {
                            TextArea::prepared_diff_highlighted_code_block(
                                data.right.clone(),
                                data.path.clone(),
                                highlighted.2.clone(),
                                cx,
                            )
                            .full_bleed_pane()
                        }),
                    ];
                    let result = cx.new(|cx| {
                        TextArea::prepared_code_editor(
                            data.result.clone(),
                            data.path.clone(),
                            "Merge result",
                            360.,
                            highlighted.3.clone(),
                            cx,
                        )
                        .full_bleed_pane()
                        .starting_at_top()
                    });
                    TextArea::subscribe_updates(&result, cx);
                    view.conflict_editor.active = true;
                    view.conflict_editor.preparing = false;
                    view.conflict_editor.focus_pending = true;
                    view.conflict_editor.data = Some(data);
                    view.conflict_editor.hunk_diffs = highlighted.4;
                    view.conflict_editor.show_raw = view
                        .conflict_editor
                        .data
                        .as_ref()
                        .is_none_or(|data| data.hunks.is_empty());
                    view.conflict_editor.sources = Some(sources);
                    view.conflict_editor.result = Some(result);
                    cx.notify();
                });
            }
            Err(_) => {
                let _ = this.update(cx, move |view, cx| {
                    if view.conflict_editor.session == session {
                        view.exit_conflict_editor(cx);
                    }
                });
            }
        })
        .detach();
        cx.notify();
    }

    pub(crate) fn exit_conflict_editor(&mut self, cx: &mut Context<Self>) {
        let session = self.conflict_editor.session;
        self.conflict_editor = ConflictEditorState {
            session,
            ..Default::default()
        };
        cx.notify();
    }
}
