use chrono::{DateTime, Local, TimeZone};
use gpui::{IntoElement, ParentElement, SharedString, Styled, div, rgb};

use crate::app::fonts::CodeText as _;
use crate::app::theme::Theme;

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
        .code_text(font_size, t)
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
    match secs {
        s if s < 3600 => jayjay_i18n::repo::minutes_ago(s / 60),
        s if s < 86_400 => jayjay_i18n::repo::hours_ago(s / 3600),
        s if s < 604_800 => jayjay_i18n::repo::days_ago(s / 86_400),
        s if s < 2_592_000 => jayjay_i18n::repo::weeks_ago(s / 604_800),
        s if s < 31_536_000 => jayjay_i18n::repo::months_ago(s / 2_592_000),
        s => jayjay_i18n::repo::years_ago(s / 31_536_000),
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
