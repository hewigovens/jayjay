use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use jayjay_core::DiffHunk;
use jayjay_core::diff::placeholders::{is_git_lfs, is_git_submodule};

use super::DiffViewMode;
use crate::app::fonts;
use crate::app::theme::{FONT_TAG, Theme};
use crate::diff::line::tag_for_hunk;
use crate::log::LogView;
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::capsule;

pub(super) fn file_header(
    hunk: &DiffHunk,
    view_mode: DiffViewMode,
    is_annotating: bool,
    just_copied: bool,
    t: &Theme,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let (label, bg, fg) = tag_for_hunk(hunk, t);
    let path = SharedString::from(hunk.path.clone());

    let path_str = hunk.path.clone();
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
        .child(icons::icon(icon_glyph, 16., icon_color))
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(px(13.))
                .text_color(rgb(t.fg))
                .child(path),
        )
        .child(path_copy_button(path_str, just_copied, t, cx));

    if let Some(old_path) = hunk.old_path.as_ref()
        && Some(old_path) != Some(&hunk.path)
    {
        row = row.child(
            div()
                .font_family(fonts::mono())
                .text_size(px(11.))
                .text_color(rgb(t.fg_faint))
                .child(SharedString::from(format!("← {old_path}"))),
        );
    }

    let mut row = row.child(div().flex_1());
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

pub(super) fn hunk_is_submodule(hunk: &DiffHunk) -> bool {
    is_git_submodule(hunk.old_content.as_deref()) || is_git_submodule(hunk.new_content.as_deref())
}

pub(super) fn hunk_is_git_lfs(hunk: &DiffHunk) -> bool {
    is_git_lfs(hunk.old_content.as_deref()) || is_git_lfs(hunk.new_content.as_deref())
}

fn exit_annotate_button(t: &Theme, cx: &mut Context<LogView>) -> AnyElement {
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

fn toggle_button<F>(
    glyph_str: &'static str,
    tooltip: &'static str,
    id: &'static str,
    active: bool,
    t: &Theme,
    on_click: F,
) -> AnyElement
where
    F: Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
{
    let (bg, fg) = if active {
        (t.toggle_active_bg, t.toggle_active_fg)
    } else {
        (t.toggle_inactive_bg, t.toggle_inactive_fg)
    };
    div()
        .id(SharedString::from(format!("toggle-{id}")))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .px(px(8.))
        .py(px(3.))
        .rounded_sm()
        .bg(rgb(bg))
        .text_size(px(11.))
        .text_color(rgb(fg))
        .cursor_pointer()
        .on_click(on_click)
        .child(icons::icon(glyph_str, 14., fg))
        .child(tooltip)
        .into_any_element()
}

fn path_copy_button(
    value: String,
    just_copied: bool,
    t: &Theme,
    cx: &mut Context<LogView>,
) -> AnyElement {
    use gpui::ClipboardItem;
    let (glyph_str, color) = if just_copied {
        (glyph::CHECK, t.success_fg)
    } else {
        (glyph::COPY, t.fg_faint)
    };
    div()
        .id(SharedString::from("copy-path"))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .w(px(20.))
        .h(px(20.))
        .rounded_sm()
        .cursor_pointer()
        .text_color(rgb(color))
        .on_click(cx.listener(move |view, _, _, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
            view.mark_copied("path".into(), cx);
        }))
        .child(icons::icon(glyph_str, 12., color))
        .into_any_element()
}

fn file_type_icon(hunk: &DiffHunk, t: &Theme) -> (&'static str, u32) {
    use jayjay_core::HunkType;
    match hunk.hunk_type {
        HunkType::Added => (glyph::PLUS_CIRCLE, t.tag_added_fg),
        HunkType::Removed => (glyph::MINUS_CIRCLE, t.tag_removed_fg),
        HunkType::Modified => (glyph::PENCIL_CIRCLE, t.tag_modified_fg),
        HunkType::Renamed => (glyph::ARROW_CIRCLE_RIGHT, t.tag_renamed_fg),
    }
}
