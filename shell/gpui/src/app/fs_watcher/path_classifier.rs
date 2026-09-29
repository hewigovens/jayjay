use std::path::{Path, PathBuf};

use notify::event::{EventKind, ModifyKind};

/// What an FS event means before debounce, derived purely from its kind and paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EventClass {
    /// Metadata-only / uninteresting, or a `.jj/` internal we already cover via op_heads.
    Ignore,
    /// A jj operation landed (op_heads changed) — refresh the graph.
    OpHeads,
    /// A working-copy path changed; relevance + debounce still gate the send.
    WorkingCopy,
}

/// Static per-watcher config used to classify each raw event.
pub(super) struct PathClassifier {
    pub(super) op_heads_dir: PathBuf,
    pub(super) repo_root: PathBuf,
}

impl PathClassifier {
    pub(super) fn classify(&self, event: &notify::Event) -> EventClass {
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

fn starts_with_dot_jj(rel: &Path) -> bool {
    rel.components()
        .next()
        .map(|c| c.as_os_str() == ".jj")
        .unwrap_or(false)
}
