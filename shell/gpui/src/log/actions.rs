use gpui::{Context, ScrollStrategy, SharedString};

use super::{ActivePane, LogView};

impl LogView {
    pub fn select_change(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.active_pane = ActivePane::Sidebar;
        self.find_matches.clear();
        self.find_current = 0;
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
            self.changes_scroll
                .scroll_to_item(ix, ScrollStrategy::Center);
            self.select_change(ix, cx);
        }
    }

    pub fn select_file(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.active_pane = ActivePane::FileColumn;
        let vm = self.vm.clone();
        vm.update(cx, |vm, cx| vm.select_file(ix, cx));
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
        self.recently_copied = Some(id.clone());
        cx.notify();
        let id_for_clear = id;
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1500))
                .await;
            let _ = this.update(cx, move |view, cx| {
                if view.recently_copied.as_ref() == Some(&id_for_clear) {
                    view.recently_copied = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    // ----- UI: toast -----

    pub fn show_toast(&mut self, message: impl Into<SharedString>, cx: &mut Context<Self>) {
        let message = message.into();
        self.toast = Some(message.clone());
        cx.notify();
        let id_for_clear = message;
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(1800))
                .await;
            let _ = this.update(cx, move |view, cx| {
                if view.toast.as_ref() == Some(&id_for_clear) {
                    view.toast = None;
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

    pub fn toggle_reviewed(&mut self, change_id: String, path: String, cx: &mut Context<Self>) {
        self.review_store.toggle(&change_id, &path);
        cx.notify();
    }

    pub fn is_reviewed(&self, change_id: &str, path: &str) -> bool {
        self.review_store.is_reviewed(change_id, path)
    }

    pub fn toggle_reviewed_for_selected_file(&mut self, cx: &mut Context<Self>) {
        let (change_id, path, file_paths) = {
            let vm = self.vm.read(cx);
            // Review state is working-copy only.
            let change = match vm.selected_change() {
                Some(c) if c.is_working_copy => c,
                _ => return,
            };
            let change_id = change.change_id.clone();
            let path = match vm.selected_hunk() {
                Some(h) => h.path.clone(),
                None => return,
            };
            let file_paths: Vec<String> = vm
                .files
                .as_ref()
                .map(|f| f.iter().map(|h| h.path.clone()).collect())
                .unwrap_or_default();
            (change_id, path, file_paths)
        };
        self.review_store.toggle(&change_id, &path);
        // Advance to the first unreviewed file only when we just marked one reviewed.
        let now_reviewed = self.review_store.is_reviewed(&change_id, &path);
        if now_reviewed
            && let Some(next_ix) = file_paths
                .iter()
                .position(|p| !self.review_store.is_reviewed(&change_id, p))
        {
            self.select_file(next_ix, cx);
            self.files_scroll
                .scroll_to_item(next_ix, ScrollStrategy::Center);
            return;
        }
        cx.notify();
    }
}
