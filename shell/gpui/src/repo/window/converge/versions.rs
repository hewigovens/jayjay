use gpui::prelude::FluentBuilder;
use gpui::{AnyElement, Context, FontWeight, IntoElement, ParentElement, Styled, div, px, rgb};
use jayjay_core::ConvergeVersion;
use jayjay_core::converge::ConvergeItem;

use super::body::ConvergeBody;
use super::cards::value_text;
use super::render::{caption, card_frame, heading, link};
use crate::app::fonts::CodeText as _;
use crate::repo::window::{RepoWindow, first_line};
use crate::ui::primitives::capsule;

impl ConvergeBody<'_> {
    pub(super) fn versions(&self, cx: &mut Context<RepoWindow>) -> AnyElement {
        let t = self.t;
        let versions = self.review.versions();
        let side_by_side = versions.len() == 2 && t.font_size <= 14.;
        let cards = versions
            .iter()
            .map(|version| self.version_card(version, cx));
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .gap(px(10.))
                    .child(heading(jayjay_i18n::converge::versions()))
                    .child(caption(jayjay_i18n::converge::versions_hint(), t)),
            )
            .children(
                self.outside_filter
                    .then(|| caption(jayjay_i18n::converge::outside_filter(), t)),
            )
            .child(
                div()
                    .flex()
                    .gap(px(10.))
                    .when(!side_by_side, |row| row.flex_col())
                    .children(cards),
            )
            .into_any_element()
    }

    fn version_card(&self, version: &ConvergeVersion, cx: &mut Context<RepoWindow>) -> AnyElement {
        let t = self.t;
        let letter = self.review.letter(&version.commit_id);
        let mut details = vec![version.author.name.clone()];
        details.extend(version.bookmarks.iter().cloned());
        details.extend(version.workspaces.iter().map(|name| format!("{name}@")));
        if version.is_immutable {
            details.push(jayjay_i18n::converge::immutable());
        }
        let commit_id = version.commit_id.clone();
        let keep = link(
            format!("converge-keep-{letter}"),
            jayjay_i18n::converge::keep(&letter.to_string()),
            self.sheet.is_interactive(),
            t,
            cx,
            move |view, cx| view.update_converge_review(cx, |review| review.keep(&commit_id)),
        );
        card_frame(t)
            .flex_1()
            .min_w_0()
            .gap(px(4.))
            .p(px(10.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(capsule(
                        letter.to_string(),
                        t.toggle_active_bg,
                        t.toggle_active_fg,
                        10.,
                    ))
                    .child(
                        div()
                            .code_text(12., t)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(t.commit_id_prefix))
                            .child(version.commit_id.chars().take(12).collect::<String>()),
                    )
                    .child(div().flex_1())
                    .child(keep),
            )
            .child(
                value_text(
                    first_line(&version.description),
                    ConvergeItem::Description,
                    t,
                )
                .font_weight(FontWeight::SEMIBOLD)
                .whitespace_normal(),
            )
            .child(caption(details.join(" · "), t).whitespace_normal())
            .into_any_element()
    }
}
