use std::error::Error;
use std::fmt::Display;
use std::future::Future;
use std::path::Path;
use std::sync::Arc;

use futures::StreamExt as _;
use jj_lib::op_store::OperationId;
use jj_lib::op_walk;
use jj_lib::repo::ReadonlyRepo;
use jj_lib::workspace::Workspace;

use super::config::{default_settings, working_copy_factories};
use crate::types::*;

/// Shell worker threads (Swift's cooperative pool, GCD) give Rust 512 KiB of stack, which jj-lib's merge and rebase futures overflow; work entered with less than the red zone left moves to a grown stack, while the CLI's main thread pays nothing.
const WORKER_STACK_RED_ZONE: usize = 4 << 20;
const GROWN_STACK_SIZE: usize = 32 << 20;

pub(crate) fn on_worker_stack<R>(work: impl FnOnce() -> R) -> R {
    stacker::maybe_grow(WORKER_STACK_RED_ZONE, GROWN_STACK_SIZE, work)
}

pub(crate) fn block_on<F: Future>(future: F) -> F::Output {
    on_worker_stack(|| pollster::block_on(future))
}

pub(crate) fn block_on_result<T, E, F>(context: &str, future: F) -> CoreResult<T>
where
    E: Display,
    F: Future<Output = Result<T, E>>,
{
    block_on(future).map_err(|e| CoreError::Internal {
        message: format!("{context}: {e}"),
    })
}

pub(crate) fn load_workspace_internal(path: &Path, context: &str) -> CoreResult<Workspace> {
    load_workspace(path).map_err(|error| CoreError::Internal {
        message: format!("{context}: {error}"),
    })
}

pub(crate) fn load_workspace(path: &Path) -> Result<Workspace, String> {
    let settings = default_settings().map_err(|error| error.to_string())?;
    let store_factories = jj_lib::default_backend_factories::default_backend_factories();
    let wc_factories = working_copy_factories();
    Workspace::load(&settings, path, &store_factories, &wc_factories).map_err(|error| {
        let mut message = error.to_string();
        let mut source = error.source();
        while let Some(err) = source {
            message.push_str(": ");
            message.push_str(&err.to_string());
            source = err.source();
        }
        message
    })
}

pub(crate) fn load_repo_at_head(
    workspace: &Workspace,
    context: &str,
) -> CoreResult<Arc<ReadonlyRepo>> {
    block_on_result(context, workspace.repo_loader().load_at_head())
}

/// True when `ancestor` is `descendant`'s operation or one of its ancestors.
/// Lets `set_repo` reject a stale candidate that would roll the view backwards.
pub(crate) fn op_is_ancestor_of(
    descendant: &Arc<ReadonlyRepo>,
    ancestor: &OperationId,
) -> CoreResult<bool> {
    if descendant.op_id() == ancestor {
        return Ok(true);
    }
    let head = descendant.operation().clone();
    let ancestors = op_walk::walk_ancestors(std::slice::from_ref(&head));
    futures::pin_mut!(ancestors);
    block_on(async {
        while let Some(op) = ancestors.next().await {
            let op = op.map_err(|e| CoreError::Internal {
                message: format!("walk operation ancestors: {e}"),
            })?;
            if op.id() == ancestor {
                return Ok(true);
            }
        }
        Ok(false)
    })
}
