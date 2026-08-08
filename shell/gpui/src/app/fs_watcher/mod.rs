use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use flume::Sender;
use gpui::{App, Global};
use notify::event::{EventKind, ModifyKind};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

const OP_DEBOUNCE: Duration = Duration::from_millis(1000);
const WC_DEBOUNCE: Duration = Duration::from_millis(2000);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsEvent {
    OpHeads,
    WorkingCopy,
}

/// Returns `true` if any path is unignored. Mirrors `hasUnignoredWorkingCopyPaths` in SwiftUI.
pub(crate) type IsRelevantWcChange = Arc<dyn Fn(&[PathBuf]) -> bool + Send + Sync>;

/// When set, the watcher is armed but the real `notify` OS thread isn't spawned — tests
/// install this so the FSEvents loop can't trip the GPUI scheduler's nondeterminism guard.
#[derive(Default)]
struct WatcherSuppressed(bool);

impl Global for WatcherSuppressed {}

/// True when the real OS watcher must not be spawned (test scheduler is active).
pub(crate) fn is_watcher_suppressed(cx: &App) -> bool {
    cx.try_global::<WatcherSuppressed>().is_some_and(|s| s.0)
}

/// Suppress real OS-thread watchers for the rest of this process. Tests call this.
pub fn suppress_for_tests(cx: &mut App) {
    cx.set_global(WatcherSuppressed(true));
}

/// What an FS event means before debounce, derived purely from its kind and paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventClass {
    /// Metadata-only / uninteresting, or a `.jj/` internal we already cover via op_heads.
    Ignore,
    /// A jj operation landed (op_heads changed) — refresh the graph.
    OpHeads,
    /// A working-copy path changed; relevance + debounce still gate the send.
    WorkingCopy,
}

/// Static per-watcher config used to classify each raw event.
struct PathClassifier {
    op_heads_dir: PathBuf,
    repo_root: PathBuf,
}

impl PathClassifier {
    fn classify(&self, event: &notify::Event) -> EventClass {
        // Metadata-only events — Spotlight / Time Machine spam them on macOS.
        let interesting = matches!(
            &event.kind,
            EventKind::Create(_)
                | EventKind::Remove(_)
                | EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Name(_))
        );
        if !interesting {
            return EventClass::Ignore;
        }
        let touches_op_heads = event
            .paths
            .iter()
            .any(|p| p == &self.op_heads_dir || p.starts_with(&self.op_heads_dir));
        if touches_op_heads {
            return EventClass::OpHeads;
        }
        // Other `.jj/` internals are already handled via op_heads above.
        let in_jj_dir = event.paths.iter().all(|p| {
            p.strip_prefix(&self.repo_root)
                .is_ok_and(starts_with_dot_jj)
        });
        if in_jj_dir {
            return EventClass::Ignore;
        }
        EventClass::WorkingCopy
    }
}

/// Leading-edge debounce timestamps for the two event streams.
struct Debounce {
    last_op: Instant,
    last_wc: Instant,
}

impl Debounce {
    fn fresh() -> Self {
        Self {
            last_op: Instant::now() - OP_DEBOUNCE,
            last_wc: Instant::now() - WC_DEBOUNCE,
        }
    }

    /// Whether enough time has elapsed to emit another op-heads refresh.
    fn op_ready(&self, now: Instant) -> bool {
        now.duration_since(self.last_op) >= OP_DEBOUNCE
    }

    /// Whether enough time has elapsed to emit another working-copy refresh.
    fn wc_ready(&self, now: Instant) -> bool {
        now.duration_since(self.last_wc) >= WC_DEBOUNCE
    }
}

pub struct RepoFsWatcher {
    _watcher: RecommendedWatcher,
}

impl RepoFsWatcher {
    pub(crate) fn new(
        repo_path: &Path,
        tx: Sender<FsEvent>,
        is_relevant_wc_change: IsRelevantWcChange,
    ) -> Option<Self> {
        let classifier = PathClassifier {
            // Match by prefix: jj writes one file per head under this dir.
            op_heads_dir: repo_path.join(".jj/repo/op_heads/heads"),
            repo_root: repo_path.to_path_buf(),
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

        watcher.watch(repo_path, RecursiveMode::Recursive).ok()?;
        Some(Self { _watcher: watcher })
    }
}

/// Decide whether a raw event should emit, stamping the debounce on a send. The window is
/// checked before the relevance filter so a build storm is dropped without running the
/// gitignore matcher more than once per `WC_DEBOUNCE`.
fn next_event(
    classifier: &PathClassifier,
    debounce: &Mutex<Debounce>,
    event: &notify::Event,
    now: Instant,
    is_relevant_wc_change: &dyn Fn(&[PathBuf]) -> bool,
) -> Option<FsEvent> {
    match classifier.classify(event) {
        EventClass::Ignore => None,
        EventClass::OpHeads => {
            let mut guard = debounce.lock().expect("op debounce lock");
            guard.op_ready(now).then(|| {
                guard.last_op = now;
                FsEvent::OpHeads
            })
        }
        EventClass::WorkingCopy => {
            // Read-only window check first; bail before touching the gitignore matcher.
            {
                let guard = debounce.lock().expect("wc debounce lock");
                if !guard.wc_ready(now) {
                    return None;
                }
            }
            if !is_relevant_wc_change(&event.paths) {
                return None;
            }
            let mut guard = debounce.lock().expect("wc debounce lock");
            // Re-check under the lock: a concurrent event may have stamped it meanwhile.
            guard.wc_ready(now).then(|| {
                guard.last_wc = now;
                FsEvent::WorkingCopy
            })
        }
    }
}

fn starts_with_dot_jj(rel: &Path) -> bool {
    rel.components()
        .next()
        .map(|c| c.as_os_str() == ".jj")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests;
