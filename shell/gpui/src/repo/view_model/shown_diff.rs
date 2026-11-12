use super::SvgPreviewContent;
use crate::diff::DiffShape;
use jayjay_core::DiffProjection;
use jayjay_core::diff::FileDiff;
use jayjay_markdown::MarkdownDocument;
use std::sync::Arc;

#[derive(Default)]
pub struct ShownDiff {
    diff: Option<(Arc<FileDiff>, DiffShape)>,
    /// The hunk's own path; a projected diff's `path` is the rendered one.
    pub(super) source_path: Option<String>,
    pub projection: Option<DiffProjection>,
    pub svg_preview: Option<Arc<SvgPreviewContent>>,
    /// Post-change document only — the rich preview renders a single after view.
    pub markdown_preview: Option<Arc<MarkdownDocument>>,
    pub old_content: Option<Arc<str>>,
    pub new_content: Option<Arc<str>>,
    pub supports_file_editor: bool,
}

impl ShownDiff {
    pub fn diff(&self) -> Option<&Arc<FileDiff>> {
        self.diff.as_ref().map(|(diff, _)| diff)
    }

    pub(crate) fn shape(&self) -> Option<DiffShape> {
        self.diff.as_ref().map(|(_, shape)| *shape)
    }

    pub fn set_diff(&mut self, diff: Option<Arc<FileDiff>>) {
        self.diff = diff.map(|diff| {
            let shape = DiffShape::of(&diff);
            (diff, shape)
        });
    }

    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }
}
