use gpui::{
    AnyElement, Context, Div, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::{ChangeInfo, DiffStats};

use crate::app::fonts;
use crate::app::theme::{FONT_ID, Theme, ui_font_size};
use crate::repo::RepoWindow;
use crate::repo::window::ref_chips::{self, copy_feedback_button};
use crate::ui::primitives::dot_separator;
use crate::ui::primitives::text_tooltip;

pub(super) fn author(change: &ChangeInfo, shows_email: bool, t: &Theme) -> AnyElement {
    let mut row =
        crate::ui::avatar::with_name(&change.author.email, &change.author.name, FONT_ID, t.fg)
            .id("detail-author")
            .flex_none()
            .tooltip(text_tooltip(format!(
                "{} <{}>",
                change.author.name, change.author.email
            )));
    if shows_email {
        row = row.child(
            div()
                .min_w_0()
                .truncate()
                .text_size(ui_font_size(FONT_ID))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(format!("<{}>", change.author.email))),
        );
    }
    row.into_any_element()
}

pub(super) fn separator(t: &Theme) -> AnyElement {
    dot_separator(t)
        .text_size(ui_font_size(FONT_ID))
        .into_any_element()
}

pub(super) fn diff_stats(stats: &DiffStats, spelled_out: bool, t: &Theme) -> Option<AnyElement> {
    if stats.insertions == 0 && stats.deletions == 0 {
        return None;
    }
    let mut row = div()
        .flex()
        .flex_row()
        .items_baseline()
        .gap(px(if spelled_out { 8. } else { 4. }));
    if stats.insertions > 0 {
        row = row.child(diff_stat(
            format!("+{}", stats.insertions),
            "added",
            stats.insertions,
            spelled_out,
            t.diff_gutter_added_fg,
        ));
    }
    if stats.deletions > 0 {
        row = row.child(diff_stat(
            format!("-{}", stats.deletions),
            "removed",
            stats.deletions,
            spelled_out,
            t.diff_gutter_removed_fg,
        ));
    }
    Some(row.into_any_element())
}

pub(super) fn diff_stat(
    value: String,
    caption: &str,
    count: u32,
    spelled_out: bool,
    color: u32,
) -> AnyElement {
    let mut row = div().flex().flex_row().items_baseline().gap(px(3.)).child(
        div()
            .font_family(fonts::mono())
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_size(ui_font_size(FONT_ID))
            .text_color(rgb(color))
            .child(SharedString::from(value)),
    );
    if spelled_out {
        row = row.child(
            div()
                .text_size(ui_font_size(FONT_ID))
                .text_color(rgb(color))
                .child(format!(
                    "{caption} {}",
                    if count == 1 { "line" } else { "lines" }
                )),
        );
    }
    row.into_any_element()
}

pub(super) fn bookmark_chip(
    name: &str,
    conflicted: bool,
    shows_copy: bool,
    recently_copied: Option<&SharedString>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let chip = ref_chips::bookmark_chip(name.to_owned().into(), conflicted, false, FONT_ID, t);
    if !shows_copy {
        return chip.into_any_element();
    }
    let id: SharedString = format!("bookmark:{name}").into();
    let just_copied = recently_copied == Some(&id);
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.))
        .child(chip)
        .child(copy_feedback_button(
            id,
            name.to_owned(),
            just_copied,
            t.fg_faint,
            t,
            cx,
        ))
        .into_any_element()
}

pub(super) fn tag_chip(name: &str, t: &Theme) -> Div {
    ref_chips::tag_chip(name.to_owned().into(), FONT_ID, t)
}
