mod controls;
mod path;

use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, ParentElement, SharedString,
    Styled, div, px, rgb,
};
use jayjay_core::{DiffHunk, DiffProjection};

use self::controls::*;
use self::path::*;
use super::DiffViewMode;
use crate::app::theme::{Theme, ui_font_size};
use crate::diff::file_status;
use crate::diff::line::tag_for_hunk;
use crate::diff::projection;
use crate::repo::window::{FocusStop, RepoWindow};

const DIFF_HEADER_STATUS_FONT: f32 = 11.;
pub(crate) const DETAIL_INSET: f32 = 20.;

pub(super) struct ProjectionHeaderState<'a> {
    pub(super) projection: Option<&'a DiffProjection>,
    pub(super) active: bool,
}

pub(super) struct FileHeaderState<'a> {
    pub(super) hunk: &'a DiffHunk,
    pub(super) view_mode: DiffViewMode,
    pub(super) projection: ProjectionHeaderState<'a>,
    pub(super) active_markdown_preview: bool,
    pub(super) can_render_markdown_preview: bool,
    pub(super) active_svg_preview: bool,
    pub(super) can_render_svg_preview: bool,
    pub(super) is_annotating: bool,
    pub(super) just_copied: bool,
    pub(super) html_external_url: Option<&'a str>,
    pub(super) can_edit_file: bool,
    pub(super) can_edit_diff: bool,
    pub(super) detail_width: f32,
    pub(super) focused: Option<FocusStop>,
}

pub(super) fn file_header(
    state: FileHeaderState<'_>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let hunk = state.hunk;
    let (label, bg, fg) = tag_for_hunk(hunk, t);
    let old_path = hunk
        .old_path
        .as_ref()
        .filter(|old_path| *old_path != &hunk.path);
    let show_projection_button = state
        .projection
        .projection
        .is_some_and(|projection| !projection::opens_automatically(projection));
    let reserved = header_reserved_width(&state, show_projection_button, old_path.is_some());

    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .px(px(DETAIL_INSET))
        .h(px(crate::diff::file_column::file_row_height(t)))
        .debug_selector(|| "diff-header".to_owned())
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border));

    if state.can_edit_file {
        row = row.child(file_editor_button(t, cx));
    }

    let mut path_group = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .flex_1()
        .min_w_0()
        .child(file_path_label(
            &hunk.path,
            path_budget_chars(state.detail_width, reserved, t),
            t,
        ))
        .child(path_copy_button(
            hunk.path.clone(),
            state.just_copied,
            t,
            cx,
        ));
    if let Some(projection) = state.projection.projection
        && !projection::opens_automatically(projection)
    {
        path_group = path_group.child(projection_button(
            projection,
            state.projection.active,
            t,
            cx,
        ));
    }
    if let Some(url) = state.html_external_url {
        path_group = path_group.child(html_external_open_button(url.to_owned(), t));
    }
    if state.can_render_markdown_preview {
        path_group = path_group.child(markdown_preview_button(
            state.active_markdown_preview,
            t,
            cx,
        ));
    }
    if state.can_render_svg_preview {
        path_group = path_group.child(svg_preview_button(state.active_svg_preview, t, cx));
    }
    row = row.child(path_group);

    if let Some(old_path) = old_path {
        row = row.child(rename_origin_label(old_path, t, cx));
    }
    if state.is_annotating {
        row = row.child(exit_annotate_button(t, cx));
    }
    if state.can_edit_diff {
        row = row.child(edit_diff_button(
            state.focused == Some(FocusStop::EditDiff),
            t,
            cx,
        ));
    }
    row.child(view_mode_button(
        state.view_mode,
        state.focused == Some(FocusStop::DiffLayout),
        t,
        cx,
    ))
    .child(hunk_status_pill(label, bg, fg))
    .into_any_element()
}

fn hunk_status_pill(label: &'static str, bg: u32, fg: u32) -> impl IntoElement {
    div()
        .flex_none()
        .px(px(6.))
        .py(px(1.))
        .rounded_full()
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(ui_font_size(DIFF_HEADER_STATUS_FONT))
        .font_weight(FontWeight::SEMIBOLD)
        .child(SharedString::from(label))
}

pub(super) fn hunk_is_submodule(hunk: &DiffHunk) -> bool {
    file_status::is_submodule(hunk)
}

pub(super) fn hunk_is_git_lfs(hunk: &DiffHunk) -> bool {
    file_status::is_lfs(hunk)
}
