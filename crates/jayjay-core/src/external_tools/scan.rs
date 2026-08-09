use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Read as _;
use std::path::{Path, PathBuf};

use crate::file_display::{MAX_DIFF_BYTES, MAX_IMAGE_BYTES, is_image_path};
use crate::filesystem::{io_error, is_executable, safe_relative_path};
use crate::{CoreResult, DiffContent, DiffHunk, DiffPreview, HunkType};

use super::JJ_INSTRUCTIONS;
use super::content::external_content;

#[derive(Clone, Debug)]
enum Entry {
    File { path: PathBuf, executable: bool },
    Symlink(PathBuf),
    Other(PathBuf),
}

pub(super) struct ScannedExternalDiff {
    pub hunk: DiffHunk,
    pub old_exists: bool,
    pub new_exists: bool,
    pub old_is_regular_file: bool,
    pub new_is_regular_file: bool,
    pub old_is_text: bool,
    pub new_is_text: bool,
    pub old_executable: Option<bool>,
    pub new_executable: Option<bool>,
}

pub(super) fn scan_external_diff(
    left: &Path,
    right: &Path,
    exclude_instructions: bool,
) -> CoreResult<Vec<ScannedExternalDiff>> {
    let single_name = (!left.is_dir() && !right.is_dir()).then(|| {
        right
            .file_name()
            .or_else(|| left.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("file")
            .to_owned()
    });
    let left_entries = collect_entries(left, single_name.as_deref())?;
    let right_entries = collect_entries(right, single_name.as_deref())?;
    let paths: BTreeSet<&String> = left_entries.keys().chain(right_entries.keys()).collect();
    let mut hunks = Vec::new();

    for path in paths {
        if exclude_instructions && path == JJ_INSTRUCTIONS {
            continue;
        }
        let old_entry = left_entries.get(path);
        let new_entry = right_entries.get(path);
        if entries_equal(old_entry, new_entry)? {
            continue;
        }
        let hunk_type = match (old_entry, new_entry) {
            (None, Some(_)) => HunkType::Added,
            (Some(_), None) => HunkType::Removed,
            _ => HunkType::Modified,
        };
        let (old, old_is_text) = display_content(path, old_entry)?;
        let (new, new_is_text) = display_content(path, new_entry)?;
        let new_is_regular_file = new_entry.is_none_or(Entry::is_regular_file);
        hunks.push(ScannedExternalDiff {
            old_exists: old_entry.is_some(),
            new_exists: new_entry.is_some(),
            old_is_regular_file: old_entry.is_none_or(Entry::is_regular_file),
            new_is_regular_file,
            old_is_text,
            new_is_text,
            old_executable: old_entry.and_then(Entry::executable),
            new_executable: new_entry.and_then(Entry::executable),
            hunk: DiffHunk {
                path: path.clone(),
                old_path: None,
                old,
                new,
                hunk_type,
                supports_conflict_editor: false,
                supports_file_editor: new_is_regular_file && new_is_text,
                review_identity: String::new(),
                projection: None,
            },
        });
    }

    Ok(hunks)
}

fn collect_entries(root: &Path, single_name: Option<&str>) -> CoreResult<BTreeMap<String, Entry>> {
    let metadata = fs::symlink_metadata(root).map_err(|error| io_error("read", root, error))?;
    if !metadata.is_dir() {
        let name = single_name
            .map(str::to_owned)
            .or_else(|| root.file_name()?.to_str().map(str::to_owned))
            .unwrap_or_else(|| "file".to_owned());
        return Ok(BTreeMap::from([(name, entry_for(root, &metadata))]));
    }

    let mut entries = BTreeMap::new();
    collect_directory(root, root, &mut entries)?;
    Ok(entries)
}

fn collect_directory(
    root: &Path,
    directory: &Path,
    entries: &mut BTreeMap<String, Entry>,
) -> CoreResult<()> {
    let children =
        fs::read_dir(directory).map_err(|error| io_error("read directory", directory, error))?;
    for child in children {
        let child = child.map_err(|error| io_error("read directory entry", directory, error))?;
        let path = child.path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| io_error("read", &path, error))?;
        if metadata.is_dir() {
            collect_directory(root, &path, entries)?;
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| crate::CoreError::Internal {
                message: format!(
                    "make {} relative to {}: {error}",
                    path.display(),
                    root.display()
                ),
            })?;
        let relative = safe_relative_path(relative, "external diff")?;
        entries.insert(
            relative.to_string_lossy().replace('\\', "/"),
            entry_for(&path, &metadata),
        );
    }
    Ok(())
}

fn entry_for(path: &Path, metadata: &fs::Metadata) -> Entry {
    if metadata.file_type().is_symlink() {
        Entry::Symlink(path.to_owned())
    } else if metadata.is_file() {
        Entry::File {
            path: path.to_owned(),
            executable: is_executable(metadata),
        }
    } else {
        Entry::Other(path.to_owned())
    }
}

impl Entry {
    fn is_regular_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    fn executable(&self) -> Option<bool> {
        match self {
            Self::File { executable, .. } => Some(*executable),
            Self::Symlink(_) | Self::Other(_) => None,
        }
    }
}

fn entries_equal(left: Option<&Entry>, right: Option<&Entry>) -> CoreResult<bool> {
    match (left, right) {
        (None, None) => Ok(true),
        (
            Some(Entry::File {
                path: left,
                executable: left_executable,
            }),
            Some(Entry::File {
                path: right,
                executable: right_executable,
            }),
        ) => {
            if left_executable != right_executable {
                return Ok(false);
            }
            files_equal(left, right)
        }
        (Some(Entry::Symlink(left)), Some(Entry::Symlink(right))) => {
            let left =
                fs::read_link(left).map_err(|error| io_error("read symlink", left, error))?;
            let right =
                fs::read_link(right).map_err(|error| io_error("read symlink", right, error))?;
            Ok(left == right)
        }
        (Some(Entry::Other(left)), Some(Entry::Other(right))) => Ok(left == right),
        _ => Ok(false),
    }
}

fn files_equal(left: &Path, right: &Path) -> CoreResult<bool> {
    let left_metadata = fs::metadata(left).map_err(|error| io_error("read", left, error))?;
    let right_metadata = fs::metadata(right).map_err(|error| io_error("read", right, error))?;
    if left_metadata.len() != right_metadata.len() {
        return Ok(false);
    }
    let mut left_file = File::open(left).map_err(|error| io_error("open", left, error))?;
    let mut right_file = File::open(right).map_err(|error| io_error("open", right, error))?;
    let mut left_buffer = [0_u8; 64 * 1024];
    let mut right_buffer = [0_u8; 64 * 1024];
    loop {
        let left_count = left_file
            .read(&mut left_buffer)
            .map_err(|error| io_error("read", left, error))?;
        let right_count = right_file
            .read(&mut right_buffer)
            .map_err(|error| io_error("read", right, error))?;
        if left_count != right_count || left_buffer[..left_count] != right_buffer[..right_count] {
            return Ok(false);
        }
        if left_count == 0 {
            return Ok(true);
        }
    }
}

fn display_content(path: &str, entry: Option<&Entry>) -> CoreResult<(DiffContent, bool)> {
    let Some(entry) = entry else {
        return Ok((DiffContent::default(), true));
    };
    match entry {
        Entry::File { path: file, .. } if is_image_path(path) => {
            let size = fs::metadata(file)
                .map(|metadata| metadata.len())
                .unwrap_or(0);
            if size == 0 || size > MAX_IMAGE_BYTES as u64 {
                return Ok((
                    DiffContent::new(Some(format!("<image ({size} bytes)>")), None),
                    false,
                ));
            }
            Ok((
                DiffContent::new(
                    Some(format!("<image ({size} bytes)>")),
                    Some(DiffPreview::Image {
                        path: file.to_string_lossy().into_owned(),
                    }),
                ),
                false,
            ))
        }
        Entry::File { path: file, .. } => {
            let content = external_content(file, MAX_DIFF_BYTES)?;
            Ok((DiffContent::new(Some(content.text), None), content.is_text))
        }
        Entry::Symlink(file) => {
            let target =
                fs::read_link(file).map_err(|error| io_error("read symlink", file, error))?;
            Ok((
                DiffContent::new(
                    Some(format!("symlink -> {}", target.to_string_lossy())),
                    None,
                ),
                false,
            ))
        }
        Entry::Other(_) => Ok((
            DiffContent::new(Some("<unsupported file>".to_owned()), None),
            false,
        )),
    }
}

#[cfg(test)]
mod tests;
