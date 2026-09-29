use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

use flume::Sender;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

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
        let repo_root =
            std::fs::canonicalize(repo_path).unwrap_or_else(|_| repo_path.to_path_buf());
        let op_heads_dir = op_heads_dir(&repo_root);
        let classifier = PathClassifier {
            op_heads_dir: op_heads_dir.clone(),
            repo_root: repo_root.clone(),
        };
        let debounce = Mutex::new(Debounce::fresh());

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            let Ok(event) = res else {
                return;
            };
            if let Some(out) = next_event(
                &classifier,
                &debounce,
                &event,
                Instant::now(),
                is_relevant_wc_change.as_ref(),
            ) {
                let _ = tx.send(out);
            }
        })
        .ok()?;

        watcher.watch(&repo_root, RecursiveMode::Recursive).ok()?;
        if !op_heads_dir.starts_with(&repo_root) {
            watcher
                .watch(&op_heads_dir, RecursiveMode::NonRecursive)
                .ok()?;
        }
        Some(Self { _watcher: watcher })
    }
}

/// Operations land in the primary repo; a secondary workspace's `.jj/repo` is only a pointer to it.
fn op_heads_dir(repo_path: &Path) -> PathBuf {
    let heads = Path::new(".jj/repo/op_heads/heads");
    let local = repo_path.join(heads);
    let Some(primary) = jayjay_core::workspace_primary_root(&repo_path.to_string_lossy()) else {
        return local;
    };
    let is_primary = std::fs::canonicalize(&primary)
        .ok()
        .zip(std::fs::canonicalize(repo_path).ok())
        .is_some_and(|(primary, repo)| primary == repo);
    if is_primary {
        local
    } else {
        PathBuf::from(primary).join(heads)
    }
}
