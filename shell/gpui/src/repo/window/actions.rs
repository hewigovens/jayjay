use gpui::{AppContext, Context, ScrollStrategy, SharedString, point, px};

use super::{ActivePane, RepoWindow, TextModalAction, TextModalState};
use crate::repo::revset;
use crate::ui::text_area::TextArea;

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
        self.active_pane = ActivePane::Sidebar;
        self.find.matches.clear();
        self.find.current = 0;
        if self.vm.read(cx).selected != Some(ix) {
            self.reset_diff_panel_for_new_file();
        } else {
            self.diff.selection = None;
        }
        let vm = self.vm.clone();
        vm.update(cx, |vm, cx| vm.select_change(ix, cx));
    }

    pub fn reveal_change_id(&mut self, change_id: &str, cx: &mut Context<Self>) {
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

    pub fn open_bookmark_manager(&mut self, cx: &mut Context<Self>) {
        let vm = self.vm.read(cx);
        let Some(repo) = vm.repo.clone() else {
            return;
        };
        crate::windows::bookmark_manager::BookmarkManagerView::open(
            repo,
            cx.entity(),
            vm.graph.bookmarks.clone(),
            cx,
        );
    }

    pub fn open_edit_description(
        &mut self,
        rev: String,
        description: String,
        cx: &mut Context<Self>,
    ) {
        let input = cx.new(|cx| TextArea::new(description, "Description", true, 190., cx));
        self.text_modal = Some(TextModalState {
            title: "Edit Description".into(),
            subtitle: rev.clone().into(),
            primary_label: "Save".into(),
            action: TextModalAction::EditDescription { rev },
            input,
        });
        cx.notify();
    }

    pub fn close_text_modal(&mut self, cx: &mut Context<Self>) {
        if self.text_modal.take().is_some() {
            cx.notify();
        }
    }

    pub fn submit_text_modal(&mut self, cx: &mut Context<Self>) {
        let Some(modal) = self.text_modal.take() else {
            return;
        };
        let text = modal.input.read(cx).text();
        match modal.action {
            TextModalAction::EditDescription { rev } => {
                let task = self
                    .vm
                    .update(cx, |vm, cx| vm.describe_change(rev, text, cx));
                task.detach();
            }
        }
        cx.notify();
    }

    pub fn commit_working_copy_from_input(&mut self, cx: &mut Context<Self>) {
        let message = self.commit_input.read(cx).text().trim().to_string();
        if message.is_empty() {
            self.show_toast("Commit message required", cx);
            return;
        }
        let task = self
            .vm
            .update(cx, |vm, cx| vm.commit_working_copy(message, cx));
        cx.spawn(async move |this, cx| {
            if task.await.is_ok() {
                let _ = this.update(cx, |view, cx| {
                    view.commit_input.update(cx, |input, cx| input.clear(cx));
                });
            }
        })
        .detach();
    }

    pub fn select_file(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.active_pane = ActivePane::FileColumn;
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
        let base = self.scrolls.diff.0.borrow().base_handle.clone();
        let offset = base.offset();
        base.set_offset(point(offset.x, px(0.)));
        self.scrolls
            .diff
            .scroll_to_item_strict(0, ScrollStrategy::Top);
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

    pub fn toggle_annotate(&mut self, cx: &mut Context<Self>) {
        let vm = self.vm.clone();
        vm.update(cx, |vm, cx| vm.toggle_annotate(cx));
    }

    pub fn load_more(&mut self, cx: &mut Context<Self>) {
        let vm = self.vm.clone();
        vm.update(cx, |vm, cx| vm.load_more(cx));
    }
    // ----- UI: copy feedback -----

    pub fn mark_copied(&mut self, id: SharedString, cx: &mut Context<Self>) {
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

    // ----- UI: toast -----

    pub fn show_toast(&mut self, message: impl Into<SharedString>, cx: &mut Context<Self>) {
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

    pub fn show_coming_soon(&mut self, label: &str, cx: &mut Context<Self>) {
        self.show_toast(format!("{label} — coming soon"), cx);
    }

    // ----- UI: tree fold -----

    pub fn toggle_dir(&mut self, path: String, cx: &mut Context<Self>) {
        if !self.collapsed_dirs.remove(&path) {
            self.collapsed_dirs.insert(path);
        }
        cx.notify();
    }

    // ----- UI: file review -----

    pub fn toggle_reviewed(
        &mut self,
        change_id: String,
        path: String,
        identity: String,
        cx: &mut Context<Self>,
    ) {
        self.review_store
            .borrow_mut()
            .toggle(&change_id, &path, &identity);
        cx.notify();
    }

    pub fn is_reviewed(&self, change_id: &str, path: &str, identity: &str) -> bool {
        self.review_store
            .borrow()
            .is_reviewed(change_id, path, identity)
    }

    pub fn toggle_reviewed_for_selected_file(&mut self, cx: &mut Context<Self>) {
        let (change_id, path, identity, files) = {
            let vm = self.vm.read(cx);
            if vm.compare.is_some() {
                return;
            }
            // Review state is working-copy only.
            let change = match vm.selected_change() {
                Some(c) if c.is_working_copy => c,
                _ => return,
            };
            let change_id = change.change_id.clone();
            let hunk = match vm.selected_hunk() {
                Some(h) => h,
                None => return,
            };
            let path = hunk.path.clone();
            let identity = hunk.review_identity.clone();
            let files: Vec<(String, String)> = vm
                .files
                .as_ref()
                .map(|f| {
                    f.iter()
                        .map(|h| (h.path.clone(), h.review_identity.clone()))
                        .collect()
                })
                .unwrap_or_default();
            (change_id, path, identity, files)
        };
        // Advance to the first unreviewed file only when we just marked one reviewed.
        let next_ix = {
            let mut store = self.review_store.borrow_mut();
            store.toggle(&change_id, &path, &identity);
            store
                .is_reviewed(&change_id, &path, &identity)
                .then(|| {
                    files
                        .iter()
                        .position(|(p, id)| !store.is_reviewed(&change_id, p, id))
                })
                .flatten()
        };
        if let Some(next_ix) = next_ix {
            self.select_file(next_ix, cx);
            self.scrolls
                .files
                .scroll_to_item(next_ix, ScrollStrategy::Center);
            return;
        }
        cx.notify();
    }
}
