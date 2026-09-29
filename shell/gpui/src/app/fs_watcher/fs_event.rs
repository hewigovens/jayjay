use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsEvent {
    OpHeads,
    WorkingCopy,
}

pub(crate) type IsRelevantWcChange = Arc<dyn Fn(&[PathBuf]) -> bool + Send + Sync>;
