use gpui::Context;
use jayjay_core::default_revset_depth;

use super::super::RepoViewModel;

impl RepoViewModel {
    pub fn revset(&self) -> &str {
        &self.revset_filter.revset
    }

    pub fn apply_revset(&mut self, revset: &str, cx: &mut Context<Self>) {
        self.revset_filter.apply(revset);
        self.reload_revset(None, cx);
    }

    pub(crate) fn apply_revset_selecting(
        &mut self,
        revset: &str,
        commit_id: String,
        cx: &mut Context<Self>,
    ) {
        self.revset_filter.apply(revset);
        self.reload_revset(Some(commit_id), cx);
    }

    pub(crate) fn show_ancestors(
        &mut self,
        change_id: &str,
        commit_id: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.revset_filter.show_ancestors(change_id);
        self.reload_revset(commit_id, cx);
    }

    pub(crate) fn return_to_previous_revset(&mut self, cx: &mut Context<Self>) {
        self.revset_filter.back();
        self.reload_revset(None, cx);
    }

    fn reload_revset(&mut self, selecting: Option<String>, cx: &mut Context<Self>) {
        self.can_load_more = false;
        match selecting {
            Some(commit_id) => {
                self.refresh_preferring(false, Some((commit_id.clone(), commit_id)), cx);
            }
            None => self.refresh(false, cx),
        }
    }

    pub(crate) fn revset_depth(&self) -> Option<u32> {
        default_revset_depth(self.revset())
    }
}
