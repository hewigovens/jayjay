use std::sync::Arc;

use gpui::{AnyElement, Context, IntoElement, ParentElement, Styled, div, px, uniform_list};
use jayjay_core::diff::{DEFAULT_WRAP_COLS, FileDiff, build_diff_display_lines, wrap_diff_lines};

use crate::app::fonts;
use crate::app::theme::Theme;
use crate::diff::line::{content_row, gutter_cell, line_bg_color};
use crate::ui::primitives::no_scrollbar_gutter;

pub(crate) fn read_only_diff<V: 'static>(
    id: &'static str,
    diff: Arc<FileDiff>,
    theme: &Theme,
    cx: &mut Context<V>,
) -> AnyElement {
    let display_lines = build_diff_display_lines(&diff.lines);
    let lines = Arc::new(wrap_diff_lines(&display_lines, DEFAULT_WRAP_COLS, 0));
    let count = lines.len();
    let theme = Arc::new(theme.clone());
    let advance = fonts::mono_advance(cx, px(theme.code_font_size()));
    let list = uniform_list(
        id,
        count,
        cx.processor(move |_view, range: std::ops::Range<usize>, _window, _cx| {
            range
                .map(|ix| {
                    let line = &lines[ix].line;
                    let bg = line_bg_color(line.style, line.conflict_kind, &theme);
                    div()
                        .flex()
                        .flex_row()
                        .w_full()
                        .child(gutter_cell(
                            line.old_line_no
                                .map(|line| line.to_string())
                                .unwrap_or_default(),
                            &theme,
                            bg,
                        ))
                        .child(gutter_cell(
                            line.new_line_no
                                .map(|line| line.to_string())
                                .unwrap_or_default(),
                            &theme,
                            bg,
                        ))
                        .child(content_row(line, &theme, None, None, advance).flex_1())
                })
                .collect()
        }),
    );
    no_scrollbar_gutter(list)
        .flex_1()
        .min_w_0()
        .h_full()
        .into_any_element()
}
