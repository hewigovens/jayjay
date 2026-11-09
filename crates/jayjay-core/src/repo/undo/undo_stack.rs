use jj_lib::object_id::ObjectId as _;
use jj_lib::op_store::OperationId;
use jj_lib::operation::Operation;
use jj_lib::repo::ReadonlyRepo;

use super::super::Repo;
use super::super::support::block_on_result;
use super::op_log::shorten_embedded_ids;
use crate::types::*;

// The CLI's descriptions, so `jj undo` and JayJay continue each other's stacks.
const UNDO_PREFIX: &str = "undo: restore to operation ";
const REDO_PREFIX: &str = "redo: restore to operation ";

struct RestoreStep {
    description: String,
    restore: Operation,
}

type StepFinder = fn(&Repo, &ReadonlyRepo) -> JayResult<Option<RestoreStep>>;

impl Repo {
    /// What undo and redo would act on from the loaded operation, without snapshotting.
    pub fn undo_state(&self) -> UndoState {
        let repo = self.get_repo();
        let describe =
            |step: JayResult<Option<RestoreStep>>| step.ok().flatten().map(|step| step.description);
        UndoState {
            operation_id: repo.op_id().hex(),
            undo_description: describe(self.undo_step(&repo)),
            redo_description: describe(self.redo_step(&repo)),
        }
    }

    /// `jj undo`, returning what it undid, or `None` when the repo already moved past `operation_id` (a snapshot of new edits counts).
    pub fn undo(&self, operation_id: &str) -> JayResult<Option<String>> {
        self.step_from(
            operation_id,
            UNDO_PREFIX,
            Self::undo_step,
            jayjay_i18n::repo::nothing_to_undo,
        )
    }

    /// `jj redo`, returning what it restored, or `None` when the repo already moved past `operation_id`.
    pub fn redo(&self, operation_id: &str) -> JayResult<Option<String>> {
        self.step_from(
            operation_id,
            REDO_PREFIX,
            Self::redo_step,
            jayjay_i18n::repo::nothing_to_redo,
        )
    }

    fn step_from(
        &self,
        operation_id: &str,
        prefix: &str,
        find: StepFinder,
        nothing: fn() -> String,
    ) -> JayResult<Option<String>> {
        let _write = self.write_guard()?;
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        if repo.op_id().hex() != operation_id {
            return Ok(None);
        }
        let step = find(self, &repo)?.ok_or_else(|| JayError::refused(nothing()))?;
        self.restore_operation_view(
            &step.restore,
            &format!("{prefix}{}", step.restore.id().hex()),
        )?;
        Ok(Some(step.description))
    }

    /// Undoing an undo goes one step further back instead of redoing.
    fn undo_step(&self, repo: &ReadonlyRepo) -> JayResult<Option<RestoreStep>> {
        let target = restored_operation(repo, repo.operation().clone(), UNDO_PREFIX)?;
        if !self.performed_here(&target) {
            return Ok(None);
        }
        let Some(parent) = single_parent(&target)? else {
            return Ok(None);
        };
        Ok(Some(RestoreStep {
            description: summary(&target),
            restore: restored_operation(repo, parent, UNDO_PREFIX)?,
        }))
    }

    fn redo_step(&self, repo: &ReadonlyRepo) -> JayResult<Option<RestoreStep>> {
        let target = restored_operation(repo, repo.operation().clone(), REDO_PREFIX)?;
        if !target.metadata().description.starts_with(UNDO_PREFIX) || !self.performed_here(&target)
        {
            return Ok(None);
        }
        let Some(parent) = single_parent(&target)? else {
            return Ok(None);
        };
        let restore = restored_operation(repo, parent, REDO_PREFIX)?;
        // Redoing the second of two undos brings back the state the first one restored.
        let shown = restored_operation(repo, restore.clone(), UNDO_PREFIX)?;
        Ok(Some(RestoreStep {
            description: summary(&shown),
            restore,
        }))
    }

    /// Like the CLI without `--allow-cross-workspace`: another workspace's operation is rarely the one meant.
    fn performed_here(&self, operation: &Operation) -> bool {
        operation
            .metadata()
            .workspace_name
            .as_ref()
            .is_none_or(|name| *name == self.workspace_name)
    }
}

/// The operation an undo or redo `operation` restored to, so stacks jump instead of chaining.
fn restored_operation(
    repo: &ReadonlyRepo,
    operation: Operation,
    prefix: &str,
) -> JayResult<Operation> {
    let Some(hex) = operation.metadata().description.strip_prefix(prefix) else {
        return Ok(operation);
    };
    let id = OperationId::try_from_hex(hex)
        .ok_or_else(|| JayError::internal(format!("malformed operation id in '{prefix}{hex}'")))?;
    block_on_result("load restored operation", repo.loader().load_operation(&id))
}

/// `None` for the root operation and for merges of concurrent operations, which the CLI also refuses to undo.
fn single_parent(operation: &Operation) -> JayResult<Option<Operation>> {
    let mut parents = block_on_result("load parent operation", operation.parents())?;
    Ok((parents.len() == 1).then(|| parents.remove(0)))
}

fn summary(operation: &Operation) -> String {
    shorten_embedded_ids(
        operation
            .metadata()
            .description
            .lines()
            .next()
            .unwrap_or(""),
    )
}
