use std::fs;
use std::path::Path;

use jj_lib::conflict_labels::ConflictLabels;
use jj_lib::conflicts::{
    ConflictMarkerStyle, ConflictMaterializeOptions, materialize_merge_result_to_bytes,
};
use jj_lib::files::FileMergeHunkLevel;
use jj_lib::merge::{Merge, SameChange};
use jj_lib::tree_merge::MergeOptions;

use crate::CoreResult;
use crate::MergeEditorHunk;
use crate::file_display::{MAX_DIFF_BYTES, bytes_to_display};
use crate::filesystem::io_error;

use super::content::{ExternalContent, external_content};

#[derive(Clone, Debug)]
pub struct ExternalMerge {
    pub left: String,
    pub base: String,
    pub right: String,
    pub result: String,
    pub is_text: bool,
    pub hunks: Vec<MergeEditorHunk>,
}

pub enum ExternalMergeResolution<'a> {
    Content(&'a str),
    Source(&'a Path),
}

pub fn load_external_merge(
    left_path: &Path,
    base_path: &Path,
    right_path: &Path,
    output_path: &Path,
    output_is_initialized: bool,
    marker_length: usize,
) -> CoreResult<ExternalMerge> {
    let left = external_content(left_path, MAX_DIFF_BYTES)?;
    let base = if base_path.as_os_str().is_empty() {
        ExternalContent {
            text: String::new(),
            is_text: true,
        }
    } else {
        external_content(base_path, MAX_DIFF_BYTES)?
    };
    let right = external_content(right_path, MAX_DIFF_BYTES)?;
    let output = external_content(output_path, MAX_DIFF_BYTES)?;
    let sides_text = left.is_text && base.is_text && right.is_text;
    let contents =
        Merge::from_removes_adds([base.text.clone()], [left.text.clone(), right.text.clone()]);
    let options = external_merge_options(marker_length);
    let (result, result_is_text) = if !output_is_initialized && output.text.is_empty() && sides_text
    {
        (
            bytes_to_display(&materialize_merge_result_to_bytes(
                &contents,
                &ConflictLabels::unlabeled(),
                &options,
            )),
            true,
        )
    } else {
        (output.text, output.is_text)
    };
    let is_text = sides_text && result_is_text;
    let hunks = if is_text {
        crate::merge_editor::merge_editor_hunks(&contents, &options, &result)
    } else {
        Vec::new()
    };
    Ok(ExternalMerge {
        left: left.text,
        base: base.text,
        right: right.text,
        result,
        is_text,
        hunks,
    })
}

pub fn conflict_marker_count(content: &str, marker_length: usize) -> usize {
    content
        .lines()
        .filter(|line| is_conflict_marker_line(line, b'<', marker_length))
        .count()
}

pub fn has_conflict_marker_remnants(content: &str, marker_length: usize) -> bool {
    let mut previous = None;
    content
        .lines()
        .filter_map(|line| conflict_marker_kind(line, marker_length))
        .any(|marker| {
            let follows_previous = previous.is_some_and(|previous| previous < marker);
            previous = (marker != ConflictMarkerKind::End).then_some(marker);
            follows_previous
        })
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ConflictMarkerKind {
    Start,
    Base,
    Separator,
    End,
}

fn conflict_marker_kind(line: &str, marker_length: usize) -> Option<ConflictMarkerKind> {
    [
        (b'<', ConflictMarkerKind::Start),
        (b'|', ConflictMarkerKind::Base),
        (b'=', ConflictMarkerKind::Separator),
        (b'>', ConflictMarkerKind::End),
    ]
    .into_iter()
    .find_map(|(marker, kind)| is_conflict_marker_line(line, marker, marker_length).then_some(kind))
}

fn is_conflict_marker_line(line: &str, marker: u8, marker_length: usize) -> bool {
    let marker_length = marker_length.max(1);
    let bytes = line.as_bytes();
    bytes.len() >= marker_length
        && bytes[..marker_length].iter().all(|byte| *byte == marker)
        && bytes.get(marker_length) != Some(&marker)
}

pub fn save_external_merge(
    output: &Path,
    resolution: ExternalMergeResolution<'_>,
) -> CoreResult<()> {
    match resolution {
        ExternalMergeResolution::Content(content) => {
            fs::write(output, content.as_bytes()).map_err(|error| io_error("write", output, error))
        }
        ExternalMergeResolution::Source(source) if source.as_os_str().is_empty() => {
            fs::write(output, []).map_err(|error| io_error("write", output, error))
        }
        ExternalMergeResolution::Source(source) => {
            let mut input = fs::File::open(source)
                .map_err(|error| io_error("open merge source", source, error))?;
            let mut result = fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(output)
                .map_err(|error| io_error("open merge result", output, error))?;
            std::io::copy(&mut input, &mut result)
                .map(|_| ())
                .map_err(|error| io_error("write merge result", output, error))
        }
    }
}

fn external_merge_options(marker_length: usize) -> ConflictMaterializeOptions {
    ConflictMaterializeOptions {
        marker_style: ConflictMarkerStyle::Git,
        marker_len: Some(marker_length.max(1)),
        merge: MergeOptions {
            hunk_level: FileMergeHunkLevel::Line,
            same_change: SameChange::Accept,
        },
    }
}

#[cfg(test)]
mod tests;
