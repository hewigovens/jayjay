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

pub(super) use registry::PathProjection;

pub(super) fn path_projection(path: &str, mode: DiffProjectionMode) -> PathProjection {
    registry::path_projection(path, mode)
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
