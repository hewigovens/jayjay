//! Notes render fully only in the unified view; this banner is the side-by-side view's bridge back to them when the file has active notes.

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};
use jayjay_review::{NoteStatus, ReviewNoteStatus};

use crate::app::theme::{Theme, ui_font_size, with_alpha};
use crate::repo::window::RepoWindow;

/// Wraps `body` with a notes banner; `notes` must already be scoped to this hunk (see `RepoWindow::notes_for_selected_hunk`).
pub(super) fn with_sbs_note_banner(
    body: AnyElement,
    notes: &[ReviewNoteStatus],
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let count = notes
        .iter()
        .filter(|n| n.status == NoteStatus::Current && !n.note.resolved)
        .count();
    if count == 0 {
        return body;
    }
    let label = if count == 1 {
        "\u{25cf} 1 review note on this file".to_owned()
    } else {
        format!("\u{25cf} {count} review notes on this file")
    };
    let banner = div()
        .debug_selector(|| "sbs-notes-banner".to_owned())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(10.))
        .py(px(6.))
        .bg(rgba(with_alpha(
            t.file_modified_color,
            if t.is_dark { 0x1c } else { 0x14 },
        )))
        .child(
            div()
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(label)),
        )
        .child(div().flex_1())
        .child(
            div()
                .id("sbs-notes-show-unified")
                .debug_selector(|| "sbs-notes-show-unified".to_owned())
                .text_size(ui_font_size(11.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(t.selected_accent))
                .cursor_pointer()
                .on_click(cx.listener(|view, _, _, cx| view.toggle_view_mode(cx)))
                .child("Show in Unified"),
        );
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .child(banner)
        .child(div().flex_1().min_h_0().child(body))
        .into_any_element()
}
