use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use flume::Sender;
use notify::event::{EventKind, ModifyKind};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

const OP_DEBOUNCE: Duration = Duration::from_millis(1000);
const WC_DEBOUNCE: Duration = Duration::from_millis(2000);

#[derive(Debug, Clone, Copy)]
pub enum FsEvent {
    OpHeads,
    WorkingCopy,
}

/// Returns `true` if any path is unignored. Mirrors `hasUnignoredWorkingCopyPaths` in SwiftUI.
pub type IsRelevantWcChange = Arc<dyn Fn(&[PathBuf]) -> bool + Send + Sync>;

pub struct RepoFsWatcher {
    _watcher: RecommendedWatcher,
}

impl RepoFsWatcher {
    pub fn new(
        repo_path: &Path,
        tx: Sender<FsEvent>,
        is_relevant_wc_change: IsRelevantWcChange,
    ) -> Option<Self> {
        // Match by prefix: jj writes one file per head under this dir.
        let op_heads_dir = repo_path.join(".jj/repo/op_heads/heads");
        let last_op = Arc::new(Mutex::new(Instant::now() - OP_DEBOUNCE));
        let last_wc = Arc::new(Mutex::new(Instant::now() - WC_DEBOUNCE));
        let repo_root: PathBuf = repo_path.to_path_buf();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            let Ok(event) = res else {
                return;
            };
            // Drop metadata-only events — Spotlight / Time Machine spam them on macOS.
            let interesting = matches!(
                &event.kind,
                EventKind::Create(_)
                    | EventKind::Remove(_)
                    | EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Name(_))
            );
            if !interesting {
                return;
            }
            let now = Instant::now();
            let touches_op_heads = event
                .paths
                .iter()
                .any(|p| p == &op_heads_dir || p.starts_with(&op_heads_dir));
            if touches_op_heads {
                let mut guard = last_op.lock().expect("op debounce lock");
                if now.duration_since(*guard) >= OP_DEBOUNCE {
                    *guard = now;
                    let _ = tx.send(FsEvent::OpHeads);
                }
                return;
            }
            // Skip `.jj/` internals — op_heads already handled above.
            let in_jj_dir = event
                .paths
                .iter()
                .all(|p| p.strip_prefix(&repo_root).is_ok_and(starts_with_dot_jj));
            if in_jj_dir {
                return;
            }
            // Skip gitignored paths so `cargo build` doesn't storm the watcher.
            if !is_relevant_wc_change(&event.paths) {
                return;
            }
            let mut guard = last_wc.lock().expect("wc debounce lock");
            if now.duration_since(*guard) >= WC_DEBOUNCE {
                *guard = now;
                let _ = tx.send(FsEvent::WorkingCopy);
            }
        })
        .ok()?;

        watcher.watch(repo_path, RecursiveMode::Recursive).ok()?;
        Some(Self { _watcher: watcher })
    }
}

fn starts_with_dot_jj(rel: &Path) -> bool {
    rel.components()
        .next()
        .map(|c| c.as_os_str() == ".jj")
        .unwrap_or(false)
}
