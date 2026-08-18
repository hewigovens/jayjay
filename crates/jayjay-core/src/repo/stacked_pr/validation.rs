use crate::types::{ChangeInfo, CoreError, CoreResult};

pub(super) fn validate_stack_changes(changes: &[ChangeInfo]) -> CoreResult<()> {
    for (i, change) in changes.iter().enumerate() {
        if change.is_immutable {
            return Err(CoreError::Internal {
                message: "The stack contains an immutable change.".to_owned(),
            });
        }
        if change.is_divergent {
            return Err(CoreError::Internal {
                message: "The stack contains a divergent change. Resolve the divergence (abandon the duplicate) before submitting."
                    .to_owned(),
            });
        }
        if change.parents.len() != 1 || (i > 0 && change.parents[0] != changes[i - 1].commit_id.id)
        {
            return Err(CoreError::Internal {
                message: "The stack must be linear (no merges).".to_owned(),
            });
        }
    }
    Ok(())
}
