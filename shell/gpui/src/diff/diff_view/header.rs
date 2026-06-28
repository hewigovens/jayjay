use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::DiffHunk;

use super::DiffViewMode;
use crate::app::fonts;
use crate::app::theme::{FONT_TAG, Theme};
use crate::diff::file_status;
use crate::diff::line::tag_for_hunk;
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::{capsule, icon_button, toggle_button};

pub(super) fn file_header(
    hunk: &DiffHunk,
    view_mode: DiffViewMode,
    is_annotating: bool,
    just_copied: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let (label, bg, fg) = tag_for_hunk(hunk, t);
    let path = SharedString::from(hunk.path.clone());

    let path_str = hunk.path.clone();
    let path_width = path_text_width(&path_str, px(13.), cx);
    let (icon_glyph, icon_color) = file_type_icon(hunk, t);
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
        .child(
            div()
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
                .child(path_copy_button(path_str, just_copied, t, cx)),
        );

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

    if is_annotating {
        row = row.child(exit_annotate_button(t, cx));
    }
    row.child(toggle_button(
        mode_glyph(view_mode),
        mode_tooltip(view_mode),
        "mode",
        view_mode == DiffViewMode::SideBySide,
        t,
        cx.listener(|view, _event: &ClickEvent, _window, cx| {
            view.toggle_view_mode(cx);
        }),
    ))
    .child(capsule(label, bg, fg, FONT_TAG))
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
