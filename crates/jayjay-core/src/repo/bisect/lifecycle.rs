use std::sync::Arc;
use std::sync::atomic::Ordering;

use jj_lib::revset::ResolvedRevsetExpression;

use super::range::{collect_commits, end_change_ids};
use super::session::Session;
use crate::repo::Repo;
use crate::repo::support::block_on_result;
use crate::types::*;
use jj_lib::repo::Repo as _;

impl Repo {
    pub fn bisect_state(&self) -> Option<BisectState> {
        self.bisect_lock().as_ref().map(Session::state)
    }

    pub fn start_bisect(&self, range: &str) -> JayResult<BisectState> {
        let _write = self.write_guard()?;
        let _execution = self.bisect_execution_lock();
        self.ensure_idle()?;
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        let expression = self.resolve_revset(&repo, range)?;
        self.install(expression)
    }

    pub fn start_bisect_between(&self, first: &str, second: &str) -> JayResult<BisectState> {
        let _write = self.write_guard()?;
        let _execution = self.bisect_execution_lock();
        self.ensure_idle()?;
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        let first_commit = self.resolve_commit(&repo, first)?;
        let second_commit = self.resolve_commit(&repo, second)?;
        if first_commit.id() == second_commit.id() {
            return Err(JayError::refused(
                jayjay_i18n::repo::bisect_needs_two_changes(),
            ));
        }
        let first_is_older = block_on_ancestor(&repo, first_commit.id(), second_commit.id())?;
        let second_is_older = block_on_ancestor(&repo, second_commit.id(), first_commit.id())?;
        let (good, bad) = if first_is_older && !second_is_older {
            (first_commit, second_commit)
        } else if second_is_older && !first_is_older {
            (second_commit, first_commit)
        } else {
            return Err(JayError::refused(jayjay_i18n::repo::bisect_not_same_line()));
        };
        let expression = ResolvedRevsetExpression::commit(good.id().clone())
            .range(&ResolvedRevsetExpression::commit(bad.id().clone()));
        self.install(expression)
    }

    pub fn bisect_stop(&self) -> JayResult<()> {
        // A start must finish installing its session, and a test process must exit, before restoration.
        drop(self.bisect_execution_lock());
        let _write = self.write_guard()?;
        let session = self.bisect_lock().take();
        let Some(session) = session else {
            return Ok(());
        };
        if session.phase() != BisectPhase::Finished
            && !session.temps.is_empty()
            && let Err(error) = self.restore_target(&session.restore)
        {
            *self.bisect_lock() = Some(session);
            return Err(error);
        }
        Ok(())
    }

    pub fn shutdown_bisect(&self) -> JayResult<()> {
        self.bisect_closed.store(true, Ordering::SeqCst);
        drop(self.bisect_execution_lock());
        self.bisect_stop()
    }

    pub fn bisect_restart(&self) -> JayResult<BisectState> {
        let _write = self.write_guard()?;
        let _execution = self.bisect_execution_lock();
        self.ensure_bisect_open()?;
        let (good_ends, bad_ends, restore) = {
            let guard = self.bisect_lock();
            let session = guard
                .as_ref()
                .ok_or_else(|| JayError::refused("No bisect is running."))?;
            if session.phase() != BisectPhase::PausedRangeChanged {
                return Err(JayError::refused(
                    jayjay_i18n::repo::bisect_restart_unavailable(),
                ));
            }
            (
                session.good_ends.clone(),
                session.bad_ends.clone(),
                session.restore.clone(),
            )
        };
        let repo = self.get_repo();
        let good_ids = self.resolve_ids(&repo, &good_ends)?;
        let bad_ids = self.resolve_ids(&repo, &bad_ends)?;
        let expression = ResolvedRevsetExpression::commits(good_ids)
            .range(&ResolvedRevsetExpression::commits(bad_ids));
        let range = collect_commits(&repo, &expression)?;
        if range.is_empty() {
            return Err(JayError::refused("The bisect range is empty."));
        }
        self.restore_target(&restore)?;
        let restore = self.capture_restore(&range)?;
        let mut session = Session::new(range, good_ends, bad_ends, restore);
        if let Err(error) = self.advance(&mut session) {
            *self.bisect_lock() = None;
            return Err(error);
        }
        let state = session.state();
        *self.bisect_lock() = Some(session);
        Ok(state)
    }

    pub fn bisect_dismiss(&self) -> JayResult<()> {
        let _write = self.write_guard()?;
        let mut guard = self.bisect_lock();
        match guard.as_ref().map(Session::phase) {
            Some(BisectPhase::Finished) => {
                *guard = None;
                Ok(())
            }
            Some(_) => Err(JayError::refused(
                jayjay_i18n::repo::bisect_result_not_ready(),
            )),
            None => Err(JayError::refused("No bisect is running.")),
        }
    }

    fn install(&self, expression: Arc<ResolvedRevsetExpression>) -> JayResult<BisectState> {
        let repo = self.get_repo();
        let range = collect_commits(&repo, &expression)?;
        if range.is_empty() {
            return Err(JayError::refused("The bisect range is empty."));
        }
        let (good_ends, bad_ends) = end_change_ids(&repo, &expression)?;
        let restore = self.capture_restore(&range)?;
        let mut session = Session::new(range, good_ends, bad_ends, restore);
        self.advance(&mut session)?;
        let state = session.state();
        *self.bisect_lock() = Some(session);
        Ok(state)
    }

    fn ensure_idle(&self) -> JayResult<()> {
        self.ensure_bisect_open()?;
        if self.bisect_lock().is_some() {
            Err(JayError::refused("A bisect is already running."))
        } else {
            Ok(())
        }
    }

    fn ensure_bisect_open(&self) -> JayResult<()> {
        if self.bisect_closed.load(Ordering::SeqCst) {
            Err(JayError::Canceled)
        } else {
            Ok(())
        }
    }

    pub(super) fn resolve_ids(
        &self,
        repo: &Arc<jj_lib::repo::ReadonlyRepo>,
        change_ids: &[String],
    ) -> JayResult<Vec<jj_lib::backend::CommitId>> {
        change_ids
            .iter()
            .map(|change_id| Ok(self.resolve_commit(repo, change_id)?.id().clone()))
            .collect()
    }
}

fn block_on_ancestor(
    repo: &jj_lib::repo::ReadonlyRepo,
    ancestor: &jj_lib::backend::CommitId,
    descendant: &jj_lib::backend::CommitId,
) -> JayResult<bool> {
    block_on_result("bisect", repo.index().is_ancestor(ancestor, descendant))
}
