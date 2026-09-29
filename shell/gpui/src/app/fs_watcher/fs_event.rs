use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsEvent {
    OpHeads,
    WorkingCopy,
}

/// Returns `true` if any path is unignored. Mirrors `hasUnignoredWorkingCopyPaths` in SwiftUI.
pub(crate) type IsRelevantWcChange = Arc<dyn Fn(&[PathBuf]) -> bool + Send + Sync>;
