use jayjay_core::{DiffHunk, HunkType};

use crate::app::theme::Theme;

pub fn tag_for_hunk(hunk: &DiffHunk, theme: &Theme) -> (&'static str, u32, u32) {
    let (bg, fg) = match hunk.hunk_type {
        HunkType::Added => (theme.tag_added_bg, theme.tag_added_fg),
        HunkType::Removed => (theme.tag_removed_bg, theme.tag_removed_fg),
        HunkType::Modified => (theme.tag_modified_bg, theme.tag_modified_fg),
        HunkType::Renamed => (theme.tag_renamed_bg, theme.tag_renamed_fg),
    };
    (crate::diff::file_status::label(hunk.hunk_type), bg, fg)
}
