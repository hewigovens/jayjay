use crate::repo::window::HiddenGitFiles;
use std::sync::Arc;

use gpui::{FocusHandle, ScrollHandle};
use jayjay_core::diff::{ConflictLineKind, DiffSpanStyle, FileDiff};
use jayjay_core::{DiffHunk, DiffProjection, FileDiffStats};
use jayjay_markdown::MarkdownDocument;
use jayjay_review::{ReviewFileRollup, ReviewGroupState, ReviewNoteStatus};

use super::review_row_map::ReviewRowMap;
use crate::diff::Annotation;
use crate::diff::MarkdownImageCacheSlot;
use crate::diff::projection::ProjectionView;
use crate::repo::window::{ConflictResolution, DiffWrapCacheSlot, FocusStop, PanelBoundsSlot};
use crate::ui::input::LineInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffViewMode {
    Unified,
    SideBySide,
}

impl DiffViewMode {
    pub(crate) fn effective_for(self, shape: Option<DiffShape>) -> Self {
        if shape.is_some_and(|shape| !shape.side_by_side) {
            Self::Unified
        } else {
            self
        }
    }
}

/// Line-kind facts about a loaded diff, computed once so renders don't rescan every line.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DiffShape {
    side_by_side: bool,
    pub(crate) has_conflict_lines: bool,
}

impl DiffShape {
    pub(crate) fn of(diff: &FileDiff) -> Self {
        let (mut added, mut removed, mut conflict) = (false, false, false);
        for line in &diff.lines {
            added |= line.style == DiffSpanStyle::Added;
            removed |= line.style == DiffSpanStyle::Removed;
            conflict |= line.conflict_kind != ConflictLineKind::None;
        }
        Self {
            side_by_side: added && removed && !conflict,
            has_conflict_lines: conflict,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailMode {
    Diff,
    Annotate,
}

pub struct DiffViewState<'a> {
    pub(crate) hunk: Option<&'a DiffHunk>,
    pub(crate) line_stats: Option<&'a FileDiffStats>,
    pub(crate) no_changes: bool,
    pub(crate) hidden_only_files: Option<HiddenGitFiles>,
    pub(crate) file_diff: Option<&'a Arc<FileDiff>>,
    pub(crate) loaded_projection: Option<&'a DiffProjection>,
    pub(crate) projection_view: Option<ProjectionView>,
    pub(crate) active_markdown_preview: bool,
    pub(crate) active_svg_preview: bool,
    pub(crate) markdown_preview: Option<&'a Arc<MarkdownDocument>>,
    pub(crate) markdown_images: MarkdownImageCacheSlot,
    pub(crate) markdown_scroll: ScrollHandle,
    pub(crate) markdown_bounds: PanelBoundsSlot,
    pub(crate) repo_path: &'a str,
    pub(crate) svg_preview: Option<SvgPreviewContent<'a>>,
    pub(crate) html_external_url: Option<&'a str>,
    pub(crate) view_mode: DiffViewMode,
    pub(crate) detail_mode: DetailMode,
    pub(crate) annotation: Option<Result<Arc<Annotation>, gpui::SharedString>>,
    pub(crate) loading_annotate: bool,
    pub(crate) path_just_copied: bool,
    pub(crate) can_resolve_conflict: bool,
    pub(crate) can_edit_file: bool,
    pub(crate) can_edit_diff: bool,
    pub(crate) file_review: Option<ReviewFileRollup>,
    pub(crate) detail_width: f32,
    pub(crate) selected_file_has_conflict: bool,
    pub(crate) conflict_resolutions: Vec<ConflictResolution>,
    pub(crate) unified_bounds: PanelBoundsSlot,
    pub(crate) sbs_old_bounds: PanelBoundsSlot,
    pub(crate) sbs_new_bounds: PanelBoundsSlot,
    pub(crate) wrap_cache: DiffWrapCacheSlot,
    /// Reserves the stripe column whenever review chrome is on, so loading, projection, and binary files keep the same gutter width as reviewable text.
    pub(crate) shows_review: bool,
    pub(crate) review: Option<Arc<ReviewDisplayState>>,
    /// Already scoped to this hunk's path + identity and gated by the notes session; unified view only.
    pub(crate) notes: &'a [ReviewNoteStatus],
    /// Stale/Orphaned notes across the whole selected change, not just this hunk; reuses the already-loaded reconciliation report rather than re-running it.
    pub(crate) stale_or_orphaned_notes: &'a [ReviewNoteStatus],
    pub(crate) context_expansion_error: Option<gpui::SharedString>,
    pub(crate) focused: Option<FocusStop>,
}

pub(crate) struct ReviewDisplayState {
    pub(crate) path: String,
    pub(crate) identity: String,
    pub(crate) group_states: Vec<ReviewGroupState>,
    pub(crate) rows: Arc<ReviewRowMap>,
}

impl ReviewDisplayState {
    pub(crate) fn state_for_group(&self, group: u32) -> ReviewGroupState {
        self.group_states
            .get(group as usize)
            .copied()
            .unwrap_or(ReviewGroupState::Unreviewed)
    }

    pub(crate) fn unified_group(&self, line_ix: usize) -> Option<u32> {
        self.rows
            .unified_line_groups
            .get(line_ix)
            .copied()
            .flatten()
    }

    pub(crate) fn side_by_side_group(&self, row_ix: usize) -> Option<u32> {
        self.rows
            .side_by_side_row_groups
            .get(row_ix)
            .copied()
            .flatten()
    }
}

#[derive(Clone, Copy)]
pub struct SvgPreviewContent<'a> {
    pub(crate) old: Option<&'a str>,
    pub(crate) new: Option<&'a str>,
}

impl<'a> DiffViewState<'a> {
    pub(crate) fn effective_projection(&self) -> Option<&'a DiffProjection> {
        self.loaded_projection
            .or_else(|| self.hunk.and_then(|hunk| hunk.projection.as_ref()))
    }
}

pub struct FindState<'a> {
    pub(crate) query: Option<&'a LineInput>,
    pub(crate) ime_focus: Option<FocusHandle>,
    pub(crate) match_count: usize,
    pub(crate) match_current: usize,
}

#[cfg(test)]
mod tests {
    use jayjay_core::diff::{DiffSpanStyle, FileDiff};

    use super::*;

    #[test]
    fn effective_view_mode_uses_unified_for_conflicts() {
        let diff = file_diff(&[
            line(DiffSpanStyle::Removed, ConflictLineKind::Start),
            line(DiffSpanStyle::Added, ConflictLineKind::Added),
        ]);

        assert_eq!(
            DiffViewMode::SideBySide.effective_for(Some(DiffShape::of(&diff))),
            DiffViewMode::Unified
        );
    }

    #[test]
    fn effective_view_mode_uses_unified_for_added_only_diff() {
        let diff = file_diff(&[line(DiffSpanStyle::Added, ConflictLineKind::None)]);

        assert_eq!(
            DiffViewMode::SideBySide.effective_for(Some(DiffShape::of(&diff))),
            DiffViewMode::Unified
        );
    }

    #[test]
    fn effective_view_mode_uses_unified_for_removed_only_diff() {
        let diff = file_diff(&[line(DiffSpanStyle::Removed, ConflictLineKind::None)]);

        assert_eq!(
            DiffViewMode::SideBySide.effective_for(Some(DiffShape::of(&diff))),
            DiffViewMode::Unified
        );
    }

    #[test]
    fn effective_view_mode_keeps_side_by_side_for_two_column_diff_without_conflicts() {
        let diff = file_diff(&[
            line(DiffSpanStyle::Removed, ConflictLineKind::None),
            line(DiffSpanStyle::Added, ConflictLineKind::None),
        ]);

        assert_eq!(
            DiffViewMode::SideBySide.effective_for(Some(DiffShape::of(&diff))),
            DiffViewMode::SideBySide
        );
    }

    #[test]
    fn effective_view_mode_keeps_requested_mode_while_diff_loads() {
        assert_eq!(
            DiffViewMode::SideBySide.effective_for(None),
            DiffViewMode::SideBySide
        );
    }

    fn file_diff(lines: &[jayjay_core::diff::DiffLine]) -> FileDiff {
        FileDiff {
            path: "file.txt".to_owned(),
            language: "Text".to_owned(),
            lines: lines.to_vec(),
            whitespace_only_hidden: false,
        }
    }

    fn line(style: DiffSpanStyle, conflict_kind: ConflictLineKind) -> jayjay_core::diff::DiffLine {
        jayjay_core::diff::DiffLine {
            old_line_no: (style != DiffSpanStyle::Added).then_some(1),
            new_line_no: (style != DiffSpanStyle::Removed).then_some(1),
            style,
            spans: Vec::new(),
            conflict_kind,
            no_eof_newline: false,
            context_region: None,
            moved: None,
        }
    }
}
