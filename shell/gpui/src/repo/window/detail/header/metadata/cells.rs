use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, Context, Div, ElementId, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::{ChangeInfo, CommitAuthor, DiffStats};

use crate::app::fonts::CodeText as _;
use crate::app::theme::{FONT_ID, Theme, ui_font_size};
use crate::repo::RepoWindow;
use crate::repo::window::ref_chips::{self, copy_feedback_button};
use crate::ui::primitives::dot_separator;
use crate::ui::primitives::text_tooltip;

pub(super) fn person(person: &CommitAuthor, id: impl Into<ElementId>, t: &Theme) -> AnyElement {
    crate::ui::avatar::with_name(&person.email, &person.name, FONT_ID, t.fg)
        .id(id)
        .flex_none()
        .tooltip(text_tooltip(identity(person)))
        .child(
            div()
                .min_w_0()
                .truncate()
                .text_size(ui_font_size(FONT_ID))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(format!("<{}>", person.email))),
        )
        .into_any_element()
}

pub(super) const BYLINE_AVATARS: usize = 3;
pub(super) const AVATAR_OVERLAP: f32 = 4.;

pub(super) fn byline_authors(change: &ChangeInfo, t: &Theme) -> AnyElement {
    let people: Vec<&CommitAuthor> = std::iter::once(&change.author)
        .chain(&change.co_authors)
        .collect();
    let avatars = people
        .iter()
        .take(BYLINE_AVATARS)
        .enumerate()
        .map(|(index, person)| {
            div()
                .flex_none()
                .rounded_full()
                .border_1()
                .border_color(rgb(t.detail_bg))
                .when(index > 0, |avatar| avatar.ml(px(-AVATAR_OVERLAP)))
                .child(crate::ui::avatar::element(&person.email, &person.name, 14.))
        });
    div()
        .id("detail-author")
        .flex()
        .flex_row()
        .flex_none()
        .items_center()
        .gap(px(5.))
        .child(div().flex().flex_row().flex_none().children(avatars))
        .child(
            div()
                .flex_none()
                .text_size(ui_font_size(FONT_ID))
                .text_color(rgb(t.fg))
                .child(SharedString::from(change.credit_line())),
        )
        .tooltip(text_tooltip(
            people
                .into_iter()
                .map(identity)
                .collect::<Vec<_>>()
                .join("\n"),
        ))
        .into_any_element()
}

fn identity(person: &CommitAuthor) -> String {
    format!("{} <{}>", person.name, person.email)
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
            t.added_fg,
            t,
        ));
    }
    if stats.deletions > 0 {
        row = row.child(diff_stat(
            format!("-{}", stats.deletions),
            "removed",
            stats.deletions,
            spelled_out,
            t.removed_fg,
            t,
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
    t: &Theme,
) -> AnyElement {
    let mut row = div().flex().flex_row().items_baseline().gap(px(3.)).child(
        div()
            .code_text(FONT_ID, t)
            .font_weight(gpui::FontWeight::SEMIBOLD)
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
