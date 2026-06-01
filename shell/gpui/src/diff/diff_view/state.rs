use jayjay_core::DiffHunk;
use jayjay_core::diff::{ConflictLineKind, FileDiff};

use crate::repo::window::PanelBoundsSlot;
use crate::ui::input::LineInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffViewMode {
    Unified,
    SideBySide,
}

impl DiffViewMode {
    pub(crate) fn effective_for_diff(self, diff: Option<&FileDiff>) -> Self {
        if diff.is_some_and(|diff| {
            diff.lines
                .iter()
                .any(|line| line.conflict_kind != ConflictLineKind::None)
        }) {
            Self::Unified
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailMode {
    Diff,
    Annotate,
}

/// Pure-data inputs for the diff/annotate body.
pub struct DiffViewState<'a> {
    pub hunk: Option<&'a DiffHunk>,
    pub file_diff: Option<&'a FileDiff>,
    pub view_mode: DiffViewMode,
    pub detail_mode: DetailMode,
    pub annotate_lines: Option<std::sync::Arc<Vec<jayjay_core::AnnotationLine>>>,
    pub loading_annotate: bool,
    pub path_just_copied: bool,
    pub can_resolve_conflict: bool,
    pub unified_bounds: PanelBoundsSlot,
    pub sbs_old_bounds: PanelBoundsSlot,
    pub sbs_new_bounds: PanelBoundsSlot,
}

/// Find-in-diff state.
pub struct FindState<'a> {
    pub query: Option<&'a LineInput>,
    pub match_count: usize,
    pub match_current: usize,
}

#[cfg(test)]
mod tests {
    use jayjay_core::diff::{DiffSpanStyle, FileDiff};

    use super::*;

    #[test]
    fn effective_view_mode_uses_unified_for_conflicts() {
        let diff = file_diff(ConflictLineKind::Start);

        assert_eq!(
            DiffViewMode::SideBySide.effective_for_diff(Some(&diff)),
            DiffViewMode::Unified
        );
    }

    #[test]
    fn effective_view_mode_keeps_side_by_side_without_conflicts() {
        let diff = file_diff(ConflictLineKind::None);

        assert_eq!(
            DiffViewMode::SideBySide.effective_for_diff(Some(&diff)),
            DiffViewMode::SideBySide
        );
    }

    fn file_diff(conflict_kind: ConflictLineKind) -> FileDiff {
        FileDiff {
            path: "file.txt".to_owned(),
            language: "Text".to_owned(),
            lines: vec![jayjay_core::diff::DiffLine {
                old_line_no: Some(1),
                new_line_no: Some(1),
                style: DiffSpanStyle::Context,
                spans: Vec::new(),
                conflict_kind,
                no_eof_newline: false,
            }],
            whitespace_only_hidden: false,
        }
    }
}
