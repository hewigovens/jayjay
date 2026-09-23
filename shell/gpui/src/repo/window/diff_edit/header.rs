use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};

use super::rows::DiffEditCardFile;
use crate::app::fonts;
use crate::app::theme::{Theme, ui_font_size, with_alpha};
use crate::diff::{file_status, line_stats};
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::{CheckCircleState, check_circle};
use jayjay_core::diff_edit::DiffEditCheckbox;

pub(super) fn header_bg(t: &Theme) -> u32 {
    with_alpha(t.fg, if t.is_dark { 0x12 } else { 0x0a })
}

pub(super) fn header_row(
    view: &RepoWindow,
    card: &DiffEditCardFile,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let selected_count = view
        .diff_edit
        .session
        .file_counts(card.path.as_ref())
        .selected;
    let header_fill = if view.diff_edit_is_focused(card.path.as_ref()) {
        with_alpha(t.selected_bg, 0xff)
    } else {
        header_bg(t)
    };
    let mut row = div()
        .flex()
        .flex_1()
        .items_center()
        .gap(px(8.))
        .h(px(t.code_line_height()))
        .px(px(14.))
        .bg(rgba(header_fill));
    row = row.child(collapse_chevron(view, card, t, cx));
    if card.supported {
        let path = card.path.to_string();
        let state = match view.diff_edit_file_state(card.path.as_ref()) {
            DiffEditCheckbox::None => CheckCircleState::Off,
            DiffEditCheckbox::Some => CheckCircleState::Partial,
            DiffEditCheckbox::All => CheckCircleState::On,
        };
        row = row.child(
            check_circle(
                SharedString::from(format!("diff-edit-file-checkbox-{}", card.path)),
                state,
                t.selected_accent,
                t,
            )
            .on_click(cx.listener(move |view, _, _, cx| view.toggle_diff_edit_file(&path, cx))),
        );
    }
    let path_toggle = card.path.to_string();
    row = row
        .child(file_status::disc(
            card.hunk_type,
            file_status::color_for_hunk_type(card.hunk_type, t),
            12.,
        ))
        .child(
            div()
                .id(SharedString::from(format!(
                    "diff-edit-file-path-{}",
                    card.path
                )))
                .cursor_pointer()
                .on_click(cx.listener(move |view, _, _, cx| {
                    view.focus_and_toggle_diff_edit_collapse(&path_toggle, cx)
                }))
                .font_family(fonts::mono())
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_size(ui_font_size(12.))
                .child(card.path.to_string()),
        );
    if let Some(badge) = stats_badge(view, card, t) {
        row = row.child(badge);
    }
    if card.supported && selected_count > 0 && selected_count < card.changed_total {
        row = row.child(
            div()
                .px(px(6.))
                .rounded_full()
                .bg(rgba(with_alpha(t.selected_accent, 0x24)))
                .text_size(ui_font_size(10.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(format!("{selected_count} / {} lines", card.changed_total)),
        );
    }
    let row = row.child(div().flex_1()).child(
        div()
            .text_size(ui_font_size(11.))
            .text_color(rgb(t.fg_dim))
            .child(if card.supported {
                "Select files or lines to edit"
            } else {
                "Text edits not supported"
            }),
    );
    div()
        .flex()
        .w_full()
        .h(px(t.code_line_height()))
        .px(px(18.))
        .child(row)
        .into_any_element()
}

fn collapse_chevron(
    view: &RepoWindow,
    card: &DiffEditCardFile,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let glyph_str = if view.diff_edit_collapsed(card.path.as_ref()) {
        glyph::CARET_RIGHT
    } else {
        glyph::CARET_DOWN
    };
    let path = card.path.to_string();
    div()
        .id(SharedString::from(format!(
            "diff-edit-chevron-{}",
            card.path
        )))
        .cursor_pointer()
        .on_click(
            cx.listener(move |view, _, _, cx| view.focus_and_toggle_diff_edit_collapse(&path, cx)),
        )
        .child(icons::icon(glyph_str, 10., t.fg_faint))
        .into_any_element()
}

fn stats_badge(view: &RepoWindow, card: &DiffEditCardFile, t: &Theme) -> Option<AnyElement> {
    let stats = view.diff_edit.stats.as_ref()?.get(card.path.as_ref())?;
    line_stats(stats, 10., t.diff_gutter_added_fg, t.diff_gutter_removed_fg)
        .map(IntoElement::into_any_element)
}
