use jayjay_core::HunkType;

use crate::app::theme::Theme;

pub fn tag_for_hunk_type(hunk_type: HunkType, theme: &Theme) -> (String, u32, u32) {
    let (bg, fg) = match hunk_type {
        HunkType::Added | HunkType::Copied => (theme.tag_added_bg, theme.tag_added_fg),
        HunkType::Removed => (theme.tag_removed_bg, theme.tag_removed_fg),
        HunkType::Modified => (theme.tag_modified_bg, theme.tag_modified_fg),
        HunkType::Renamed => (theme.tag_renamed_bg, theme.tag_renamed_fg),
    };
    (crate::diff::file_status::label(hunk_type), bg, fg)
}
