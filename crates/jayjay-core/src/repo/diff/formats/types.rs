use crate::types::*;

use super::{FormatInput, ProjectionPair};

pub(super) trait DiffFormatPlugin: Sync {
    fn id(&self) -> &'static str;
    fn version(&self) -> u32;
    fn label(&self) -> &'static str;
    fn render_kind(&self) -> DiffRenderKind;
    fn matches_path(&self, path: &str) -> bool;
    fn matches_input(&self, input: FormatInput<'_>) -> bool {
        self.matches_path(input.path)
    }
    fn virtual_path(&self, path: &str) -> String;
    fn project(&self, input: FormatInput<'_>) -> CoreResult<ProjectionPair>;

    fn projection(&self, path: &str, mode: DiffProjectionMode) -> DiffProjection {
        DiffProjection {
            plugin_id: self.id().to_owned(),
            plugin_label: self.label().to_owned(),
            plugin_version: self.version(),
            mode,
            render_kind: self.render_kind(),
            virtual_path: match mode {
                DiffProjectionMode::Raw => path.to_owned(),
                DiffProjectionMode::Processed => self.virtual_path(path),
            },
            diagnostics: Vec::new(),
        }
    }
}

pub(super) fn has_extension(path: &str, extensions: &[&str]) -> bool {
    path.rsplit('.').next().is_some_and(|ext| {
        extensions
            .iter()
            .any(|candidate| ext.eq_ignore_ascii_case(candidate))
    })
}

pub(super) fn project_text_pair(
    input: FormatInput<'_>,
    projection: DiffProjection,
    mut project: impl FnMut(&[u8]) -> CoreResult<String>,
) -> CoreResult<ProjectionPair> {
    Ok(ProjectionPair {
        old_content: input.old.map(&mut project).transpose()?,
        new_content: input.new.map(project).transpose()?,
        projection,
    })
}

pub(super) fn projection_error(message: impl Into<String>) -> CoreError {
    CoreError::Internal {
        message: message.into(),
    }
}
