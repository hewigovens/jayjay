use gpui::{
    AnyElement, App, ClickEvent, Context, FontWeight, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use jayjay_core::{DiffHunk, DiffProjection};

use super::DiffViewMode;
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::diff::file_status;
use crate::diff::line::tag_for_hunk;
use crate::diff::projection;
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::{icon_button, text_tooltip, toggle_button};

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
    row.child(view_mode_button(state.view_mode, t, cx))
        .child(hunk_status_pill(label, bg, fg))
        .into_any_element()
}

fn view_mode_button(mode: DiffViewMode, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    toggle_button(
        mode_glyph(mode),
        mode_tooltip(mode),
        "mode",
        mode == DiffViewMode::SideBySide,
        t,
        cx.listener(|view, _event: &ClickEvent, _window, cx| {
            view.toggle_view_mode(cx);
        }),
    )
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

fn projection_button(
    projection: &DiffProjection,
    active: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    preview_button(
        "toggle-projection-preview",
        projection::icon(Some(projection)),
        projection::help(Some(projection)),
        active,
        t,
        cx.listener(|view, _event: &ClickEvent, _window, cx| {
            view.toggle_projection_rich_preview(cx);
        }),
    )
}

fn svg_preview_button(active: bool, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    preview_button(
        "toggle-svg-preview",
        glyph::EYE,
        "Show rendered SVG preview",
        active,
        t,
        cx.listener(|view, _event: &ClickEvent, _window, cx| {
            view.toggle_svg_rich_preview(cx);
        }),
    )
}

fn markdown_preview_button(active: bool, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    preview_button(
        "toggle-markdown-preview",
        glyph::EYE,
        "Show rendered Markdown",
        active,
        t,
        cx.listener(|view, _event: &ClickEvent, _window, cx| {
            view.toggle_markdown_rich_preview(cx);
        }),
    )
}

fn preview_button<F>(
    id: &'static str,
    glyph_str: &'static str,
    help: &'static str,
    active: bool,
    t: &Theme,
    on_click: F,
) -> AnyElement
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    let (bg, fg) = if active {
        (t.toggle_active_bg, t.toggle_active_fg)
    } else {
        (t.toggle_inactive_bg, t.toggle_inactive_fg)
    };
    div()
        .id(SharedString::from(id))
        .debug_selector(move || id.to_owned())
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .w(px(24.))
        .h(px(22.))
        .rounded_sm()
        .bg(rgb(bg))
        .cursor_pointer()
        .tooltip(text_tooltip(help))
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_click(on_click)
        .child(icons::icon(glyph_str, 14., fg))
        .into_any_element()
}

fn html_external_open_button(url: String, t: &Theme) -> AnyElement {
    icon_button(
        "open-html-external",
        glyph::EXTERNAL_LINK,
        12.,
        20.,
        20.,
        t.fg_dim,
        t,
    )
    .debug_selector(|| "open-html-external".to_owned())
    .tooltip(text_tooltip("Open working-copy HTML in default app"))
    .on_click(move |_, _, cx| cx.open_url(&url))
    .into_any_element()
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

fn exit_annotate_button(t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    div()
        .id(SharedString::from("exit-annotate"))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .px(px(8.))
        .py(px(3.))
        .rounded_sm()
        .bg(rgb(t.toggle_active_bg))
        .text_size(px(11.))
        .text_color(rgb(t.toggle_active_fg))
        .cursor_pointer()
        .on_click(cx.listener(|view, _event: &ClickEvent, _w, cx| {
            view.toggle_annotate(cx);
        }))
        .child(icons::icon(glyph::X, 11., t.toggle_active_fg))
        .child("Exit Annotate")
        .into_any_element()
}

fn mode_glyph(mode: DiffViewMode) -> &'static str {
    match mode {
        DiffViewMode::Unified => glyph::ROWS,
        DiffViewMode::SideBySide => glyph::COLUMNS,
    }
}

fn mode_tooltip(mode: DiffViewMode) -> &'static str {
    match mode {
        DiffViewMode::Unified => "Unified",
        DiffViewMode::SideBySide => "Side-by-side",
    }
}

fn path_copy_button(
    value: String,
    just_copied: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    use gpui::ClipboardItem;
    let (glyph_str, color) = if just_copied {
        (glyph::CHECK, t.success_fg)
    } else {
        (glyph::COPY, t.fg_dim)
    };
    icon_button("copy-path", glyph_str, 12., 20., 20., color, t)
        .debug_selector(|| "diff-copy-path".to_owned())
        .on_click(cx.listener(move |view, _, _, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
            view.mark_copied("path".into(), cx);
        }))
        .into_any_element()
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
