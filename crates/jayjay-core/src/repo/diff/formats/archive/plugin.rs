use crate::types::*;

use super::super::{
    FormatInput, ProjectionPair,
    types::{DiffFormatPlugin, has_extension, project_text_pair},
};
use super::{listing::ArchiveListing, zip::ZipReader};

pub(crate) struct ArchivePlugin;

impl DiffFormatPlugin for ArchivePlugin {
    fn id(&self) -> &'static str {
        "archive"
    }

    fn version(&self) -> u32 {
        1
    }

    fn label(&self) -> String {
        jayjay_i18n::diff::plugin_archive()
    }

    fn render_kind(&self) -> DiffRenderKind {
        DiffRenderKind::Text
    }

    fn matches_path(&self, path: &str) -> bool {
        has_extension(
            path,
            &[
                "zip", "jar", "war", "ear", "aar", "apk", "ipa", "whl", "nupkg", "vsix", "xpi",
                "epub",
            ],
        )
    }

    fn matches_input(&self, input: FormatInput<'_>) -> bool {
        self.matches_path(input.path)
            && [input.old, input.new]
                .into_iter()
                .flatten()
                .any(|bytes| ZipReader::new(bytes).is_archive())
    }

    fn content_gated(&self) -> bool {
        true
    }

    fn virtual_path(&self, path: &str) -> String {
        format!("{path}.txt")
    }

    fn project(&self, input: FormatInput<'_>) -> JayResult<ProjectionPair> {
        project_text_pair(
            input,
            self.projection(input.path, DiffProjectionMode::Processed),
            |bytes| Ok(ArchiveListing::new(ZipReader::new(bytes).entries()?).to_string()),
        )
    }
}
