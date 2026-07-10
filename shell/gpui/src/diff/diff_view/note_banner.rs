//! Above-the-diff banner for Stale/Orphaned review notes; reuses `RepoViewModel::stale_or_orphaned_notes` rather than running its own reconcile.

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};
use jayjay_review::{NoteStatus, ReviewNoteStatus};

use crate::app::fonts;
use crate::app::theme::{Theme, with_alpha};
use crate::repo::window::RepoWindow;
use crate::ui::primitives::button;

pub(super) fn stale_notes_banner(
    notes: &[ReviewNoteStatus],
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Option<AnyElement> {
    if notes.is_empty() {
        return None;
    }
    let mut col = div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .px(px(18.))
        .py(px(8.))
        .bg(rgba(with_alpha(
            t.tag_conflict_fg,
            if t.is_dark { 0x16 } else { 0x0c },
        )))
        .debug_selector(|| "stale-notes-banner".to_owned());
    for note in notes {
        col = col.child(stale_note_row(note, t, cx));
    }
    Some(col.into_any_element())
}

fn stale_note_row(
    status: &ReviewNoteStatus,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let label = if status.status == NoteStatus::Orphaned {
        "Orphaned"
    } else {
        "Stale"
    };
    let note_id = status.note.id.clone();
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .text_size(px(11.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(t.tag_conflict_fg))
                .child(SharedString::from(label)),
        )
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(px(11.))
                .text_color(rgb(t.fg))
                .child(SharedString::from(format!(
                    "{}:{}",
                    status.note.path, status.note.line
                ))),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(status.note.body.clone())),
        )
        .child(
            button(
                SharedString::from(format!("stale-note-resolve-{note_id}")),
                "Resolve",
                t,
                false,
            )
            .on_click(cx.listener(move |view, _, _, cx| {
                view.resolve_review_note(note_id.clone(), cx);
            })),
        )
        .into_any_element()
}
