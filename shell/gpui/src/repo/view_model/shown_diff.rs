use super::SvgPreviewContent;
use jayjay_core::DiffProjection;
use jayjay_core::diff::FileDiff;
use jayjay_markdown::MarkdownDocument;
use std::sync::Arc;

#[derive(Default)]
pub struct ShownDiff {
    pub diff: Option<Arc<FileDiff>>,
    pub projection: Option<DiffProjection>,
    pub svg_preview: Option<Arc<SvgPreviewContent>>,
    /// Post-change document only — the rich preview renders a single after view.
    pub markdown_preview: Option<Arc<MarkdownDocument>>,
    pub old_content: Option<Arc<str>>,
    pub new_content: Option<Arc<str>>,
    pub supports_file_editor: bool,
}
