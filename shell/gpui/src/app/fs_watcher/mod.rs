mod debounce;
mod fs_event;
mod path_classifier;
mod suppression;
mod watcher;

pub use fs_event::FsEvent;
pub(crate) use fs_event::IsRelevantWcChange;
pub(crate) use suppression::is_watcher_suppressed;
pub use suppression::suppress_for_tests;
pub use watcher::RepoFsWatcher;

#[cfg(test)]
mod tests;
