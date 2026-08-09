use std::{
    fs,
    path::{Path, PathBuf},
};

use super::*;
use crate::external_tools::{diff_edit_ranges, load_external_diff};
use crate::{DiffEditFileSelection, DiffEditRange, HunkType};

mod entries;
mod selection;
mod topology;

fn external(file: DiffEditFileSelection) -> ExternalDiffSelection {
    let selected_exists = file.new_content.is_some();
    ExternalDiffSelection {
        file,
        selected_exists,
        selected_executable: None,
        whole_file_side: None,
    }
}

fn all_loaded_selections(left: &Path, right: &Path) -> Vec<ExternalDiffSelection> {
    load_external_diff(left, right, true)
        .expect("load external diff")
        .into_iter()
        .map(|file| ExternalDiffSelection {
            whole_file_side: (!file.supports_editing).then_some(ExternalDiffSide::New),
            file: DiffEditFileSelection {
                path: file.hunk.path,
                old_path: file.hunk.old_path,
                old_content: file.hunk.old.content,
                new_content: file.hunk.new.content,
                hunk_type: file.hunk.hunk_type,
                line_ranges: diff_edit_ranges(file.changed_lines),
            },
            selected_exists: file.new_exists,
            selected_executable: file.new_executable,
        })
        .collect()
}

fn discarded_loaded_selections(left: &Path, right: &Path) -> Vec<ExternalDiffSelection> {
    load_external_diff(left, right, true)
        .expect("load external diff")
        .into_iter()
        .map(|file| ExternalDiffSelection {
            whole_file_side: (!file.supports_editing).then_some(ExternalDiffSide::Old),
            file: DiffEditFileSelection {
                path: file.hunk.path,
                old_path: file.hunk.old_path,
                old_content: file.hunk.old.content,
                new_content: file.hunk.new.content,
                hunk_type: file.hunk.hunk_type,
                line_ranges: Vec::new(),
            },
            selected_exists: file.old_exists,
            selected_executable: file.old_executable,
        })
        .collect()
}
