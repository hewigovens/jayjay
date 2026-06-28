use std::sync::Arc;

use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, uniform_list,
};
use jayjay_core::OpLogEntry;

use super::OperationLogView;
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::{capsule, no_scrollbar_gutter};

pub(super) fn operation_list(
    entries: Arc<Vec<OpLogEntry>>,
    selected_id: Option<String>,
    theme: Theme,
    cx: &mut Context<OperationLogView>,
) -> AnyElement {
    let count = entries.len();
    let theme = Arc::new(theme);
    let list = uniform_list(
        "operation-log",
        count,
        cx.processor(move |_this, range: std::ops::Range<usize>, _w, cx| {
            range
                .map(|ix| {
                    let entry = entries[ix].clone();
                    let selected = selected_id.as_deref() == Some(entry.id.as_str());
                    operation_row(entry, selected, theme.clone(), cx)
                })
                .collect()
        }),
    );
    no_scrollbar_gutter(list).h_full().into_any_element()
}

fn operation_row(
    entry: OpLogEntry,
    selected: bool,
    t: Arc<Theme>,
    cx: &mut Context<OperationLogView>,
) -> AnyElement {
    let id = entry.id.id.clone();
    let select_id = id.clone();
    let description = description_label(&entry.description);
    let glyph_str = operation_glyph(&entry.description);
    let is_current = entry.is_current;
    let timestamp = entry.timestamp.clone();
    let short_id = id.chars().take(12).collect::<String>();
    let n = (entry.id.short_len as usize).min(short_id.len());
    let id_prefix = short_id[..n].to_owned();
    let id_rest = short_id[n..].to_owned();
    let (bg, fg) = if selected {
        (t.selected_bg, t.fg)
    } else {
        (t.detail_bg, t.fg)
    };

    div()
        .id(SharedString::from(format!("operation-log-row-{id}")))
        .debug_selector(move || format!("operation-log-row-{id}"))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.))
        .w_full()
        .min_w_0()
        .px(px(16.))
        .py(px(8.))
        .bg(rgb(bg))
        .border_b_1()
        .border_color(rgb(t.row_border))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_click(cx.listener(move |view, _: &ClickEvent, _, cx| {
            view.select_operation(select_id.clone(), cx);
        }))
        .child(icons::icon(glyph_str, 14., t.fg_dim))
        .child(operation_text(
            description,
            timestamp,
            is_current,
            fg,
            &t,
            cx,
        ))
        .child(operation_id(id_prefix, id_rest, is_current, &t))
        .into_any_element()
}

fn operation_text(
    description: String,
    timestamp: String,
    is_current: bool,
    fg: u32,
    t: &Theme,
    cx: &mut Context<OperationLogView>,
) -> AnyElement {
    let description_width = text_width(&description, px(13.), cx);
    let description_selector = if is_current {
        "operation-log-current-description"
    } else {
        "operation-log-description"
    };
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .gap(px(3.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.))
                .min_w_0()
                .child(
                    div()
                        .debug_selector(move || description_selector.to_owned())
                        .w(description_width)
                        .flex_shrink_1()
                        .min_w_0()
                        .truncate()
                        .text_size(px(13.))
                        .text_color(rgb(fg))
                        .child(SharedString::from(description)),
                )
                .child(current_badge(is_current, t)),
        )
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(px(11.))
                .text_color(rgb(t.fg_faint))
                .child(SharedString::from(timestamp)),
        )
        .into_any_element()
}

fn operation_id(id_prefix: String, id_rest: String, is_current: bool, t: &Theme) -> AnyElement {
    let selector = if is_current {
        "operation-log-current-id"
    } else {
        "operation-log-id"
    };
    div()
        .debug_selector(move || selector.to_owned())
        .flex()
        .flex_row()
        .flex_none()
        .font_family(fonts::mono())
        .text_size(px(11.))
        .child(
            div()
                .text_color(rgb(t.change_id_prefix))
                .child(SharedString::from(id_prefix)),
        )
        .child(
            div()
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(id_rest)),
        )
        .into_any_element()
}

fn current_badge(is_current: bool, t: &Theme) -> AnyElement {
    if !is_current {
        return div().into_any_element();
    }
    div()
        .debug_selector(|| "operation-log-current-badge".to_owned())
        .flex_none()
        .child(capsule(
            "current",
            t.toggle_active_bg,
            t.toggle_active_fg,
            9.,
        ))
        .into_any_element()
}

fn text_width(
    text: &str,
    font_size: gpui::Pixels,
    cx: &mut Context<OperationLogView>,
) -> gpui::Pixels {
    let font_id = cx.text_system().resolve_font(&gpui::font(".SystemUIFont"));
    let width = text
        .chars()
        .map(|ch| cx.text_system().layout_width(font_id, font_size, ch))
        .fold(px(0.), |acc, width| acc + width);
    px(f32::from(width).ceil() + 2.)
}

fn description_label(description: &str) -> String {
    let trimmed = description.trim();
    if trimmed.is_empty() {
        "(no description)".to_owned()
    } else {
        trimmed.lines().next().unwrap_or("").trim().to_owned()
    }
}

fn operation_glyph(description: &str) -> &'static str {
    let description = description.to_ascii_lowercase();
    if description.contains("bookmark") {
        glyph::BOOKMARK
    } else if description.contains("rebase")
        || description.contains("parallelize")
        || description.contains("merge")
    {
        glyph::GIT_BRANCH
    } else if description.contains("restore") || description.contains("undo") {
        glyph::ARROW_CIRCLE_RIGHT
    } else if description.contains("fetch") {
        glyph::ARROW_DOWN
    } else if description.contains("push") {
        glyph::ARROW_UP
    } else if description.contains("commit") {
        glyph::CHECK
    } else if description.contains("new ") {
        glyph::PLUS_CIRCLE
    } else if description.contains("describe") || description.contains("edit") {
        glyph::PENCIL_CIRCLE
    } else {
        glyph::ARROW_CLOCKWISE
    }
}
