use crate::repo::Repo;
use crate::types::*;

use super::naming;
use super::validation::validate_stack_changes;

impl Repo {
    pub fn detect_stack(&self, base_rev: &str, tip_rev: &str) -> CoreResult<Stack> {
        let mut changes = self.log(&format!("{base_rev}..{tip_rev}"))?;
        if changes.is_empty() {
            return Err(CoreError::Internal {
                message: "No mutable changes above trunk to stack.".to_owned(),
            });
        }
        changes.reverse(); // bottom → top
        validate_stack_changes(&changes)?;

        // Keep preview local; submit_stack resolves the forge's authoritative default before mutation.
        let base_bookmark = self.default_pull_request_base();
        let mut layers: Vec<StackLayer> = Vec::with_capacity(changes.len());
        for (i, change) in changes.iter().enumerate() {
            let short_len = (change.change_id.short_len as usize).min(change.change_id.id.len());
            let change_id_short = change.change_id.id[..short_len].to_owned();
            let existing = change.bookmarks.first().cloned();
            let bookmark = existing.clone().unwrap_or_else(|| {
                naming::bookmark_name(
                    &change.description,
                    &change.change_id.id,
                    change.change_id.short_len,
                )
            });
            let base = if i == 0 {
                base_bookmark.clone()
            } else {
                layers[i - 1].bookmark.clone()
            };
            layers.push(StackLayer {
                change_id: change.change_id.id.clone(),
                commit_id: change.commit_id.id.clone(),
                title: naming::first_line(&change.description),
                body: naming::body(&change.description),
                bookmark,
                base,
                bookmark_existed: existing.is_some(),
                change_id_short,
            });
        }
        Ok(Stack {
            layers,
            base_bookmark,
        })
    }
}
