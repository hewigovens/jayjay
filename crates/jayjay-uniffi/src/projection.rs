use jayjay_core::projection::{self, ProjectionView};
use jayjay_core::{DiffProjection, DiffProjectionMode};

#[uniffi::remote(Enum)]
pub enum ProjectionView {
    Source,
    Diff,
    Rendered,
}

#[uniffi::export]
fn projection_default_view(projection: DiffProjection) -> ProjectionView {
    ProjectionView::default_for(&projection)
}

#[uniffi::export]
fn projection_view_request_mode(view: ProjectionView) -> DiffProjectionMode {
    view.request_mode()
}

#[uniffi::export]
fn projection_default_mode(projection: Option<DiffProjection>) -> Option<DiffProjectionMode> {
    projection::default_mode(projection.as_ref())
}

#[uniffi::export]
fn projection_opens_automatically(projection: DiffProjection) -> bool {
    projection::opens_automatically(&projection)
}

#[uniffi::export]
fn projection_shows_source_toggle(projection: DiffProjection) -> bool {
    projection::shows_source_toggle(&projection)
}

#[uniffi::export]
fn projection_shows_banner(projection: DiffProjection, view: ProjectionView) -> bool {
    projection::shows_banner(&projection, view)
}

#[uniffi::export]
fn projection_title(projection: DiffProjection) -> String {
    projection::title(&projection)
}

#[uniffi::export]
fn projection_help(projection: Option<DiffProjection>) -> String {
    projection::help(projection.as_ref())
}

#[uniffi::export]
fn projection_active_help(projection: Option<DiffProjection>) -> String {
    projection::active_help(projection.as_ref())
}

#[uniffi::export]
fn projection_cache_identity(
    projection: Option<DiffProjection>,
    mode: Option<DiffProjectionMode>,
) -> String {
    projection::cache_identity(projection.as_ref(), mode)
}

#[uniffi::export]
fn svg_opens_preview(old_content: Option<String>, new_content: Option<String>) -> bool {
    projection::svg_opens_preview(
        old_content.as_deref().unwrap_or_default(),
        new_content.as_deref().unwrap_or_default(),
    )
}
