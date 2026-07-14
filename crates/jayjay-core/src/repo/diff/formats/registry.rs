use crate::types::*;

use super::{
    FormatInput, ProjectionPair, delimited::DelimitedPlugin, ipynb::IpynbPlugin,
    plist::PlistPlugin, sarif::SarifPlugin, types::DiffFormatPlugin,
};

static IPYNB: IpynbPlugin = IpynbPlugin;
static PLIST: PlistPlugin = PlistPlugin;
static DELIMITED: DelimitedPlugin = DelimitedPlugin;
static SARIF: SarifPlugin = SarifPlugin;

static PLUGINS: [&'static dyn DiffFormatPlugin; 4] = [&IPYNB, &PLIST, &DELIMITED, &SARIF];

fn plugin_for_path(path: &str) -> Option<&'static dyn DiffFormatPlugin> {
    PLUGINS
        .iter()
        .copied()
        .find(|plugin| plugin.matches_path(path))
}

fn plugin_for_input(input: FormatInput<'_>) -> Option<&'static dyn DiffFormatPlugin> {
    PLUGINS
        .iter()
        .copied()
        .find(|plugin| plugin.matches_input(input))
}

pub(in crate::repo::diff) enum PathProjection {
    None,
    Ready(DiffProjection),
    ContentGated,
}

pub(super) fn path_projection(path: &str, mode: DiffProjectionMode) -> PathProjection {
    match plugin_for_path(path) {
        None => PathProjection::None,
        Some(plugin) if plugin.content_gated() => PathProjection::ContentGated,
        Some(plugin) => PathProjection::Ready(plugin.projection(path, mode)),
    }
}

pub(super) fn projection_for_input(
    input: FormatInput<'_>,
    mode: DiffProjectionMode,
) -> Option<DiffProjection> {
    plugin_for_input(input).map(|plugin| plugin.projection(input.path, mode))
}

pub(super) fn project_pair(input: FormatInput<'_>) -> Option<CoreResult<ProjectionPair>> {
    plugin_for_input(input).map(|plugin| plugin.project(input))
}
