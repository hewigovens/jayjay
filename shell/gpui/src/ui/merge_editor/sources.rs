use gpui::{
    AnyElement, Div, Entity, InteractiveElement, IntoElement, ParentElement, SharedString,
    Stateful, Styled, div, px, rgb,
};

use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::{button, button_container};
use crate::ui::text_area::TextArea;

pub(crate) fn merge_base_toggle(
    id: impl Into<SharedString>,
    shows_base: bool,
    t: &Theme,
) -> Stateful<Div> {
    let (icon_glyph, label) = if shows_base {
        (glyph::ARROW_UTURN_BACK, "Back to Left & Right")
    } else {
        (glyph::EYE, "Show Base")
    };
    button_container(id, t, false)
        .gap(px(6.))
        .child(icons::icon(icon_glyph, 13., t.toggle_inactive_fg))
        .child(label)
}

pub(crate) fn merge_source_row(
    panels: impl IntoIterator<Item = AnyElement>,
    height: f32,
    t: &Theme,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .flex_none()
        .h(px(height))
        .border_b_1()
        .border_color(rgb(t.border))
        .children(panels)
        .into_any_element()
}

pub(crate) fn merge_source_panel(
    index: usize,
    scroll_id: SharedString,
    label: &'static str,
    content: Entity<TextArea>,
    action: AnyElement,
    t: &Theme,
) -> AnyElement {
    let mut panel = div().flex().flex_col().flex_1().min_w_0().h_full();
    if index > 0 {
        panel = panel.border_l_1().border_color(rgb(t.border));
    }
    panel
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .h(px(38.))
                .px(px(10.))
                .bg(rgb(t.header_bg))
                .border_b_1()
                .border_color(rgb(t.border))
                .child(
                    div()
                        .flex_1()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_size(px(12.))
                        .child(label),
                )
                .child(action),
        )
        .child(
            div()
                .id(scroll_id)
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .child(content),
        )
        .into_any_element()
}

pub(crate) fn merge_result_mode_button(
    id: impl Into<SharedString>,
    label: &'static str,
    selected: bool,
    t: &Theme,
) -> Stateful<Div> {
    let mut result = button(id, label, t, false).text_size(px(10.));
    if !selected {
        result = result.text_color(rgb(t.fg_dim));
    }
    result
}
