use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, ClickEvent, Context, Div, InteractiveElement, IntoElement, ParentElement,
    SharedString, Stateful, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use jayjay_core::DiffProjection;

use super::super::DiffViewMode;
use crate::app::theme::{Theme, ui_font_size};
use crate::diff::projection;
use crate::repo::window::ref_chips::copy_feedback_button;
use crate::repo::window::{RepoWindow, focus_ring};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::text_tooltip;

pub(super) fn file_editor_button(
    compact: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    header_action(
        "edit-working-copy-file",
        glyph::PENCIL,
        (!compact).then_some("Edit File"),
        false,
        t,
    )
    .tooltip(text_tooltip("Edit this working-copy file"))
    .on_click(cx.listener(|view, _, _, cx| {
        view.enter_selected_file_editor(cx);
    }))
    .into_any_element()
}

pub(super) fn edit_diff_button(
    compact: bool,
    focused: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    focus_ring(
        header_action(
            "edit-diff",
            glyph::SQUARE_PENCIL,
            (!compact).then_some("Edit Diff"),
            false,
            t,
        )
        .tooltip(text_tooltip("Open dedicated diff edit mode"))
        .on_click(cx.listener(|view, _, _, cx| view.enter_diff_edit(cx))),
        focused,
        t,
    )
    .into_any_element()
}

pub(super) fn view_mode_button(
    mode: DiffViewMode,
    compact: bool,
    focused: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    focus_ring(
        header_action(
            "toggle-mode",
            mode_glyph(mode),
            (!compact).then_some(mode_label(mode)),
            false,
            t,
        )
        .tooltip(text_tooltip(match mode {
            DiffViewMode::Unified => "Switch to side-by-side",
            DiffViewMode::SideBySide => "Switch to unified",
        }))
        .on_click(cx.listener(|view, _event: &ClickEvent, _window, cx| {
            view.toggle_view_mode(cx);
        })),
        focused,
        t,
    )
    .into_any_element()
}

pub(super) fn projection_button(
    projection: &DiffProjection,
    active: bool,
    compact: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    preview_button(
        "toggle-projection-preview",
        projection::icon(Some(projection)),
        projection::help(Some(projection)),
        active,
        compact,
        t,
        cx.listener(|view, _event: &ClickEvent, _window, cx| {
            view.toggle_projection_rich_preview(cx);
        }),
    )
}

pub(super) fn svg_preview_button(
    active: bool,
    compact: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    preview_button(
        "toggle-svg-preview",
        glyph::EYE,
        "Show rendered SVG preview",
        active,
        compact,
        t,
        cx.listener(|view, _event: &ClickEvent, _window, cx| {
            view.toggle_svg_rich_preview(cx);
        }),
    )
}

pub(super) fn markdown_preview_button(
    active: bool,
    compact: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    preview_button(
        "toggle-markdown-preview",
        glyph::EYE,
        "Show rendered Markdown",
        active,
        compact,
        t,
        cx.listener(|view, _event: &ClickEvent, _window, cx| {
            view.toggle_markdown_rich_preview(cx);
        }),
    )
}

pub(super) fn html_external_open_button(url: String, t: &Theme) -> AnyElement {
    header_action("open-html-external", glyph::EXTERNAL_LINK, None, false, t)
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
        .text_size(ui_font_size(11.))
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
    copy_feedback_button("path".into(), value, just_copied, t.fg_dim, t, cx)
        .debug_selector(|| "diff-copy-path".to_owned())
        .relative()
        .top(px(1.))
        .into_any_element()
}

fn preview_button<F>(
    id: &'static str,
    glyph_str: &'static str,
    help: &'static str,
    active: bool,
    compact: bool,
    t: &Theme,
    on_click: F,
) -> AnyElement
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    header_action(id, glyph_str, (!compact).then_some("Preview"), active, t)
        .tooltip(text_tooltip(help))
        .on_click(on_click)
        .into_any_element()
}

fn header_action(
    id: &'static str,
    glyph_str: &'static str,
    label: Option<&'static str>,
    active: bool,
    t: &Theme,
) -> Stateful<Div> {
    let fg = if active { t.toggle_active_fg } else { t.fg_dim };
    div()
        .id(SharedString::from(id))
        .debug_selector(move || id.to_owned())
        .flex()
        .flex_none()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .px(px(6.))
        .h(px(t.scaled_control_height(22., 11.)))
        .rounded_md()
        .text_size(ui_font_size(11.))
        .text_color(rgb(fg))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)).text_color(rgb(t.fg)))
        .child(icons::icon(glyph_str, 12., fg))
        .when_some(label, |el, label| el.child(label))
}

fn mode_glyph(mode: DiffViewMode) -> &'static str {
    match mode {
        DiffViewMode::Unified => glyph::ROWS,
        DiffViewMode::SideBySide => glyph::COLUMNS,
    }
}

pub(super) fn mode_label(mode: DiffViewMode) -> &'static str {
    match mode {
        DiffViewMode::Unified => "Unified",
        DiffViewMode::SideBySide => "Side-by-side",
    }
}
