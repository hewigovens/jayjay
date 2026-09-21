use chrono::{DateTime, Local, TimeZone};
use gpui::{IntoElement, ParentElement, SharedString, Styled, div, rgb};

use crate::app::theme::{Theme, ui_font_size};

pub(crate) fn id_cell(
    id: &str,
    short_len: u32,
    prefix_color: u32,
    font_size: f32,
    t: &Theme,
) -> impl IntoElement {
    let (prefix, rest) = split_prefix(id, short_len);
    div()
        .flex()
        .flex_row()
        .flex_none()
        .font_family(crate::app::fonts::mono())
        .text_size(ui_font_size(font_size))
        .child(
            div()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(prefix_color))
                .child(SharedString::from(prefix)),
        )
        .child(
            div()
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(rest)),
        )
}

pub(crate) fn format_when(ts_millis: i64) -> String {
    let dt: DateTime<Local> = match Local.timestamp_millis_opt(ts_millis).single() {
        Some(dt) => dt,
        None => return String::new(),
    };
    dt.format("%Y-%m-%d %H:%M").to_string()
}

pub(crate) fn compact_id_len(short_len: u32) -> usize {
    (short_len as usize).max(jayjay_core::ShortId::LABEL_CHARS)
}

pub(crate) fn compact_id(id: &jayjay_core::ShortId) -> String {
    id.prefix(compact_id_len(id.short_len))
}

/// Split `value` into its shortest-unique-prefix and the remainder at `short_len`.
fn split_prefix(value: &str, short_len: u32) -> (String, String) {
    let n = (short_len as usize).min(value.chars().count());
    (
        value.chars().take(n).collect(),
        value.chars().skip(n).collect(),
    )
}

/// "10 days ago" — coarse relative age for the DAG meta line.
pub(crate) fn format_relative(ts_millis: i64) -> String {
    let dt: DateTime<Local> = match Local.timestamp_millis_opt(ts_millis).single() {
        Some(dt) => dt,
        None => return String::new(),
    };
    // Floor to whole minutes so a fresh change reads "1 minute ago" instead of ticking
    // per second; clamping also reads a clock-skewed future timestamp as "1 minute ago".
    let secs = Local::now().signed_duration_since(dt).num_seconds().max(60);
    let ago = |n: i64, unit: &str| {
        if n == 1 {
            format!("1 {unit} ago")
        } else {
            format!("{n} {unit}s ago")
        }
    };
    match secs {
        s if s < 3600 => ago(s / 60, "minute"),
        s if s < 86_400 => ago(s / 3600, "hour"),
        s if s < 604_800 => ago(s / 86_400, "day"),
        s if s < 2_592_000 => ago(s / 604_800, "week"),
        s if s < 31_536_000 => ago(s / 2_592_000, "month"),
        s => ago(s / 31_536_000, "year"),
    }
}

pub(crate) fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::format_relative;
    use chrono::Local;

    #[test]
    fn format_relative_floors_sub_minute_to_one_minute() {
        let now = Local::now().timestamp_millis();
        assert_eq!(format_relative(now), "1 minute ago");
        assert_eq!(format_relative(now - 30_000), "1 minute ago");
        assert_eq!(format_relative(now - 2 * 3_600_000), "2 hours ago");
    }
}
