use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use flume::{Receiver, RecvTimeoutError, Sender};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

use super::debounce::{Debounce, next_event};
use super::fs_event::{FsEvent, IsRelevantWcChange};
use super::path_classifier::PathClassifier;

pub struct RepoFsWatcher {
    _watcher: RecommendedWatcher,
}

impl RepoFsWatcher {
    pub(crate) fn new(
        repo_path: &Path,
        tx: Sender<FsEvent>,
        is_relevant_wc_change: IsRelevantWcChange,
    ) -> Option<Self> {
        // Watchers report real paths, so classify against the canonical root or every event under a symlinked checkout looks foreign.
        let repo_root = dunce::canonicalize(repo_path).unwrap_or_else(|_| repo_path.to_path_buf());
        let op_heads_dir = op_heads_dir(&repo_root);
        let classifier = PathClassifier {
            op_heads_dir: op_heads_dir.clone(),
            repo_root: repo_root.clone(),
        };
        let debounce = Arc::new(Mutex::new(Debounce::fresh()));
        let (wake_tx, wake_rx) = flume::bounded::<()>(1);
        spawn_flusher(
            debounce.clone(),
            wake_rx,
            tx.clone(),
            is_relevant_wc_change.clone(),
        );

        let handler = move |res: notify::Result<notify::Event>| {
            let Ok(event) = res else {
                return;
            };
            match next_event(
                &classifier,
                &debounce,
                &event,
                Instant::now(),
                is_relevant_wc_change.as_ref(),
            ) {
                Some(out) => {
                    let _ = tx.send(out);
                }
                None => {
                    let _ = wake_tx.try_send(());
                }
            }
        };
        // pnpm's node_modules symlinks make the inotify walk revisit the same trees.
        let config = Config::default().with_follow_symlinks(false);
        let mut watcher = RecommendedWatcher::new(handler, config).ok()?;

        watcher.watch(&repo_root, RecursiveMode::Recursive).ok()?;
        if !op_heads_dir.starts_with(&repo_root) {
            watcher
                .watch(&op_heads_dir, RecursiveMode::NonRecursive)
                .ok()?;
        }
        Some(Self { _watcher: watcher })
    }
}

/// Replays events held by the debounce window; exits once the watcher's handler, and with it `wake`, is dropped.
fn spawn_flusher(
    debounce: Arc<Mutex<Debounce>>,
    wake: Receiver<()>,
    tx: Sender<FsEvent>,
    is_relevant_wc_change: IsRelevantWcChange,
) {
    std::thread::spawn(move || {
        loop {
            let due = debounce.lock().expect("debounce lock").next_flush();
            let woke = match due {
                Some(due) => wake.recv_deadline(due),
                None => wake.recv().map_err(|_| RecvTimeoutError::Disconnected),
            };
            if woke == Err(RecvTimeoutError::Disconnected) {
                return;
            }
            let events = debounce
                .lock()
                .expect("debounce lock")
                .flush(Instant::now(), is_relevant_wc_change.as_ref());
            if events.into_iter().any(|event| tx.send(event).is_err()) {
                return;
            }
        }
    });
}

/// Operations land in the primary repo; a secondary workspace's `.jj/repo` is only a pointer to it.
fn op_heads_dir(repo_path: &Path) -> PathBuf {
    let heads = Path::new(".jj/repo/op_heads/heads");
    let local = repo_path.join(heads);
    let Some(primary) = jayjay_core::workspace_primary_root(&repo_path.to_string_lossy()) else {
        return local;
    };
    let is_primary = dunce::canonicalize(&primary)
        .ok()
        .zip(dunce::canonicalize(repo_path).ok())
        .is_some_and(|(primary, repo)| primary == repo);
    if is_primary {
        local
    } else {
        PathBuf::from(primary).join(heads)
    }
}
