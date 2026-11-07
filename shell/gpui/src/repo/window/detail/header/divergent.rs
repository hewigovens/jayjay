use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::ChangeInfo;

use crate::app::theme::{Theme, ui_font_size};
use crate::repo::RepoWindow;
use crate::ui::icons::{glyph, icon};
use crate::ui::primitives::small_button;

pub(super) fn divergent_bar(
    change: &ChangeInfo,
    versions: usize,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let change_id = change.change_id.id.clone();
    div()
        .debug_selector(|| "divergent-bar".to_owned())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(12.))
        .py(px(7.))
        .bg(rgb(t.tag_divergent_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(icon(glyph::GIT_BRANCH, 14., t.tag_divergent_fg))
        .child(
            div()
                .text_size(ui_font_size(12.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(t.tag_divergent_fg))
                .child(jayjay_i18n::converge::detail_versions(versions.max(2))),
        )
        .child(div().flex_1())
        .child(
            small_button("divergent-resolve", jayjay_i18n::converge::resolve(), t)
                .debug_selector(|| "divergent-resolve".to_owned())
                .on_click(cx.listener(move |view, _, _, cx| {
                    view.open_converge(change_id.clone(), cx);
                })),
        )
        .into_any_element()
}
