use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

use jayjay_core::diff::FileDiff;
use jayjay_core::external_tools::{
    ExternalDiffFile as CoreExternalDiffFile, ExternalDiffSelection, ExternalDiffSide,
    apply_external_diff_selections, diff_edit_ranges, load_external_diff,
};
use jayjay_core::{CoreResult, DiffEditFileSelection, DiffHunk};

pub(super) struct ExternalDiffSession {
    pub left: PathBuf,
    pub right: PathBuf,
    pub editable: bool,
    pub files: Vec<ExternalDiffFile>,
    pub selected_file: usize,
}

pub(super) struct ExternalDiffSave {
    left: PathBuf,
    right: PathBuf,
    selections: Vec<ExternalDiffSelection>,
}

impl ExternalDiffSession {
    pub fn load(left: PathBuf, right: PathBuf, editable: bool) -> CoreResult<Self> {
        let files = load_external_diff(&left, &right, editable)?
            .into_iter()
            .map(ExternalDiffFile::new)
            .collect();
        Ok(Self {
            left,
            right,
            editable,
            files,
            selected_file: 0,
        })
    }

    pub fn save_request(&self) -> ExternalDiffSave {
        let selections: Vec<_> = self
            .files
            .iter()
            .filter_map(ExternalDiffFile::selection)
            .collect();
        ExternalDiffSave {
            left: self.left.clone(),
            right: self.right.clone(),
            selections,
        }
    }

    pub fn selected(&self) -> Option<&ExternalDiffFile> {
        self.files.get(self.selected_file)
    }

    pub fn selected_mut(&mut self) -> Option<&mut ExternalDiffFile> {
        self.files.get_mut(self.selected_file)
    }

    pub fn toggle_selected_file(&mut self) {
        let Some(file) = self.selected() else {
            return;
        };
        let side = if file.keeps_all_changes() {
            ExternalDiffSide::Old
        } else {
            ExternalDiffSide::New
        };
        let group = file.topology_group.clone();
        for (index, file) in self.files.iter_mut().enumerate() {
            if index == self.selected_file
                || group
                    .as_ref()
                    .is_some_and(|group| file.topology_group.as_ref() == Some(group))
            {
                file.select_side(side);
            }
        }
    }
}

impl ExternalDiffSave {
    pub fn run(self) -> CoreResult<()> {
        apply_external_diff_selections(&self.left, &self.right, &self.selections, false)
    }
}

pub(super) struct ExternalDiffFile {
    pub hunk: DiffHunk,
    pub topology_group: Option<String>,
    pub display: Arc<FileDiff>,
    pub display_to_full: HashMap<usize, u32>,
    pub changed: BTreeSet<u32>,
    pub selected: BTreeSet<u32>,
    pub supports_editing: bool,
    pub old_exists: bool,
    pub new_exists: bool,
    pub selected_exists: bool,
    pub old_executable: Option<bool>,
    pub new_executable: Option<bool>,
    pub selected_executable: Option<bool>,
    pub whole_file_side: Option<ExternalDiffSide>,
}

impl ExternalDiffFile {
    fn new(file: CoreExternalDiffFile) -> Self {
        let display_to_full = file
            .display_to_full
            .into_iter()
            .map(|mapping| ((mapping.display_line - 1) as usize, mapping.full_line))
            .collect();
        let changed: BTreeSet<u32> = file.changed_lines.into_iter().collect();
        Self {
            hunk: file.hunk,
            topology_group: file.topology_group,
            display: Arc::new(file.display_diff),
            selected: changed.clone(),
            changed,
            display_to_full,
            supports_editing: file.supports_editing,
            old_exists: file.old_exists,
            new_exists: file.new_exists,
            selected_exists: file.new_exists,
            old_executable: file.old_executable,
            new_executable: file.new_executable,
            selected_executable: file.new_executable,
            whole_file_side: (!file.supports_editing).then_some(ExternalDiffSide::New),
        }
    }

    pub fn toggle_line(&mut self, display_index: usize) {
        let Some(full_line) = self.display_to_full.get(&display_index).copied() else {
            return;
        };
        if !self.selected.remove(&full_line) {
            self.selected.insert(full_line);
        }
        self.sync_exists_with_selection();
    }

    fn sync_exists_with_selection(&mut self) {
        if self.old_exists == self.new_exists {
            return;
        }
        self.selected_exists = if self.selected.is_empty() {
            self.old_exists
        } else if self.selected == self.changed {
            self.new_exists
        } else {
            true
        };
        self.selected_executable = if !self.selected_exists {
            None
        } else if self.new_exists {
            self.new_executable
        } else {
            self.old_executable
        };
    }

    fn select_side(&mut self, side: ExternalDiffSide) {
        if self.whole_file_side.is_some() {
            self.whole_file_side = Some(side);
        }
        match side {
            ExternalDiffSide::Old => {
                self.selected.clear();
                self.selected_exists = self.old_exists;
                self.selected_executable = self.old_executable;
            }
            ExternalDiffSide::New => {
                self.selected.clone_from(&self.changed);
                self.selected_exists = self.new_exists;
                self.selected_executable = self.new_executable;
            }
        }
    }

    pub fn executable_changed(&self) -> bool {
        matches!(
            (self.old_executable, self.new_executable),
            (Some(old), Some(new)) if old != new
        )
    }

    pub fn keeps_all_changes(&self) -> bool {
        if let Some(side) = self.whole_file_side {
            return side == ExternalDiffSide::New;
        }
        self.selected == self.changed
            && self.selected_exists == self.new_exists
            && self.selected_executable == self.new_executable
    }

    pub fn keeps_any_changes(&self) -> bool {
        if let Some(side) = self.whole_file_side {
            return side == ExternalDiffSide::New;
        }
        !self.selected.is_empty()
            || (self.old_exists != self.new_exists && self.selected_exists == self.new_exists)
            || (self.executable_changed() && self.selected_executable == self.new_executable)
    }

    pub fn is_display_line_selected(&self, index: usize) -> bool {
        self.display_to_full
            .get(&index)
            .is_some_and(|line| self.selected.contains(line))
    }

    pub fn selection(&self) -> Option<ExternalDiffSelection> {
        Some(ExternalDiffSelection {
            file: DiffEditFileSelection {
                path: self.hunk.path.clone(),
                old_path: self.hunk.old_path.clone(),
                old_content: self.hunk.old.content.clone(),
                new_content: self.hunk.new.content.clone(),
                hunk_type: self.hunk.hunk_type,
                line_ranges: diff_edit_ranges(self.selected.iter().copied().collect()),
            },
            selected_exists: self.selected_exists,
            selected_executable: self.selected_executable,
            whole_file_side: self.whole_file_side,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn line_selection_updates_added_and_deleted_file_existence() {
        let fixture = tempfile::tempdir().expect("fixture");
        let left = fixture.path().join("left");
        let right = fixture.path().join("right");
        fs::create_dir(&left).expect("left directory");
        fs::create_dir(&right).expect("right directory");
        fs::write(left.join("deleted.rs"), "fn deleted() {}\n").expect("deleted file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(left.join("deleted.rs"), fs::Permissions::from_mode(0o755))
                .expect("deleted executable mode");
        }
        fs::write(right.join("added.rs"), "fn added() {}\n").expect("added file");
        let mut session = ExternalDiffSession::load(left, right, true).expect("diff session");

        let added = session
            .files
            .iter_mut()
            .find(|file| file.hunk.path == "added.rs")
            .expect("added diff");
        let added_line = *added.display_to_full.keys().next().expect("added line");
        added.toggle_line(added_line);
        assert!(!added.selected_exists);

        let deleted = session
            .files
            .iter_mut()
            .find(|file| file.hunk.path == "deleted.rs")
            .expect("deleted diff");
        let deleted_line = *deleted.display_to_full.keys().next().expect("deleted line");
        deleted.toggle_line(deleted_line);
        assert!(deleted.selected_exists);
        #[cfg(unix)]
        assert_eq!(deleted.selected_executable, Some(true));
    }
}
