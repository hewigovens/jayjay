use gpui::{
    AnyElement, App, Context, InteractiveElement, IntoElement, ParentElement, SharedString, Styled,
    div, px, rgb,
};
use jayjay_core::{BookmarkInfo, ChangeInfo, DiffStats};

use super::cells::{author, bookmark_chip, diff_stats, separator, tag_chip};
use crate::app::fonts;
use crate::app::theme::{FONT_ID, Theme, ui_font_size};
use crate::diff::DETAIL_INSET;
use crate::diff::file_row_height;
use crate::repo::RepoWindow;
use crate::repo::window::dag_row::{compact_id, compact_id_len, format_when, id_cell};
use crate::repo::window::ref_chips::copy_feedback_button;

pub(super) fn byline(
    change: &ChangeInfo,
    stats: Option<&DiffStats>,
    recently_copied: Option<&SharedString>,
    bookmarks: &[BookmarkInfo],
    budget: f32,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let shows_date = byline_width(change, stats, bookmarks, t, cx) <= budget;
    let compact = compact_id(&change.change_id);
    let change_copy_id: SharedString = "Change".into();

    let mut row = div()
        .debug_selector(|| "detail-metadata".to_owned())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .h(px(file_row_height(t)))
        .px(px(DETAIL_INSET))
        .border_b_1()
        .border_color(rgb(t.border))
        .min_w_0()
        .overflow_hidden()
        .child(author(change, false, t))
        .child(separator(t))
        .child(id_cell(
            &compact,
            change.change_id.short_len,
            t.change_id_prefix,
            FONT_ID,
            t,
        ))
        .child(copy_feedback_button(
            change_copy_id.clone(),
            change.change_id.id.clone(),
            recently_copied == Some(&change_copy_id),
            t.fg_faint,
            t,
            cx,
        ));
    if shows_date {
        row = row.child(separator(t)).child(
            div()
                .flex_none()
                .text_size(ui_font_size(FONT_ID))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(format_when(
                    change.author.timestamp_millis,
                ))),
        );
    }
    row = row.child(div().flex_1().min_w(px(8.)));
    if let Some(stats) = stats.and_then(|stats| diff_stats(stats, false, t)) {
        row = row.child(stats);
    }
    for name in &change.bookmarks {
        row = row.child(bookmark_chip(
            name,
            BookmarkInfo::is_conflicted_name(bookmarks, name),
            false,
            recently_copied,
            t,
            cx,
        ));
    }
    for name in &change.tags {
        row = row.child(tag_chip(name, t));
    }
    row.into_any_element()
}

fn byline_width(
    change: &ChangeInfo,
    stats: Option<&DiffStats>,
    bookmarks: &[BookmarkInfo],
    t: &Theme,
    cx: &App,
) -> f32 {
    let meta = t.scaled_font_size(FONT_ID);
    let mono_char = f32::from(fonts::mono_advance(cx, px(meta)));
    let ui_text = |s: &str| f32::from(fonts::ui_text_width(cx, s, px(meta)));

    let mut children = 7usize;
    let mut width = 14.
        + 5.
        + ui_text(&change.author.name)
        + 2. * ui_text("·")
        + compact_id_len(change.change_id.short_len) as f32 * mono_char
        + 20.
        + ui_text(&format_when(change.author.timestamp_millis))
        + 8.;
    if let Some(stats) = stats
        && (stats.insertions > 0 || stats.deletions > 0)
    {
        children += 1;
        let number = |n: u32| (1. + n.ilog10() as f32) * mono_char;
        let mut stats_width = 0.;
        if stats.insertions > 0 {
            stats_width += number(stats.insertions);
        }
        if stats.deletions > 0 {
            stats_width += number(stats.deletions);
        }
        if stats.insertions > 0 && stats.deletions > 0 {
            stats_width += 4.;
        }
        width += stats_width;
    }
    for name in &change.bookmarks {
        children += 1;
        width += ui_text(name) + 12.;
        if BookmarkInfo::is_conflicted_name(bookmarks, name) {
            width += meta + 3.;
        }
    }
    for name in &change.tags {
        children += 1;
        width += ui_text(name) + 12. + meta + 3.;
    }
    width + 6. * (children - 1) as f32
}
