use gpui::Context;

use super::RepoWindow;

#[derive(Default)]
pub(crate) struct CommitBoxState {
    working_copy_change_id: Option<String>,
    working_copy_description: String,
}

impl CommitBoxState {
    fn should_replace(
        &mut self,
        change_id: String,
        description: &str,
        box_description: &str,
    ) -> bool {
        let identity_changed = self.working_copy_change_id.as_deref() != Some(change_id.as_str());
        let description_changed = self.working_copy_description != description;
        let box_is_clean = box_description.trim_end() == self.working_copy_description.trim_end();
        self.working_copy_description = description.to_owned();
        self.working_copy_change_id = Some(change_id);

        if !identity_changed && !description_changed {
            return false;
        }
        if identity_changed {
            box_is_clean || !description.is_empty()
        } else {
            box_is_clean
        }
    }
}

impl RepoWindow {
    /// When @ moves, replace a typed draft only if the new change has a real description.
    pub(crate) fn sync_commit_box_from_working_copy(&mut self, cx: &mut Context<Self>) {
        let Some((change_id, description)) = self
            .vm
            .read(cx)
            .working_copy_change()
            .map(|change| (change.change_id.id.clone(), change.description.clone()))
        else {
            return;
        };
        let box_description = self.commit_box_message(cx);
        if !self
            .commit_box
            .should_replace(change_id, &description, &box_description)
        {
            return;
        }

        let summary = jayjay_core::commit_message::summary(&description);
        let body = jayjay_core::commit_message::body(&description);
        self.summary_input
            .update(cx, |input, cx| input.set_text(summary, cx));
        self.description_input
            .update(cx, |input, cx| input.set_text(body, cx));
    }

    /// Summary + optional body joined into jj's one change description (summary\n\nbody).
    pub(super) fn commit_box_message(&self, cx: &Context<Self>) -> String {
        let summary = self.summary_input.read(cx).text();
        let description = self.description_input.read(cx).text();
        jayjay_core::commit_message::join(&summary, &description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_change_refreshes_clean_description_and_preserves_typed_draft() {
        let mut state = CommitBoxState::default();
        assert!(state.should_replace("change".to_owned(), "initial", ""));
        assert!(state.should_replace("change".to_owned(), "external", "initial"));
        assert!(!state.should_replace("change".to_owned(), "newer", "typed draft"));
    }
}
