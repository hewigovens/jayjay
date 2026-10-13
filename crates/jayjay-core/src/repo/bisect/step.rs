use futures::TryStreamExt as _;
use jj_lib::backend::CommitId;
use jj_lib::bisect::{Bisector, NextStep};
use jj_lib::object_id::ObjectId as _;
use jj_lib::repo::Repo as JjRepo;
use jj_lib::revset::{ResolvedRevsetExpression, Revset};

use super::range::commit_visible;
use super::session::{Session, StoredMark, ToEvaluation};
use crate::Repo;
use crate::repo::support::{block_on_result, short_commit_id};
use crate::types::*;

impl Repo {
    pub fn bisect_mark(&self, commit_id: &str, verdict: BisectVerdict) -> JayResult<BisectState> {
        let _write = self.write_guard()?;
        let Some(mut session) = self.bisect_lock().clone() else {
            return Err(JayError::refused("No bisect is running."));
        };
        let result = self.mark_and_advance(&mut session, commit_id, verdict);
        *self.bisect_lock() = Some(session);
        result
    }

    pub(super) fn mark_and_advance(
        &self,
        session: &mut Session,
        commit_id: &str,
        verdict: BisectVerdict,
    ) -> JayResult<BisectState> {
        if session.phase() != BisectPhase::Evaluating {
            return Err(JayError::refused("There is no candidate to mark."));
        }
        let id = session.guard_candidate(commit_id)?;
        if !self.range_intact(session)? {
            session.pause();
            return Ok(session.state());
        }
        self.ensure_candidate_checkout(session)?;
        session.push_mark(id, verdict.to_evaluation());
        self.advance_after_mark(session)
    }

    pub(super) fn advance_after_mark(&self, session: &mut Session) -> JayResult<BisectState> {
        if let Err(error) = self.advance(session) {
            session.pop_mark();
            return Err(error);
        }
        if session.phase() == BisectPhase::PausedRangeChanged {
            session.pop_mark();
        }
        Ok(session.state())
    }

    pub(super) fn advance(&self, session: &mut Session) -> JayResult<()> {
        if !self.range_intact(session)? {
            session.pause();
            return Ok(());
        }
        let repo = self.get_repo();
        let mut bisector = build_bisector(repo.as_ref(), &session.range, &session.marks)?;
        let remaining = block_on_result("bisect remaining", bisector.remaining_revset())?;
        let focused = commit_hexes(&*remaining)?;
        drop(remaining);
        session.set_remaining(focused);
        let step = block_on_result("bisect next", bisector.next_step())?;
        drop(bisector);
        match step {
            NextStep::Verify { .. } => Err(JayError::internal(
                "bisect precondition verification is not used",
            )),
            NextStep::Evaluate(commit) => {
                let temp = self.checkout_candidate(commit.id(), &session.range)?;
                session.temps.push(temp);
                session.set_candidate(commit);
                Ok(())
            }
            NextStep::Done(result) => self.finish(session, result),
        }
    }

    pub(super) fn range_intact(&self, session: &Session) -> JayResult<bool> {
        let repo = self.get_repo();
        for id in session
            .range
            .iter()
            .chain(session.marks.iter().map(|mark| &mark.id))
        {
            if !commit_visible(&repo, id)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn ensure_candidate_checkout(&self, session: &Session) -> JayResult<()> {
        let repo = self.get_repo();
        let working_copy = self.working_copy_commit(&repo)?;
        let change_id = super::result::change_id_of(&working_copy);
        if session.temps.last() == Some(&change_id)
            && session.candidate.as_ref().is_some_and(|candidate| {
                working_copy.parent_ids() == std::slice::from_ref(candidate.id())
            })
        {
            Ok(())
        } else {
            Err(JayError::refused(
                "The working copy moved away from the bisect candidate. Stop and start a new bisect.",
            ))
        }
    }

    fn checkout_candidate(&self, commit_id: &CommitId, range: &[CommitId]) -> JayResult<String> {
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        let commit = repo
            .store()
            .get_commit(commit_id)
            .map_err(|error| JayError::internal(format!("load bisect candidate: {error}")))?;
        let preserve_working_copy = range.contains(self.working_copy_commit(&repo)?.id());
        let label = short_commit_id(repo.as_ref(), &commit).unique_prefix();
        let description = format!("bisect: test {label}");
        self.with_repo_transaction(&description, true, |_, repo_mut| {
            let new_commit = repo_mut
                .new_commit(vec![commit.id().clone()], commit.tree())
                .set_description("")
                .write();
            let new_commit = block_on_result("bisect checkout", new_commit)?;
            if preserve_working_copy {
                // `edit` would abandon a discardable working copy that is still part of the bisect range.
                repo_mut
                    .set_wc_commit(self.workspace_name.clone(), new_commit.id().clone())
                    .map_err(|error| JayError::internal(format!("bisect checkout: {error}")))
            } else {
                self.edit_working_copy_commit(repo_mut, &new_commit, "bisect checkout")
            }
        })?;
        let repo = self.get_repo();
        let working_copy = self.working_copy_commit(&repo)?;
        Ok(super::result::change_id_of(&working_copy))
    }
}

fn build_bisector<'repo>(
    repo: &'repo dyn JjRepo,
    range: &[CommitId],
    marks: &[StoredMark],
) -> JayResult<Bisector<'repo>> {
    let range = ResolvedRevsetExpression::commits(range.to_vec());
    let mut bisector = block_on_result("bisect", Bisector::new(repo, range, false))?;
    for mark in marks {
        bisector.mark(mark.id.clone(), mark.evaluation.clone());
    }
    Ok(bisector)
}

fn commit_hexes(revset: &dyn Revset) -> JayResult<Vec<String>> {
    let ids: Vec<CommitId> = block_on_result("bisect remaining", revset.stream().try_collect())?;
    Ok(ids.iter().map(|id| id.hex()).collect())
}
