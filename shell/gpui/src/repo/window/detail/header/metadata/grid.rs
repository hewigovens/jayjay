use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div,
    px, rgb,
};
use jayjay_core::{BookmarkInfo, ChangeInfo, DiffStats};

use super::cells::{author, bookmark_chip, diff_stats, tag_chip};
use crate::app::fonts;
use crate::app::theme::{FONT_ID, Theme, ui_font_size};
use crate::diff::DETAIL_INSET;
use crate::repo::RepoWindow;
use crate::repo::window::dag_row::{format_when, id_cell};
use crate::repo::window::ref_chips::copy_feedback_button;

pub(super) fn grid(
    change: &ChangeInfo,
    stats: Option<&DiffStats>,
    recently_copied: Option<&SharedString>,
    bookmarks: &[BookmarkInfo],
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let mut col = div()
        .debug_selector(|| "detail-metadata-grid".to_owned())
        .px(px(DETAIL_INSET))
        .flex()
        .flex_col()
        .gap(px(5.))
        .child(grid_row(
            "Author:",
            author(change, true, t),
            GridAlign::Center,
            t,
        ))
        .child(grid_row(
            "Date:",
            div()
                .text_size(ui_font_size(FONT_ID))
                .text_color(rgb(t.fg))
                .child(SharedString::from(format_when(
                    change.author.timestamp_millis,
                )))
                .into_any_element(),
            GridAlign::Baseline,
            t,
        ))
        .child(id_row(
            "Change:",
            &change.change_id,
            t.change_id_prefix,
            recently_copied,
            t,
            cx,
        ))
        .child(id_row(
            "Commit:",
            &change.commit_id,
            t.commit_id_prefix,
            recently_copied,
            t,
            cx,
        ));

    if !change.parents.is_empty() {
        let parents = change
            .parents
            .iter()
            .map(|p| p.chars().take(12).collect::<String>())
            .collect::<Vec<_>>()
            .join(", ");
        col = col.child(grid_row(
            "Parents:",
            div()
                .min_w_0()
                .truncate()
                .font_family(fonts::mono())
                .text_size(ui_font_size(FONT_ID))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(parents))
                .into_any_element(),
            GridAlign::Baseline,
            t,
        ));
    }

    if !change.bookmarks.is_empty() {
        let mut value = div().flex().flex_row().items_center().gap(px(6.));
        for name in &change.bookmarks {
            value = value.child(bookmark_chip(
                name,
                BookmarkInfo::is_conflicted_name(bookmarks, name),
                true,
                recently_copied,
                t,
                cx,
            ));
        }
        col = col.child(grid_row(
            "Bookmarks:",
            value.into_any_element(),
            GridAlign::Center,
            t,
        ));
    }

    if !change.tags.is_empty() {
        let mut value = div().flex().flex_row().items_center().gap(px(6.));
        for name in &change.tags {
            value = value.child(tag_chip(name, t));
        }
        col = col.child(grid_row(
            "Tags:",
            value.into_any_element(),
            GridAlign::Center,
            t,
        ));
    }

    if let Some(stats) = stats.and_then(|stats| diff_stats(stats, true, t)) {
        col = col.child(grid_row("Changes:", stats, GridAlign::Baseline, t));
    }

    col.into_any_element()
}

#[derive(Clone, Copy)]
enum GridAlign {
    Baseline,
    Center,
}

fn grid_row(label: &str, value: AnyElement, align: GridAlign, t: &Theme) -> AnyElement {
    let row = div()
        .flex()
        .flex_row()
        .gap(px(8.))
        .child(label_cell(label, t))
        .child(value);
    match align {
        GridAlign::Baseline => row.items_baseline(),
        GridAlign::Center => row.items_center(),
    }
    .into_any_element()
}

fn label_cell(label: &str, t: &Theme) -> AnyElement {
    div()
        .flex_none()
        .flex()
        .justify_end()
        .w(px(t.scaled_font_size(FONT_ID) * 6.5))
        .child(
            div()
                .text_size(ui_font_size(FONT_ID))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(label.to_owned())),
        )
        .into_any_element()
}

fn id_row(
    label: &'static str,
    id: &jayjay_core::ShortId,
    prefix_color: u32,
    recently_copied: Option<&SharedString>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let copy_id: SharedString = label.trim_end_matches(':').into();
    let value = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .child(id_cell(&id.id, id.short_len, prefix_color, FONT_ID, t))
        .child(copy_feedback_button(
            copy_id.clone(),
            id.id.clone(),
            recently_copied == Some(&copy_id),
            t.fg_faint,
            t,
            cx,
        ));
    grid_row(label, value.into_any_element(), GridAlign::Baseline, t)
}
