use gpui::{
    AnyElement, Context, IntoElement, ParentElement, Render, SharedString, Styled, Window, div, px,
    rgb,
};

use super::payload::DagDrag;
use crate::app::theme::{FONT_META, FONT_TAG, ui_font_size};
use crate::ui::icons::glyph;
use crate::ui::primitives::{capsule, icon_chip, icon_label};

pub(crate) struct DagDragGhost {
    drag: DagDrag,
}

impl DagDragGhost {
    pub(in crate::repo::window) fn new(drag: DagDrag) -> Self {
        Self { drag }
    }
}

impl Render for DagDragGhost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = crate::app::theme::theme_for_window(window, cx);
        match &self.drag {
            DagDrag::WorkingCopy => drag_outline(
                capsule("@", t.tag_wc_bg, t.tag_wc_fg, FONT_TAG).into_any_element(),
                t.toggle_active_bg,
            ),
            DagDrag::Bookmark { name, .. } => drag_outline(
                icon_chip(
                    glyph::BOOKMARK,
                    name.clone(),
                    t.tag_bookmark_bg,
                    t.tag_bookmark_fg,
                    t.tag_bookmark_icon,
                    FONT_TAG,
                )
                .into_any_element(),
                t.toggle_active_bg,
            ),
            DagDrag::Change { .. } => div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .px(px(12.))
                .py(px(8.))
                .rounded_full()
                .border_1()
                .border_color(rgb(t.toggle_active_bg))
                .bg(rgb(t.header_bg))
                .text_size(ui_font_size(FONT_META))
                .text_color(rgb(t.fg))
                .opacity(0.92)
                .child(icon_label(
                    glyph::ARROW_UP,
                    SharedString::from(
                        self.drag
                            .source_change()
                            .map(DagDrag::label_for_change)
                            .unwrap_or_default(),
                    ),
                    t.scaled_font_size(12.),
                    t.selected_accent,
                ))
                .into_any_element(),
        }
    }
}

fn drag_outline(child: AnyElement, border: u32) -> AnyElement {
    div()
        .rounded_full()
        .border_1()
        .border_color(rgb(border))
        .opacity(0.9)
        .child(child)
        .into_any_element()
}
