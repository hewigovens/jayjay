use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, ParentElement, Styled, div,
    px, rgb,
};
use jayjay_core::ChangeInfo;
use jayjay_core::converge::{ConvergeItem, ConvergeReview};

use super::render::{caption, heading};
use super::sheet::ConvergeSheet;
use crate::app::theme::{Theme, ui_font_size};
use crate::repo::window::{RepoWindow, compact_id, id_cell};
use crate::ui::icons::glyph;
use crate::ui::primitives::icon_label;

pub(super) struct ConvergeBody<'a> {
    pub(super) sheet: &'a ConvergeSheet,
    pub(super) review: &'a ConvergeReview,
    pub(super) t: &'a Theme,
    change_id: (String, u32),
    pub(super) outside_filter: bool,
}

impl<'a> ConvergeBody<'a> {
    pub(super) fn new(
        sheet: &'a ConvergeSheet,
        review: &'a ConvergeReview,
        loaded: &[ChangeInfo],
        t: &'a Theme,
    ) -> Self {
        let id = &review.plan().change_id;
        let change_id = loaded
            .iter()
            .find(|change| &change.change_id.id == id)
            .map(|change| (compact_id(&change.change_id), change.change_id.short_len))
            .unwrap_or_else(|| (id.chars().take(12).collect(), 8));
        let outside_filter = review.versions().iter().any(|version| {
            !loaded
                .iter()
                .any(|change| change.commit_id.id == version.commit_id)
        });
        Self {
            sheet,
            review,
            t,
            change_id,
            outside_filter,
        }
    }

    pub(super) fn summary(&self) -> AnyElement {
        let t = self.t;
        let unresolved = self.review.unresolved_items().len();
        let (glyph, color, label, selector) = if unresolved == 0 {
            let label = jayjay_i18n::converge::ready();
            (glyph::CHECK, t.success_fg, label, "converge-ready")
        } else {
            let label = jayjay_i18n::converge::unresolved_count(unresolved);
            (glyph::WARNING, t.warning_fg, label, "converge-unresolved")
        };
        let (id, short_len) = &self.change_id;
        div()
            .flex()
            .items_center()
            .gap(px(12.))
            .pb(px(10.))
            .border_b_1()
            .border_color(rgb(t.border))
            .child(id_cell(id, *short_len, t.change_id_prefix, 12., t))
            .child(caption(
                jayjay_i18n::converge::version_summary(self.review.versions().len()),
                t,
            ))
            .child(div().flex_1())
            .child(
                icon_label(glyph, label, 11., color)
                    .debug_selector(move || selector.to_owned())
                    .text_size(ui_font_size(11.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(color)),
            )
            .into_any_element()
    }

    pub(super) fn sections(&self, cx: &mut Context<RepoWindow>) -> Vec<AnyElement> {
        let mut sections = vec![self.versions(cx), self.choices_heading()];
        sections.extend(
            self.review
                .card_items()
                .into_iter()
                .map(|item| self.choice_card(item, cx)),
        );
        sections.extend(self.same_rows());
        sections.push(self.effects(cx));
        sections
    }

    fn choices_heading(&self) -> AnyElement {
        let automatic = jayjay_i18n::converge::decided_automatically(
            &self.review.automatic_count().to_string(),
            &ConvergeItem::ALL.len().to_string(),
        );
        div()
            .flex()
            .items_center()
            .child(heading(jayjay_i18n::converge::choices()))
            .child(div().flex_1())
            .child(caption(automatic, self.t))
            .into_any_element()
    }
}
