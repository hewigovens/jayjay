use super::DescriptionState;
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Pixels, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, svg,
};
use jayjay_core::ChangeInfo;

use crate::app::fonts;
use crate::app::theme::{HEADER_HEIGHT, Theme, ui_font_size};
use crate::diff::DETAIL_INSET;
use crate::diff::file_row_height;
use crate::repo::window::dag_row::{first_line, text_line_height};
use crate::repo::window::{FocusStop, RepoWindow, focus_ring};
use crate::ui::icons::{self, glyph, icon};
use crate::ui::primitives::text_tooltip;

const COLLAPSED_HEIGHT: f32 = 80.;
const BODY_FONT: f32 = 12.;

/// The expanded body still leaves most of the pane to the diff, so its cap follows the window instead of a fixed size.
pub(super) fn expanded_height(viewport_height: Pixels) -> Pixels {
    (viewport_height * 0.3).max(px(COLLAPSED_HEIGHT * 2.))
}

fn whole_lines(cap: f32, line_height: f32) -> f32 {
    (cap / line_height).floor().max(1.) * line_height
}

pub(super) fn description_block(
    change: &ChangeInfo,
    state: &DescriptionState,
    focused: Option<FocusStop>,
    expanded_height: Pixels,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let title = first_line(&change.description);
    let has_description = !change.description.trim().is_empty();
    let body = jayjay_core::commit_message::body(&change.description);
    let has_body = !body.is_empty();
    let expanded = state.expanded;

    let line_height = text_line_height(t, BODY_FONT);
    let collapsed_cap = whole_lines(COLLAPSED_HEIGHT.min(file_row_height(t) - 12.), line_height);
    let cap = if expanded {
        whole_lines(f32::from(expanded_height), line_height)
    } else {
        collapsed_cap
    };

    let scroll = div()
        .id(format!(
            "description-body-{}-{expanded}",
            change.commit_id.id
        ))
        .debug_selector(|| "description-body".to_owned())
        .flex()
        .flex_col()
        .min_w_0()
        .min_h_0()
        .max_h(px(cap))
        .overflow_y_scroll()
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(ui_font_size(BODY_FONT))
                .text_color(rgb(t.fg))
                .debug_selector(|| "description-text".to_owned())
                .child(SharedString::from(body)),
        );
    let body_section = div()
        .flex_shrink_0()
        .py(px(6.))
        .when(!expanded, |el| el.h(px(file_row_height(t))))
        .child(scroll);

    let toggle = focus_ring(
        div().id("description-expansion"),
        focused == Some(FocusStop::ExpandDescription),
        t,
    )
    .debug_selector(|| "description-expansion".to_owned())
    .size(px(22.))
    .flex_shrink_0()
    .flex()
    .items_center()
    .justify_center()
    .cursor_pointer()
    .child(
        svg()
            .path(if expanded {
                icons::COLLAPSE_VERTICAL_SVG
            } else {
                icons::EXPAND_VERTICAL_SVG
            })
            .size(px(18.))
            .text_color(rgb(t.fg_dim)),
    )
    .tooltip(text_tooltip(if expanded {
        "Collapse description"
    } else {
        "Expand description"
    }))
    .on_click(cx.listener(|view, _, _, cx| {
        view.description.expanded = !view.description.expanded;
        cx.notify();
    }));

    let header = div()
        .flex()
        .items_center()
        .gap(px(8.))
        .flex_shrink_0()
        .py(px(6.))
        .min_h(px(t.scaled_font_size(HEADER_HEIGHT)))
        .when(has_description, |el| {
            el.child(
                div()
                    .debug_selector(|| "description-title".to_owned())
                    .min_w_0()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_size(ui_font_size(20.))
                    .line_height(px(t.scaled_font_size(HEADER_HEIGHT - 12.)))
                    .text_color(rgb(t.fg))
                    .child(SharedString::from(title)),
            )
        })
        .when(!has_description, |el| {
            el.child(
                div()
                    .debug_selector(|| "description-empty".to_owned())
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_size(ui_font_size(20.))
                    .line_height(px(t.scaled_font_size(HEADER_HEIGHT - 12.)))
                    .text_color(rgb(t.fg_faint))
                    .child("No description"),
            )
        })
        .when_some(edit_button(change, focused, t, cx), |el, button| {
            el.child(button)
        })
        .child(toggle);

    div()
        .flex()
        .flex_col()
        .min_h_0()
        .debug_selector(|| "detail-description".to_owned())
        .px(px(DETAIL_INSET))
        .child(header)
        .when(has_body, |el| el.child(body_section))
        .into_any_element()
}

fn edit_button(
    change: &ChangeInfo,
    focused: Option<FocusStop>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Option<AnyElement> {
    if change.is_immutable || change.is_working_copy {
        return None;
    }

    let is_empty = change.description.trim().is_empty();
    Some(
        focus_ring(
            div().id(SharedString::from("edit-description")),
            focused == Some(FocusStop::EditDescription),
            t,
        )
        .debug_selector(|| "edit-description".to_owned())
        .flex()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .gap(px(4.))
        .text_size(ui_font_size(12.))
        .text_color(rgb(t.fg_dim))
        .rounded_md()
        .child(icon(glyph::PENCIL, 12., t.fg_dim))
        .child(if is_empty { "Add description" } else { "Edit" })
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .tooltip(text_tooltip(if is_empty {
            "Add description"
        } else {
            "Edit description"
        }))
        .on_click(cx.listener(|view, _, _, cx| view.edit_selected_description(cx)))
        .into_any_element(),
    )
}

#[cfg(test)]
mod tests {
    use super::whole_lines;

    #[test]
    fn whole_lines_snap_the_cap_down_to_full_lines() {
        assert_eq!(whole_lines(34., 19.5), 19.5);
        assert_eq!(whole_lines(300., 19.5), 292.5);
        assert_eq!(whole_lines(10., 19.5), 19.5);
    }
}
