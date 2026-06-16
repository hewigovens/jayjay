use std::fmt::Display;
use std::future::Future;
use std::path::Path;
use std::sync::Arc;

use futures::StreamExt as _;
use jj_lib::op_store::OperationId;
use jj_lib::op_walk;
use jj_lib::repo::ReadonlyRepo;
use jj_lib::workspace::Workspace;
use pollster::FutureExt as _;

use super::config::{default_settings, working_copy_factories};
use crate::types::*;

pub(crate) fn block_on_result<T, E, F>(context: &str, future: F) -> CoreResult<T>
where
    E: Display,
    F: Future<Output = Result<T, E>>,
{
    future.block_on().map_err(|e| CoreError::Internal {
        message: format!("{context}: {e}"),
    })
}

pub(crate) fn load_workspace_internal(path: &Path, context: &str) -> CoreResult<Workspace> {
    let settings = default_settings()?;
    let store_factories = jj_lib::repo::StoreFactories::default();
    let wc_factories = working_copy_factories();
    Workspace::load(&settings, path, &store_factories, &wc_factories).map_err(|e| {
        CoreError::Internal {
            message: format!("{context}: {e}"),
        }
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
    async {
        while let Some(op) = ancestors.next().await {
            let op = op.map_err(|e| CoreError::Internal {
                message: format!("walk operation ancestors: {e}"),
            })?;
            if op.id() == ancestor {
                return Ok(true);
            }
        }
        Ok(false)
    }
    .block_on()
}
