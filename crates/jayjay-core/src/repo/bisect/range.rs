use std::sync::Arc;

use futures::TryStreamExt as _;
use jj_lib::backend::CommitId;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::revset::ResolvedRevsetExpression;

use super::result::change_id_of;
use crate::repo::support::block_on_result;
use crate::types::*;

pub fn compose_bisect_range(good: &str, bad: &str, range: &str) -> JayResult<String> {
    let range = range.trim();
    if !range.is_empty() {
        return Ok(range.to_owned());
    }
    let good = good.trim();
    let bad = bad.trim();
    if good.is_empty() || bad.is_empty() {
        return Err(JayError::refused(jayjay_i18n::repo::bisect_enter_range()));
    }
    Ok(format!("({good})..({bad})"))
}

pub(super) fn collect_commits(
    repo: &ReadonlyRepo,
    expression: &Arc<ResolvedRevsetExpression>,
) -> JayResult<Vec<CommitId>> {
    let revset = expression
        .clone()
        .evaluate(repo)
        .map_err(|error| JayError::internal(format!("bisect revset: {error}")))?;
    block_on_result("bisect revset", revset.stream().try_collect())
}

pub(super) fn end_change_ids(
    repo: &ReadonlyRepo,
    expression: &Arc<ResolvedRevsetExpression>,
) -> JayResult<(Vec<String>, Vec<String>)> {
    let connected = expression.connected();
    let good = connected.parents().minus(&connected);
    let bad = expression.heads();
    Ok((change_ids(repo, &good)?, change_ids(repo, &bad)?))
}

pub(super) fn commit_visible(repo: &ReadonlyRepo, id: &CommitId) -> JayResult<bool> {
    let commit = match repo.store().get_commit(id) {
        Ok(commit) => commit,
        Err(_) => return Ok(false),
    };
    let hidden = block_on_result("bisect visibility", commit.is_hidden(repo))?;
    Ok(!hidden)
}

fn change_ids(
    repo: &ReadonlyRepo,
    expression: &Arc<ResolvedRevsetExpression>,
) -> JayResult<Vec<String>> {
    let ids = collect_commits(repo, expression)?;
    ids.iter()
        .map(|id| {
            let commit = repo
                .store()
                .get_commit(id)
                .map_err(|error| JayError::internal(format!("load bisect end: {error}")))?;
            Ok(change_id_of(&commit))
        })
        .collect()
}
