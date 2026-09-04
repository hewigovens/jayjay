use gpui::{
    AnyElement, App, ClickEvent, ClipboardItem, Context, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use jayjay_core::DiffProjection;

use super::super::DiffViewMode;
use crate::app::theme::Theme;
use crate::diff::projection;
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::{icon_button, text_tooltip, toggle_button};

pub(super) fn file_editor_button(t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    div()
        .id("edit-working-copy-file")
        .debug_selector(|| "edit-working-copy-file".to_owned())
        .flex()
        .flex_none()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .px(px(6.))
        .h(px(22.))
        .rounded_md()
        .text_size(px(11.))
        .text_color(rgb(t.fg_dim))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .tooltip(text_tooltip("Edit this working-copy file"))
        .on_click(cx.listener(|view, _, _, cx| {
            view.enter_selected_file_editor(cx);
        }))
        .child(icons::icon(glyph::PENCIL_CIRCLE, 12., t.fg_dim))
        .child("Edit")
        .into_any_element()
}

pub(super) fn view_mode_button(
    mode: DiffViewMode,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
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

pub(super) fn projection_button(
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

pub(super) fn svg_preview_button(
    active: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
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

pub(super) fn markdown_preview_button(
    active: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
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

pub(super) fn html_external_open_button(url: String, t: &Theme) -> AnyElement {
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
    .on_click(move |_, _, cx| crate::app::links::open_url(cx, &url))
    .into_any_element()
}

pub(super) fn exit_annotate_button(t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    div()
        .id(SharedString::from("exit-annotate"))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .px(px(8.))
        .py(px(3.))
        .rounded_md()
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

pub(super) fn path_copy_button(
    value: String,
    just_copied: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
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
        .rounded_md()
        .bg(rgb(bg))
        .cursor_pointer()
        .tooltip(text_tooltip(help))
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_click(on_click)
        .child(icons::icon(glyph_str, 14., fg))
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
