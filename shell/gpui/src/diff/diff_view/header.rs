mod controls;

use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, ParentElement, SharedString,
    Styled, div, px, rgb,
};
use jayjay_core::{DiffHunk, DiffProjection};

use self::controls::*;
use super::DiffViewMode;
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::diff::file_status;
use crate::diff::line::tag_for_hunk;
use crate::diff::projection;
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, glyph};

const DIFF_HEADER_STATUS_FONT: f32 = 11.;

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
}

pub(super) fn file_header(
    state: FileHeaderState<'_>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let hunk = state.hunk;
    let (label, bg, fg) = tag_for_hunk(hunk, t);
    let path = SharedString::from(hunk.path.clone());

    let path_str = hunk.path.clone();
    let path_width = path_text_width(&path_str, px(13.), cx);
    let (icon_glyph, icon_color) = file_type_icon(hunk, t);
    let mut path_group = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .flex_1()
        .min_w_0()
        .child(icons::icon(icon_glyph, 16., icon_color))
        .child(
            div()
                .debug_selector(|| "diff-file-path".to_owned())
                .w(path_width)
                .flex_shrink_1()
                .min_w_0()
                .truncate()
                .font_family(fonts::mono())
                .text_size(px(13.))
                .text_color(rgb(t.fg))
                .child(path),
        )
        .child(path_copy_button(path_str, state.just_copied, t, cx));
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
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .px(px(12.))
        .py(px(8.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(path_group);

    if let Some(old_path) = hunk.old_path.as_ref()
        && Some(old_path) != Some(&hunk.path)
    {
        let old_path_label = format!("{old_path} →");
        let old_path_width = path_text_width(&old_path_label, px(11.), cx);
        row = row.child(
            div()
                .debug_selector(|| "diff-file-old-path".to_owned())
                .w(old_path_width)
                .flex_shrink_1()
                .min_w_0()
                .truncate()
                .font_family(fonts::mono())
                .text_size(px(11.))
                .text_color(rgb(t.fg_faint))
                .child(SharedString::from(old_path_label)),
        );
    }

    if state.is_annotating {
        row = row.child(exit_annotate_button(t, cx));
    }
    if state.can_edit_file {
        row = row.child(file_editor_button(t, cx));
    }
    row.child(view_mode_button(state.view_mode, t, cx))
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
        .text_size(px(DIFF_HEADER_STATUS_FONT))
        .font_weight(FontWeight::SEMIBOLD)
        .child(SharedString::from(label))
}

fn path_text_width(
    path: &str,
    font_size: gpui::Pixels,
    cx: &mut Context<RepoWindow>,
) -> gpui::Pixels {
    let advance = fonts::mono_advance(cx, font_size);
    px((f32::from(advance) * path.chars().count() as f32).ceil() + 2.)
}

pub(super) fn hunk_is_submodule(hunk: &DiffHunk) -> bool {
    file_status::is_submodule(hunk)
}

pub(super) fn hunk_is_git_lfs(hunk: &DiffHunk) -> bool {
    file_status::is_lfs(hunk)
}

fn file_type_icon(hunk: &DiffHunk, t: &Theme) -> (&'static str, u32) {
    use jayjay_core::HunkType;
    let color = file_status::color(hunk, t);
    match hunk.hunk_type {
        HunkType::Added => (glyph::PLUS_CIRCLE, color),
        HunkType::Removed => (glyph::MINUS_CIRCLE, color),
        HunkType::Modified => (glyph::PENCIL_CIRCLE, color),
        HunkType::Renamed => (glyph::ARROW_CIRCLE_RIGHT, color),
    }
}
