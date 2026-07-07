use jayjay_core::diff::placeholders::{is_git_lfs, is_git_submodule};
use jayjay_core::{DiffHunk, HunkType};

use crate::app::theme::Theme;

pub(crate) fn color(hunk: &DiffHunk, theme: &Theme) -> u32 {
    if is_submodule(hunk) {
        return theme.file_renamed_color;
    }
    if is_lfs(hunk) {
        return theme.file_lfs_color;
    }
    color_for_hunk_type(hunk.hunk_type, theme)
}

pub(crate) fn color_for_hunk_type(hunk_type: HunkType, theme: &Theme) -> u32 {
    match hunk_type {
        HunkType::Added => theme.file_added_color,
        HunkType::Removed => theme.file_removed_color,
        HunkType::Modified => theme.file_modified_color,
        HunkType::Renamed => theme.file_renamed_color,
    }
}

pub(crate) fn is_submodule(hunk: &DiffHunk) -> bool {
    is_git_submodule(hunk.old.content.as_deref()) || is_git_submodule(hunk.new.content.as_deref())
}

pub(crate) fn is_lfs(hunk: &DiffHunk) -> bool {
    is_git_lfs(hunk.old.content.as_deref()) || is_git_lfs(hunk.new.content.as_deref())
}
