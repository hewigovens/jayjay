use jayjay_core::DiffHunk;
use jayjay_core::diff::FileDiff;

use crate::repo::window::PanelBoundsSlot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffViewMode {
    Unified,
    SideBySide,
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
    pub unified_bounds: PanelBoundsSlot,
    pub sbs_old_bounds: PanelBoundsSlot,
    pub sbs_new_bounds: PanelBoundsSlot,
}

/// Find-in-diff state.
pub struct FindState<'a> {
    pub query: Option<&'a str>,
    pub match_count: usize,
    pub match_current: usize,
    pub caret_visible: bool,
}
