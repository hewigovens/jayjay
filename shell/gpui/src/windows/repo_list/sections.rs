use gpui::{
    Div, FontWeight, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, div, px,
};
use jayjay_core::repositories::{RepoGroup, RepoListGroups};

use super::card::repository_card;
use crate::app::config;
use crate::app::theme::Theme;
use crate::ui::primitives::button;

pub(super) fn repository_sections(groups: RepoListGroups, t: &Theme) -> impl IntoElement {
    let mut sections = div().flex().flex_col().gap(px(18.)).px(px(30.)).py(px(18.));
    if !groups.pinned.is_empty() {
        sections = sections.child(repository_section(
            "Pinned",
            groups.pinned,
            RowKind::Pinned,
            t,
        ));
    }
    if !groups.recent.is_empty() {
        sections = sections.child(repository_section(
            "Recent Repositories",
            groups.recent,
            RowKind::Recent,
            t,
        ));
    }

    div()
        .id("repo-list-scroll")
        .flex()
        .flex_1()
        .min_h_0()
        .flex_col()
        .overflow_y_scroll()
        .scrollbar_width(px(0.))
        .child(sections)
}

#[derive(Clone, Copy)]
pub(super) enum RowKind {
    Pinned,
    Recent,
}

fn repository_section(
    title: &'static str,
    groups: Vec<RepoGroup>,
    kind: RowKind,
    t: &Theme,
) -> Div {
    let mut rows = div().flex().flex_col().gap(px(10.));
    for (index, group) in groups.into_iter().enumerate() {
        rows = rows.child(repository_card(index, group, kind, t));
    }

    let mut header = div().flex().items_center().pb(px(2.)).child(
        div()
            .flex_1()
            .text_size(px(13.))
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
