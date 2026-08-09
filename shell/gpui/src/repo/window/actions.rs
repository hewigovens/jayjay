use gpui::{AppContext, Context, ScrollStrategy, SharedString, point, px};

use super::{
    ActivePane, DiffRichPreviewKind, DiffRichPreviewSelection, RepoWindow, TextModalAction,
    TextModalState,
};
use crate::diff::projection;
use crate::repo::revset;
use crate::ui::text_area::TextArea;
use crate::windows::bookmark_manager::BookmarkManagerView;
use crate::windows::operation_log::OperationLogView;

impl RepoWindow {
    pub fn select_or_compare_change(
        &mut self,
        ix: usize,
        shift_pressed: bool,
        cx: &mut Context<Self>,
    ) {
        let selected = self.vm.read(cx).selected;
        if shift_pressed
            && let Some(selected_ix) = selected
            && selected_ix != ix
        {
            if self.diff_edit_active() {
                self.exit_diff_edit(cx);
            }
            if self.conflict_editor.active || self.conflict_editor.preparing {
                self.exit_conflict_editor(cx);
            }
            self.active_pane = ActivePane::Sidebar;
            self.find.matches.clear();
            self.find.current = 0;
            self.diff.selection = None;
            let vm = self.vm.clone();
            vm.update(cx, |vm, cx| vm.compare_changes(selected_ix, ix, cx));
            return;
        }
        self.select_change(ix, cx);
    }

    pub fn select_change(&mut self, ix: usize, cx: &mut Context<Self>) {
        if self.diff_edit_active() {
            self.exit_diff_edit(cx);
        }
        if self.conflict_editor.active || self.conflict_editor.preparing {
            self.exit_conflict_editor(cx);
        }
        self.active_pane = ActivePane::Sidebar;
        self.find.matches.clear();
        self.find.current = 0;
        self.file_column.multi_select.clear();
        if self.vm.read(cx).selected != Some(ix) {
            self.reset_diff_panel_for_new_file();
        } else {
            self.diff.selection = None;
        }
        let vm = self.vm.clone();
        vm.update(cx, |vm, cx| vm.select_change(ix, cx));
    }

    pub(crate) fn reveal_change_id(&mut self, change_id: &str, cx: &mut Context<Self>) {
        let ix = {
            let vm = self.vm.read(cx);
            vm.graph
                .changes
                .iter()
                .position(|c| c.change_id.starts_with(change_id))
        };
        if let Some(ix) = ix {
            self.scrolls
                .changes
                .scroll_to_item(ix, ScrollStrategy::Center);
            self.select_change(ix, cx);
        }
    }

    pub(crate) fn open_bookmark_manager(&mut self, cx: &mut Context<Self>) {
        let vm = self.vm.read(cx);
        let Some(repo) = vm.repo.clone() else {
            return;
        };
        BookmarkManagerView::open(repo, cx.entity(), vm.graph.bookmarks.clone(), cx);
    }

    pub(crate) fn open_operation_log(&mut self, cx: &mut Context<Self>) {
        let Some(repo) = self.vm.read(cx).repo.clone() else {
            self.show_toast("Repository is not open", cx);
            return;
        };
        OperationLogView::open(repo, cx.entity(), cx);
    }

    fn open_edit_description(&mut self, rev: String, description: String, cx: &mut Context<Self>) {
        self.open_description_modal(
            rev.clone(),
            description,
            TextModalAction::EditDescription { rev },
            cx,
        );
    }

    pub(super) fn open_diff_edit_description(&mut self, cx: &mut Context<Self>) {
        let Some((subtitle, message, session)) = self.diff_edit_description_context() else {
            return;
        };
        self.open_description_modal(
            subtitle,
            message,
            TextModalAction::DiffEditDescription { session },
            cx,
        );
    }

    fn open_description_modal(
        &mut self,
        subtitle: String,
        description: String,
        action: TextModalAction,
        cx: &mut Context<Self>,
    ) {
        let input = cx.new(|cx| TextArea::new(description, "Description", true, 190., cx));
        self.text_modal = Some(TextModalState {
            title: "Edit Description".into(),
            subtitle: subtitle.into(),
            primary_label: "Save".into(),
            action,
            input,
            focus_pending: true,
            context: None,
            checkbox: None,
            file_list: None,
        });
        cx.notify();
    }

    pub(crate) fn open_create_bookmark(&mut self, rev: String, cx: &mut Context<Self>) {
        let input = cx.new(|cx| TextArea::new("", "Bookmark name", false, 32., cx));
        self.text_modal = Some(TextModalState {
            title: "Create Bookmark".into(),
            subtitle: rev.chars().take(12).collect::<String>().into(),
            primary_label: "Create".into(),
            action: TextModalAction::CreateBookmark { rev },
            input,
            focus_pending: true,
            context: None,
            checkbox: None,
            file_list: None,
        });
        cx.notify();
    }

    pub(crate) fn close_text_modal(&mut self, cx: &mut Context<Self>) {
        if self.text_modal.take().is_some() {
            cx.notify();
        }
    }

    /// Flips the modal's optional checkbox (currently only the split-files modal's "Parallel split"); a no-op if the open modal has none.
    pub fn toggle_text_modal_checkbox(&mut self, cx: &mut Context<Self>) {
        if let Some(checkbox) = self.text_modal.as_mut().and_then(|m| m.checkbox.as_mut()) {
            checkbox.checked ^= true;
            cx.notify();
        }
    }

    pub fn submit_text_modal(&mut self, cx: &mut Context<Self>) {
        let Some(modal) = self.text_modal.as_ref() else {
            return;
        };
        let text = modal.input.read(cx).text();
        match modal.action.clone() {
            TextModalAction::EditDescription { rev } => {
                self.text_modal = None;
                let task = self
                    .vm
                    .update(cx, |vm, cx| vm.describe_change(rev, text, cx));
                task.detach();
            }
            TextModalAction::DiffEditDescription { session } => {
                self.text_modal = None;
                self.apply_diff_edit_description(session, text);
            }
            TextModalAction::CreateBookmark { rev } => {
                let name = text.trim().to_string();
                if name.is_empty() {
                    self.show_toast("Bookmark name required", cx);
                    return;
                }
                if !jayjay_core::is_valid_bookmark_name(&name) {
                    self.show_toast(format!("Invalid bookmark name: {name}"), cx);
                    return;
                }
                self.text_modal = None;
                let task = self
                    .vm
                    .update(cx, |vm, cx| vm.create_bookmark(name.clone(), rev, cx));
                cx.spawn(async move |this, cx| {
                    if task.await.is_ok() {
                        let _ = this.update(cx, move |view, cx| {
                            view.show_toast(format!("Created bookmark {name}"), cx);
                        });
                    }
                })
                .detach();
            }
            TextModalAction::ReviewNote(target) => {
                if text.trim().is_empty() {
                    self.show_toast("Note cannot be empty", cx);
                    return;
                }
                self.text_modal = None;
                self.save_review_note(target, text, cx);
            }
            TextModalAction::CreateWorkspace(parent) => {
                self.submit_create_workspace(parent, &text, cx);
            }
            TextModalAction::SplitFiles(request) => {
                let message = text.trim().to_owned();
                if message.is_empty() {
                    self.show_toast("Description required", cx);
                    return;
                }
                let parallel = self
                    .text_modal
                    .as_ref()
                    .and_then(|m| m.checkbox.as_ref())
                    .is_some_and(|c| c.checked);
                self.text_modal = None;
                self.confirm_split_files(request, message, parallel, cx);
            }
        }
        cx.notify();
    }

    /// SwiftUI parity: every commit path (Commit button, Commit N Files) needs a non-empty summary line; a body-only draft may Describe but never commit with a blank subject.
    pub(super) fn commit_message_requiring_summary(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Option<String> {
        if self.summary_input.read(cx).text().trim().is_empty() {
            self.show_toast("Summary required", cx);
            return None;
        }
        Some(self.commit_box_message(cx))
    }

    /// Clears both commit-box inputs and drops any pending AI generation, whose reply snapshotted the pre-commit inputs and must not refill the cleared box.
    pub(super) fn clear_commit_box(&mut self, cx: &mut Context<Self>) {
        self.summary_input.update(cx, |input, cx| input.clear(cx));
        self.description_input
            .update(cx, |input, cx| input.clear(cx));
        self.cancel_pending_commit_message_generation();
    }

    pub fn commit_working_copy_from_input(&mut self, cx: &mut Context<Self>) {
        let Some(message) = self.commit_message_requiring_summary(cx) else {
            return;
        };
        let committed_change_id = self
            .vm
            .read(cx)
            .working_copy_change()
            .map(|c| c.change_id.id.clone());
        let task = self
            .vm
            .update(cx, |vm, cx| vm.commit_working_copy(message, cx));
        cx.spawn(async move |this, cx| {
            if task.await.is_ok() {
                let _ = this.update(cx, |view, cx| {
                    if let Some(change_id) = committed_change_id {
                        super::review::mutate(&view.review_store, |store| {
                            store.clear_change(&change_id);
                        });
                    }
                    view.clear_commit_box(cx);
                });
            }
        })
        .detach();
    }

    /// `jj describe` on @: saves the box message as the working copy's description without starting a new change, so the inputs keep mirroring @ and stay put.
    pub fn describe_working_copy_from_input(&mut self, cx: &mut Context<Self>) {
        let message = self.commit_box_message(cx);
        if message.is_empty() {
            self.show_toast("Description required", cx);
            return;
        }
        // Unlike commit, a pending AI generation stays valid: describe leaves the inputs (its snapshot) and the working-copy diff untouched, and the untouched-snapshot guard already drops replies once the user types.
        let task = self
            .vm
            .update(cx, |vm, cx| vm.describe_change("@".to_owned(), message, cx));
        task.detach();
    }

    pub fn select_file(&mut self, ix: usize, cx: &mut Context<Self>) {
        if self.conflict_editor.active || self.conflict_editor.preparing {
            self.exit_conflict_editor(cx);
        }
        self.active_pane = ActivePane::FileColumn;
        self.collapse_file_multi_select(ix, cx);
        if self.vm.read(cx).selected_file_ix == Some(ix) {
            cx.notify();
            return;
        }

        self.reset_diff_panel_for_new_file();
        let vm = self.vm.clone();
        vm.update(cx, |vm, cx| vm.select_file(ix, cx));
    }

    fn reset_diff_panel_for_new_file(&mut self) {
        self.diff.selection = None;
        self.diff.gutter_selection = None;
        self.diff.rich_preview = None;
        self.reset_context_expansion();
        let base = self.scrolls.diff.0.borrow().base_handle.clone();
        let offset = base.offset();
        base.set_offset(point(offset.x, px(0.)));
        self.scrolls
            .diff
            .scroll_to_item_strict(0, ScrollStrategy::Top);
        self.diff.markdown_scroll.set_offset(point(px(0.), px(0.)));
    }

    pub fn edit_selected_description(&mut self, cx: &mut Context<Self>) {
        let Some(change) = self.vm.read(cx).selected_change().cloned() else {
            return;
        };
        if change.is_immutable {
            self.show_toast("Immutable change cannot be edited", cx);
            return;
        }
        if change.is_working_copy {
            return;
        }
        self.open_edit_description(
            revset::change_revision(&change),
            change.description.clone(),
            cx,
        );
    }

    pub fn toggle_view_mode(&mut self, cx: &mut Context<Self>) {
        let vm = self.vm.clone();
        vm.update(cx, |vm, cx| vm.toggle_view_mode(cx));
    }

    pub fn toggle_projection_rich_preview(&mut self, cx: &mut Context<Self>) {
        let (rev, hunk) = {
            let vm = self.vm.read(cx);
            let rev = vm.selected_revision();
            let hunk = vm.selected_hunk().cloned();
            (rev, hunk)
        };
        let (Some(rev), Some(hunk)) = (rev, hunk) else {
            return;
        };
        if hunk.projection.is_none() {
            return;
        }

        let active = self.toggle_rich_preview(DiffRichPreviewKind::Projection, hunk.path.as_str());
        let projection_mode = projection::request_mode(hunk.projection.as_ref(), active);
        let vm = self.vm.clone();
        vm.update(cx, |vm, cx| {
            vm.load_diff_async_with_projection(rev, hunk, projection_mode, cx)
        });
        cx.notify();
    }

    pub(crate) fn toggle_svg_rich_preview(&mut self, cx: &mut Context<Self>) {
        let hunk = self.vm.read(cx).selected_hunk().cloned();
        let Some(hunk) = hunk else {
            return;
        };
        if !projection::can_render_svg_preview(&hunk) {
            return;
        }
        self.toggle_rich_preview(DiffRichPreviewKind::Svg, hunk.path.as_str());
        cx.notify();
    }

    pub fn toggle_markdown_rich_preview(&mut self, cx: &mut Context<Self>) {
        let hunk = self.vm.read(cx).selected_hunk().cloned();
        let Some(hunk) = hunk else {
            return;
        };
        if !projection::can_render_markdown_file_preview(&hunk) {
            return;
        }
        self.toggle_rich_preview(DiffRichPreviewKind::Markdown, hunk.path.as_str());
        cx.notify();
    }

    fn toggle_rich_preview(&mut self, kind: DiffRichPreviewKind, path: &str) -> bool {
        if self
            .diff
            .rich_preview
            .as_ref()
            .is_some_and(|selection| selection.is_active(kind, path))
        {
            self.diff.rich_preview = None;
            false
        } else {
            self.diff.rich_preview = Some(DiffRichPreviewSelection {
                kind,
                path: path.to_owned(),
            });
            true
        }
    }

    pub(crate) fn toggle_annotate(&mut self, cx: &mut Context<Self>) {
        let vm = self.vm.clone();
        vm.update(cx, |vm, cx| vm.toggle_annotate(cx));
    }

    pub(crate) fn load_more(&mut self, cx: &mut Context<Self>) {
        let vm = self.vm.clone();
        vm.update(cx, |vm, cx| vm.load_more(cx));
    }

    pub(crate) fn mark_copied(&mut self, id: SharedString, cx: &mut Context<Self>) {
        self.feedback.recently_copied = Some(id.clone());
        cx.notify();
        let id_for_clear = id;
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1500))
                .await;
            let _ = this.update(cx, move |view, cx| {
                if view.feedback.recently_copied.as_ref() == Some(&id_for_clear) {
                    view.feedback.recently_copied = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub(crate) fn show_toast(&mut self, message: impl Into<SharedString>, cx: &mut Context<Self>) {
        let message = message.into();
        self.feedback.toast = Some(message.clone());
        cx.notify();
        let id_for_clear = message;
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1800))
                .await;
            let _ = this.update(cx, move |view, cx| {
                if view.feedback.toast.as_ref() == Some(&id_for_clear) {
                    view.feedback.toast = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub fn toggle_dir(&mut self, path: String, cx: &mut Context<Self>) {
        if !self.collapsed_dirs.remove(&path) {
            self.collapsed_dirs.insert(path);
        }
        cx.notify();
    }
}
