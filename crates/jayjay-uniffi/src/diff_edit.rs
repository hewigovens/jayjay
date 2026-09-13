use std::sync::{Arc, Mutex};

use jayjay_core::diff_edit::{self, DiffEditFile, DiffEditFileDiff};
use jayjay_core::{DiffEditDestination, DiffEditFileSelection, FileDiffStats};

#[uniffi::export]
fn compute_diff_edit_file(
    path: String,
    old_content: String,
    new_content: String,
    ignore_whitespace: bool,
    highlight: bool,
) -> DiffEditFileDiff {
    DiffEditFileDiff::compute(
        &path,
        &old_content,
        &new_content,
        ignore_whitespace,
        highlight,
    )
}

#[derive(Default, uniffi::Object)]
pub struct DiffEditSession {
    inner: Mutex<diff_edit::DiffEditSession>,
}

impl DiffEditSession {
    fn read<T: Default>(&self, read: impl FnOnce(&diff_edit::DiffEditSession) -> T) -> T {
        self.inner
            .lock()
            .map(|session| read(&session))
            .unwrap_or_default()
    }

    fn write<T: Default>(&self, write: impl FnOnce(&mut diff_edit::DiffEditSession) -> T) -> T {
        self.inner
            .lock()
            .map(|mut session| write(&mut session))
            .unwrap_or_default()
    }
}

#[uniffi::export]
impl DiffEditSession {
    #[uniffi::constructor]
    fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn load(&self, file: DiffEditFile) {
        self.write(|session| session.load(file));
    }

    fn skip(&self, path: String) {
        self.write(|session| session.skip(&path));
    }

    fn unload(&self) {
        self.write(diff_edit::DiffEditSession::unload);
    }

    fn is_loaded(&self, path: String) -> bool {
        self.read(|session| session.is_loaded(&path))
    }

    fn selected_lines(&self, path: String) -> Vec<u32> {
        self.read(|session| session.selected_lines(&path))
    }

    fn toggle_line(&self, path: String, line: u32) {
        self.write(|session| session.toggle_line(&path, line));
    }

    fn toggle_file(&self, path: String) {
        self.write(|session| session.toggle_file(&path));
    }

    fn select_file(&self, path: String) {
        self.write(|session| session.select_file(&path));
    }

    fn select_lines(&self, path: String, lines: Vec<u32>) {
        self.write(|session| session.select_lines(&path, &lines));
    }

    fn toggle_all(&self, paths: Vec<String>) -> Vec<String> {
        self.write(|session| session.toggle_all(&paths))
    }

    fn is_selecting_all(&self) -> bool {
        self.read(diff_edit::DiffEditSession::is_selecting_all)
    }

    fn has_selection(&self) -> bool {
        self.read(diff_edit::DiffEditSession::has_selection)
    }

    fn should_deselect(&self) -> bool {
        self.read(diff_edit::DiffEditSession::should_deselect)
    }

    fn summary_text(&self) -> String {
        self.read(diff_edit::DiffEditSession::summary_text)
    }

    fn selections(
        &self,
        paths: Vec<String>,
        destination: DiffEditDestination,
    ) -> Vec<DiffEditFileSelection> {
        self.read(|session| session.selections(&paths, destination))
    }

    fn seed_collapse(&self, paths: Vec<String>, total_changed_lines: Option<u64>) {
        self.write(|session| session.seed_collapse(&paths, total_changed_lines));
    }

    fn apply_stats(&self, paths: Vec<String>, stats: Vec<FileDiffStats>) {
        self.write(|session| session.apply_stats(&paths, &stats));
    }

    fn toggle_collapse(&self, path: String) {
        self.write(|session| session.toggle_collapse(&path));
    }

    fn set_collapsed(&self, path: String, collapsed: bool) -> bool {
        self.write(|session| session.set_collapsed(&path, collapsed))
    }

    fn collapse_all(&self, paths: Vec<String>) {
        self.write(|session| session.collapse_all(&paths));
    }

    fn expand_all(&self) {
        self.write(diff_edit::DiffEditSession::expand_all);
    }

    fn collapsed_paths(&self, paths: Vec<String>) -> Vec<String> {
        self.read(|session| {
            paths
                .into_iter()
                .filter(|path| session.is_collapsed(path))
                .collect()
        })
    }

    fn set_focused(&self, path: String) {
        self.write(|session| session.set_focused(&path));
    }

    fn move_focus(&self, paths: Vec<String>, forward: bool) -> Option<String> {
        self.write(|session| session.move_focus(&paths, forward).map(str::to_owned))
    }
}
