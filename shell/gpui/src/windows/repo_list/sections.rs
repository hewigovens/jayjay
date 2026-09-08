use gpui::{
    Div, FontWeight, InteractiveElement, ParentElement, StatefulInteractiveElement, Styled, div,
    px, rgb,
};
use jayjay_core::repositories::RepoGroup;

use super::card::repository_card;
use crate::app::config;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::primitives::button;

pub(super) fn recent_section(groups: Vec<RepoGroup>, pinned_paths: &[String], t: &Theme) -> Div {
    if groups.is_empty() {
        return div()
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .text_size(ui_font_size(12.))
            .text_color(rgb(t.fg_dim))
            .child("No Recent Repositories");
    }
    repository_section(
        "Recent Repositories",
        groups,
        RowKind::Recent,
        pinned_paths,
        t,
    )
}

#[derive(Clone, Copy)]
pub(super) enum RowKind {
    Pinned,
    Recent,
}

pub(super) fn repository_section(
    title: &'static str,
    groups: Vec<RepoGroup>,
    kind: RowKind,
    pinned_paths: &[String],
    t: &Theme,
) -> Div {
    let mut rows = div().flex().flex_col().gap(px(10.));
    for (index, group) in groups.into_iter().enumerate() {
        rows = rows.child(repository_card(index, group, kind, pinned_paths, t));
    }

    let mut header = div().flex().items_center().pb(px(2.)).child(
        div()
            .flex_1()
            .text_size(ui_font_size(13.))
            .font_weight(FontWeight::SEMIBOLD)
            .child(title),
    );
    if matches!(kind, RowKind::Recent) {
        header = header.child(
            button("repo-list-clear", "Clear", t, false)
                .debug_selector(|| "repo-list-clear".to_owned())
                .on_click(|_, _, cx| config::update(cx, |cfg| cfg.clear_recent_repos())),
        );
    }
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(header)
        .child(rows)
}
