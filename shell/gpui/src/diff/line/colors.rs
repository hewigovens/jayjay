use jayjay_core::diff::{ConflictLineKind, DiffSpanStyle};

use crate::app::theme::Theme;

pub fn line_bg_color(style: DiffSpanStyle, conflict_kind: ConflictLineKind, theme: &Theme) -> u32 {
    match conflict_kind {
        ConflictLineKind::Start => theme.diff_conflict_header_bg,
        ConflictLineKind::End | ConflictLineKind::Section => theme.diff_conflict_section_bg,
        ConflictLineKind::Content => theme.diff_conflict_content_bg,
        ConflictLineKind::Added => theme.diff_added_bg,
        ConflictLineKind::Removed => theme.diff_removed_bg,
        ConflictLineKind::None => match style {
            DiffSpanStyle::Added => theme.diff_added_bg,
            DiffSpanStyle::Removed => theme.diff_removed_bg,
            DiffSpanStyle::Context | DiffSpanStyle::Unchanged => theme.diff_context_bg,
            DiffSpanStyle::Separator => theme.diff_separator_bg,
        },
    }
}

pub fn line_text_color(
    style: DiffSpanStyle,
    conflict_kind: ConflictLineKind,
    theme: &Theme,
) -> u32 {
    match conflict_kind {
        ConflictLineKind::Start => theme.diff_conflict_header_fg,
        ConflictLineKind::End | ConflictLineKind::Section => theme.diff_conflict_section_fg,
        ConflictLineKind::Added => theme.diff_text_added,
        ConflictLineKind::Removed => theme.diff_text_removed,
        _ => match style {
            DiffSpanStyle::Added => theme.diff_text_added,
            DiffSpanStyle::Removed => theme.diff_text_removed,
            _ => theme.diff_text_context,
        },
    }
}
