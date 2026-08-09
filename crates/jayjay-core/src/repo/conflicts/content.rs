use jj_lib::backend::{FileId, TreeValue};
use jj_lib::conflicts::{ConflictMarkerStyle, ConflictMaterializeOptions, MaterializedTreeValue};
use jj_lib::files::FileMergeHunkLevel;
use jj_lib::merge::MergedTreeValue;
use jj_lib::object_id::ObjectId as _;
use jj_lib::tree_merge::MergeOptions;

use crate::file_display::{MAX_DIFF_BYTES, optional_bytes_to_display, text_content};
use crate::repo::support::block_on_result;
use crate::{CoreResult, filesystem};

pub(super) fn conflict_supports_editor(
    store: &jj_lib::store::Store,
    path: &jj_lib::repo_path::RepoPath,
    value: &MergedTreeValue,
) -> CoreResult<bool> {
    if value.is_resolved() {
        return Ok(false);
    }
    let mut remaining = MAX_DIFF_BYTES;
    let mut expected_executable = None;
    let mut expected_copy_id = None;
    for term in value.iter().flatten() {
        let TreeValue::File {
            id,
            executable,
            copy_id,
        } = term
        else {
            return Ok(false);
        };
        if expected_executable.is_some_and(|expected| expected != *executable)
            || expected_copy_id.is_some_and(|expected| expected != copy_id)
        {
            return Ok(false);
        }
        expected_executable = Some(*executable);
        expected_copy_id = Some(copy_id);
        let mut reader = block_on_result("open conflicted file", store.read_file(path, id))?;
        let (bytes, truncated) = block_on_result(
            "read conflicted file",
            filesystem::read_to_limit(&mut reader, remaining),
        )?;
        if truncated || text_content(&bytes).is_none() {
            return Ok(false);
        }
        remaining -= bytes.len();
    }
    Ok(true)
}

pub(in crate::repo) fn materialized_conflict_supports_editor(
    value: &MaterializedTreeValue,
) -> bool {
    let MaterializedTreeValue::FileConflict(file) = value else {
        return false;
    };
    let total_bytes = conflict_total_bytes(&file.contents);
    file.executable.is_some()
        && file.copy_id.is_some()
        && total_bytes <= MAX_DIFF_BYTES
        && file
            .contents
            .iter()
            .all(|content| text_content(content.as_ref()).is_some())
}

pub(super) fn conflict_total_bytes<T: AsRef<[u8]>>(contents: &jj_lib::merge::Merge<T>) -> usize {
    contents.iter().map(|content| content.as_ref().len()).sum()
}

pub(super) fn conflict_materialize_options(marker_length: usize) -> ConflictMaterializeOptions {
    ConflictMaterializeOptions {
        marker_style: ConflictMarkerStyle::Diff,
        marker_len: Some(marker_length),
        merge: MergeOptions {
            hunk_level: FileMergeHunkLevel::Line,
            same_change: jj_lib::merge::SameChange::Accept,
        },
    }
}

pub(super) fn display_conflict_term<T: AsRef<[u8]>>(
    term: Option<&T>,
    total_bytes: usize,
) -> String {
    match term {
        Some(_) if total_bytes <= MAX_DIFF_BYTES => optional_bytes_to_display(term),
        Some(content) => format!(
            "<file too large to display ({} bytes)>",
            content.as_ref().len()
        ),
        None => String::new(),
    }
}

pub(super) fn conflict_fingerprint(ids: &jj_lib::merge::Merge<Option<FileId>>) -> String {
    ids.iter()
        .map(|id| id.as_ref().map_or_else(|| "-".to_owned(), |id| id.hex()))
        .collect::<Vec<_>>()
        .join("|")
}
