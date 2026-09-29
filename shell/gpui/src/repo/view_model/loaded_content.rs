use jayjay_core::DiffProjection;
use jayjay_core::diff::FileDiff;
use jayjay_markdown::MarkdownDocument;
use jayjay_review::ReviewFileSnapshot;
use std::sync::Arc;

#[derive(Clone)]
pub struct LoadedDiff {
    pub diff: Arc<FileDiff>,
    pub projection: Option<DiffProjection>,
    pub svg_preview: Option<Arc<SvgPreviewContent>>,
    pub markdown_preview: Option<Arc<MarkdownDocument>>,
    /// The exact (old, new) strings `diff` was computed from; must be retained rather than re-read, since file content may have changed by the time an abandon-selected-lines action runs.
    pub old_content: Option<Arc<str>>,
    pub new_content: Option<Arc<str>>,
    pub supports_file_editor: bool,
    pub review: Option<Arc<LoadedReviewSnapshot>>,
}

pub struct LoadedReviewSnapshot {
    pub snapshot: ReviewFileSnapshot,
    pub display_groups: Vec<Vec<u32>>,
}

pub(in crate::repo) enum DiffLoadState {
    Missing,
    Failed,
    Loaded(LoadedDiff),
}

#[derive(Clone)]
pub struct SvgPreviewContent {
    pub(crate) old: Option<String>,
    pub new: Option<String>,
}
