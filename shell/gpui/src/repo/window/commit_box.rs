use gpui::Context;

use super::RepoWindow;

#[derive(Default)]
pub(crate) struct CommitBoxState {
    working_copy_change_id: Option<String>,
    working_copy_description: String,
}

impl RepoWindow {
    /// When @ moves, replace a typed draft only if the new change has a real description.
    pub(crate) fn sync_commit_box_from_working_copy(&mut self, cx: &mut Context<Self>) {
        let Some((change_id, description, is_divergent)) =
            self.vm.read(cx).working_copy_change().map(|change| {
                (
                    change.change_id.id.clone(),
                    change.description.clone(),
                    change.is_divergent,
                )
            })
        else {
            return;
        };
        let identity_changed =
            self.commit_box.working_copy_change_id.as_deref() != Some(change_id.as_str());
        let description_changed = self.commit_box.working_copy_description != description;
        self.commit_box.working_copy_description = description.clone();

        if !identity_changed && !(is_divergent && description_changed) {
            return;
        }
        self.commit_box.working_copy_change_id = Some(change_id);
        let has_draft = !self.summary_input.read(cx).text().is_empty()
            || !self.description_input.read(cx).text().is_empty();
        if has_draft && description.is_empty() {
            return;
        }

        let summary = jayjay_core::commit_message::summary(&description);
        let body = jayjay_core::commit_message::body(&description);
        self.summary_input
            .update(cx, |input, cx| input.set_text(summary, cx));
        self.description_input
            .update(cx, |input, cx| input.set_text(body, cx));
    }
}
