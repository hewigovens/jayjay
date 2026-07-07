mod delimited;
mod ipynb;
mod markdown;
mod plist;
mod registry;
mod sarif;
mod types;

use crate::types::*;

#[derive(Clone, Copy)]
pub(super) struct FormatInput<'a> {
    pub(super) path: &'a str,
    pub(super) old: Option<&'a [u8]>,
    pub(super) new: Option<&'a [u8]>,
}

pub(super) struct ProjectionPair {
    pub(super) old_content: Option<String>,
    pub(super) new_content: Option<String>,
    pub(super) projection: DiffProjection,
}

pub(super) fn projection_for_path(path: &str, mode: DiffProjectionMode) -> Option<DiffProjection> {
    registry::projection_for_path(path, mode)
}

pub(super) fn projection_for_input(
    input: FormatInput<'_>,
    mode: DiffProjectionMode,
) -> Option<DiffProjection> {
    registry::projection_for_input(input, mode)
}

pub(super) fn project_pair(input: FormatInput<'_>) -> Option<CoreResult<ProjectionPair>> {
    registry::project_pair(input)
}
