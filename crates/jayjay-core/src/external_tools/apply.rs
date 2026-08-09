use std::path::{Path, PathBuf};

use crate::filesystem::safe_relative_path;
use crate::repo::partition_validated_text_selection;
use crate::{CoreError, CoreResult};

use super::output::{copy_entry, output_matches_selection, remove_output_path, write_text};
use super::{ExternalDiffSelection, ExternalDiffSide};

pub fn apply_external_diff_selections(
    left_root: &Path,
    right_root: &Path,
    selections: &[ExternalDiffSelection],
    ignore_whitespace: bool,
) -> CoreResult<()> {
    let directory_output = right_root.is_dir();
    if !directory_output && selections.len() > 1 {
        return Err(CoreError::Internal {
            message: format!(
                "cannot apply multiple external diff files to {}",
                right_root.display()
            ),
        });
    }
    let mut prepared = Vec::with_capacity(selections.len());
    for selection in selections {
        let file = &selection.file;
        let relative = safe_relative_path(Path::new(&file.path), "external diff")?;
        let right_path = if directory_output {
            right_root.join(&relative)
        } else {
            right_root.to_owned()
        };
        let content = match selection.whole_file_side {
            Some(ExternalDiffSide::Old) => PreparedContent::OldEntry(if left_root.is_dir() {
                left_root.join(&relative)
            } else {
                left_root.to_owned()
            }),
            Some(ExternalDiffSide::New) => PreparedContent::KeepNew,
            None => PreparedContent::Lines(
                partition_validated_text_selection(file, ignore_whitespace)?.selected_text,
            ),
        };
        prepared.push(PreparedSelection {
            relative,
            right_path,
            content,
            selected_exists: selection.selected_exists,
            selected_executable: selection.selected_executable,
        });
    }
    validate_selected_topology(&prepared)?;
    if directory_output {
        prepared.sort_by(|left, right| {
            left.selected_exists
                .cmp(&right.selected_exists)
                .then_with(|| {
                    if left.selected_exists {
                        left.depth().cmp(&right.depth())
                    } else {
                        right.depth().cmp(&left.depth())
                    }
                })
                .then_with(|| left.relative.cmp(&right.relative))
        });
    }
    for operation in prepared {
        match &operation.content {
            PreparedContent::KeepNew => continue,
            PreparedContent::OldEntry(source) => {
                if operation.selected_exists {
                    copy_entry(
                        source,
                        right_root,
                        &operation.relative,
                        &operation.right_path,
                        directory_output,
                    )?;
                } else {
                    remove_output_path(&operation.right_path)?;
                }
                continue;
            }
            PreparedContent::Lines(_) => {}
        }
        let PreparedContent::Lines(selected_text) = &operation.content else {
            unreachable!();
        };
        if output_matches_selection(
            &operation.right_path,
            selected_text,
            operation.selected_exists,
            operation.selected_executable,
        )? {
            continue;
        }
        if operation.selected_exists {
            write_text(
                right_root,
                &operation.relative,
                &operation.right_path,
                directory_output,
                selected_text,
                operation.selected_executable,
            )?;
        } else {
            remove_output_path(&operation.right_path)?;
        }
    }
    Ok(())
}

fn validate_selected_topology(selections: &[PreparedSelection]) -> CoreResult<()> {
    for (index, left) in selections.iter().enumerate() {
        if !left.selected_exists {
            continue;
        }
        for right in selections.iter().skip(index + 1) {
            if right.selected_exists
                && (left.relative.starts_with(&right.relative)
                    || right.relative.starts_with(&left.relative))
            {
                return Err(CoreError::internal(format!(
                    "external diff selection cannot keep both {} and {}",
                    left.relative.display(),
                    right.relative.display()
                )));
            }
        }
    }
    Ok(())
}

struct PreparedSelection {
    relative: PathBuf,
    right_path: PathBuf,
    content: PreparedContent,
    selected_exists: bool,
    selected_executable: Option<bool>,
}

enum PreparedContent {
    Lines(String),
    OldEntry(PathBuf),
    KeepNew,
}

impl PreparedSelection {
    fn depth(&self) -> usize {
        self.relative.components().count()
    }
}

#[cfg(test)]
mod tests;
