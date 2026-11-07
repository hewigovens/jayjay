use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use super::body::ConvergeBody;
use super::render::caption;
use crate::app::theme::ui_font_size;
use crate::repo::window::RepoWindow;
use crate::ui::icons::glyph;
use crate::ui::primitives::icon_label;

impl ConvergeBody<'_> {
    pub(super) fn effects(&self, cx: &mut Context<RepoWindow>) -> AnyElement {
        let t = self.t;
        let effects = &self.review.plan().effects;
        let descendants = effects
            .descendants
            .iter()
            .map(|descendant| {
                let subject = if descendant.description.is_empty() {
                    jayjay_i18n::converge::no_description()
                } else {
                    descendant.description.clone()
                };
                let id: String = descendant.commit_id.chars().take(12).collect();
                format!("{id}  {subject}")
            })
            .collect();
        let blocked = effects.blocked_reason.as_ref().map(|reason| {
            let details = self.section(
                "details",
                jayjay_i18n::converge::details(),
                vec![reason.clone()],
                cx,
            );
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(
                    icon_label(
                        glyph::WARNING,
                        jayjay_i18n::converge::blocked(),
                        12.,
                        t.error_fg,
                    )
                    .debug_selector(|| "converge-blocked".to_owned())
                    .text_size(ui_font_size(12.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(t.error_fg)),
                )
                .child(jayjay_i18n::converge::immutable_reason())
                .children(details)
        });
        let lists = [
            (
                "descendants",
                jayjay_i18n::converge::rebase_descendants(effects.descendants.len()),
                descendants,
            ),
            (
                "bookmarks",
                jayjay_i18n::converge::move_bookmarks(effects.bookmarks.len()),
                effects.bookmarks.clone(),
            ),
            (
                "workspaces",
                jayjay_i18n::converge::move_workspaces(effects.workspaces.len()),
                effects.workspaces.clone(),
            ),
        ];
        let no_effects = lists.iter().all(|(_, _, values)| values.is_empty());
        div()
            .debug_selector(|| "converge-effects".to_owned())
            .flex()
            .flex_col()
            .flex_none()
            .gap(px(6.))
            .pt(px(10.))
            .border_t_1()
            .border_color(rgb(t.border))
            .text_size(ui_font_size(11.))
            .children(blocked)
            .children(no_effects.then(|| caption(jayjay_i18n::converge::no_effects(), t)))
            .children(
                lists
                    .into_iter()
                    .filter_map(|(id, title, values)| self.section(id, title, values, cx)),
            )
            .child(
                div()
                    .text_color(rgb(t.fg_faint))
                    .child(jayjay_i18n::converge::undo_hint()),
            )
            .into_any_element()
    }

    fn section(
        &self,
        id: &'static str,
        title: String,
        values: Vec<String>,
        cx: &mut Context<RepoWindow>,
    ) -> Option<AnyElement> {
        if values.is_empty() {
            return None;
        }
        let t = self.t;
        let expanded = self.sheet.expanded.contains(id);
        let caret = if expanded {
            glyph::CARET_DOWN
        } else {
            glyph::CARET_RIGHT
        };
        Some(
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    icon_label(caret, title, 9., t.fg_dim)
                        .id(id)
                        .debug_selector(move || format!("converge-effects-{id}"))
                        .cursor_pointer()
                        .on_click(
                            cx.listener(move |view, _, _, cx| view.toggle_converge_section(id, cx)),
                        ),
                )
                .children(expanded.then(|| {
                    div()
                        .pl(px(15.))
                        .flex()
                        .flex_col()
                        .gap(px(2.))
                        .whitespace_normal()
                        .children(values)
                }))
                .into_any_element(),
        )
    }
}
