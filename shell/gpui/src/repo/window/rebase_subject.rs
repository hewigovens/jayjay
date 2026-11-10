use gpui::{Context, Task};
use jayjay_core::JayResult;

use super::RepoWindow;
use crate::repo::view_model::Rewrite;

/// What a rebase moves, as its toast names it.
pub(super) enum RebaseSubject {
    Change { label: String },
    Changes { count: usize },
}

impl RebaseSubject {
    /// A rebase of changes already in place records no operation, so it says so instead of offering Undo.
    fn message(&self, destination: &str, rebased: bool) -> String {
        match (self, rebased) {
            (Self::Change { label }, true) => jayjay_i18n::repo::rebased_onto(label, destination),
            (Self::Changes { count }, true) => jayjay_i18n::repo::rebased_onto(
                &jayjay_i18n::repo::change_count(*count),
                destination,
            ),
            (Self::Change { label }, false) => jayjay_i18n::repo::already_on(label, destination),
            (Self::Changes { count }, false) => {
                jayjay_i18n::repo::changes_already_on(*count, destination)
            }
        }
    }
}

impl RepoWindow {
    pub(super) fn notify_rebase<T: 'static>(
        &mut self,
        task: Task<JayResult<Rewrite<T>>>,
        subject: RebaseSubject,
        destination: String,
        cx: &mut Context<Self>,
    ) {
        self.notify_rewrite(
            task,
            move |rewrite| subject.message(&destination, rewrite.operation_id.is_some()),
            cx,
        );
    }
}
