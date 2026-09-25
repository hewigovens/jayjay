use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};

use parking_lot::{ReentrantMutex, ReentrantMutexGuard};

use super::Repo;
use crate::types::CoreResult;

/// One lock per repository store, shared by every `Repo` opened on it; reentrant because mutations call other mutations.
static LOCKS: LazyLock<Mutex<HashMap<PathBuf, Arc<ReentrantMutex<()>>>>> =
    LazyLock::new(Mutex::default);

pub(super) fn for_store(store: &Path) -> Arc<ReentrantMutex<()>> {
    LOCKS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .entry(store.to_owned())
        .or_default()
        .clone()
}

impl Repo {
    /// Never take it on a thread that a holder waits for. The outermost acquisition reloads, so a mutation queued behind another `Repo` builds on its result.
    pub(crate) fn write_guard(&self) -> CoreResult<ReentrantMutexGuard<'_, ()>> {
        let reentered = self.write_lock.is_owned_by_current_thread();
        let guard = self.write_lock.lock();
        if !reentered {
            self.reload()?;
        }
        Ok(guard)
    }

    /// Called where mutations publish, so a public mutation that forgot `write_guard()` fails in tests.
    pub(crate) fn debug_assert_write_guarded(&self) {
        debug_assert!(
            self.write_lock.is_owned_by_current_thread(),
            "repository mutation without write_guard(); take it at the public entry point"
        );
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use jj_test::{init_jj_repo, run_jj_in};

    use crate::repo::Repo;

    #[test]
    fn a_second_window_waits_for_a_running_mutation_and_builds_on_it() {
        let temp_dir = init_jj_repo();
        let repo_path = temp_dir.path().join("repo");
        let first = Repo::open(&repo_path).expect("open first window");
        let second = Repo::open(&repo_path).expect("open second window");

        let guard = first.write_guard().expect("lock");
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                let result = second.describe("@", "second");
                done_tx.send(()).expect("report");
                result
            });
            assert!(
                done_rx.recv_timeout(Duration::from_millis(300)).is_err(),
                "the second window must wait while the first holds the lock"
            );
            first
                .describe("@", "first")
                .expect("the holder's own mutation re-enters the lock");
            drop(guard);
            done_rx.recv().expect("second mutation finishes");
        });

        let heads = second.log("@").expect("log @");
        assert_eq!(heads.len(), 1);
        assert_eq!(heads[0].description.trim(), "second");
        assert!(second.log("divergent()").expect("log divergent").is_empty());
    }

    #[test]
    fn a_window_opened_earlier_builds_on_another_windows_bookmark_move() {
        let temp_dir = init_jj_repo();
        let repo_path = temp_dir.path().join("repo");
        run_jj_in(&repo_path, &["describe", "-m", "a"]);
        run_jj_in(&repo_path, &["new", "-m", "b"]);
        run_jj_in(&repo_path, &["bookmark", "create", "main", "-r", "@-"]);
        let first = Repo::open(&repo_path).expect("open first window");
        let second = Repo::open(&repo_path).expect("open second window");

        first.move_bookmark("main", "@").expect("first move");
        second.move_bookmark("main", "@-").expect("second move");

        let bookmarks = second.list_bookmarks().expect("bookmarks");
        let main = bookmarks.iter().find(|b| b.name == "main").expect("main");
        assert!(
            !main.is_conflicted,
            "the second move must build on the first"
        );
        assert_eq!(
            second.log("main").expect("log main")[0].description.trim(),
            "a"
        );
    }
}
