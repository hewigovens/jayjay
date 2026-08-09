use std::sync::Arc;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, uniform_list,
};

use crate::app::fonts;
use crate::app::theme::Theme;
use crate::diff::line::{ROW_HEIGHT, content_row, gutter_cell, line_bg_color};
use crate::ui::primitives::button;

use super::view::{ExternalToolState, ExternalToolWindow};

impl ExternalToolWindow {
    pub(super) fn render_diff(&mut self, t: &Theme, cx: &mut Context<Self>) -> AnyElement {
        let ExternalToolState::Diff(session) = &self.state else {
            return div().into_any_element();
        };
        if session.files.is_empty() {
            return div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(t.fg_dim))
                .child("No differences")
                .into_any_element();
        }

        let editable = session.editable;
        let selected_file = session.selected_file;
        let sidebar_items: Vec<_> = session
            .files
            .iter()
            .enumerate()
            .map(|(index, file)| {
                (
                    index,
                    file.hunk.path.clone(),
                    file.keeps_all_changes(),
                    file.keeps_any_changes(),
                )
            })
            .collect();
        let selected = session.selected().map(|file| {
            (
                file.hunk.path.clone(),
                file.selected.len(),
                file.changed.len(),
                file.supports_editing,
                file.executable_changed(),
                file.display.clone(),
            )
        });

        let mut sidebar = div()
            .id("external-files-scroll")
            .flex()
            .flex_col()
            .flex_none()
            .w(px(270.))
            .h_full()
            .overflow_y_scroll()
            .border_r_1()
            .border_color(rgb(t.border));
        for (index, path, keeps_all_changes, keeps_any_changes) in sidebar_items {
            let active = index == selected_file;
            let selector = format!("external-file-{index}");
            let indicator = if !editable {
                ""
            } else if keeps_all_changes {
                "✓"
            } else if !keeps_any_changes {
                "○"
            } else {
                "−"
            };
            sidebar = sidebar.child(
                div()
                    .id(SharedString::from(selector.clone()))
                    .debug_selector(move || selector.clone())
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(7.))
                    .h(px(34.))
                    .px(px(10.))
                    .bg(rgb(if active { t.selected_bg } else { t.detail_bg }))
                    .border_b_1()
                    .border_color(rgb(t.row_border))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(t.row_alt_bg)))
                    .on_click(cx.listener(move |view, _, _, cx| {
                        if let ExternalToolState::Diff(session) = &mut view.state {
                            session.selected_file = index;
                            cx.notify();
                        }
                    }))
                    .child(
                        div()
                            .w(px(14.))
                            .text_color(rgb(t.selected_accent))
                            .child(indicator),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .truncate()
                            .text_size(px(12.))
                            .child(path),
                    ),
            );
        }

        let Some((
            path,
            selected_count,
            changed_count,
            supports_editing,
            executable_changed,
            display,
        )) = selected
        else {
            return sidebar.into_any_element();
        };
        let mut header = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(10.))
            .h(px(38.))
            .px(px(10.))
            .bg(rgb(t.header_bg))
            .border_b_1()
            .border_color(rgb(t.border))
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .truncate()
                    .text_size(px(12.))
                    .child(path.clone()),
            )
            .child(div().text_size(px(11.)).text_color(rgb(t.fg_dim)).child(
                if executable_changed && changed_count == 0 {
                    "Executable permission changed".to_owned()
                } else {
                    format!("{selected_count} of {changed_count} changed lines")
                },
            ));
        if editable {
            if !supports_editing {
                header = header.child(
                    div()
                        .text_size(px(11.))
                        .text_color(rgb(t.fg_dim))
                        .child("Whole-file selection"),
                );
            }
            header = header.child(
                button("external-toggle-file", "Toggle File", t, false)
                    .debug_selector(|| "external-toggle-file".to_owned())
                    .on_click(cx.listener(|view, _, _, cx| {
                        if let ExternalToolState::Diff(session) = &mut view.state {
                            session.toggle_selected_file();
                            cx.notify();
                        }
                    })),
            );
        }

        let row_count = display.lines.len();
        let row_theme = Arc::new(t.clone());
        let row_path = path.clone();
        let row_list_id = SharedString::from(format!("external-lines-{path}"));
        let rows = uniform_list(
            row_list_id,
            row_count,
            cx.processor(move |view, range: std::ops::Range<usize>, _, cx| {
                range
                    .map(|index| {
                        let line = &display.lines[index];
                        let file = match &view.state {
                            ExternalToolState::Diff(session) => session.selected(),
                            _ => None,
                        };
                        let current_file = file.is_some_and(|file| file.hunk.path == row_path);
                        let changed = current_file
                            && file.is_some_and(|file| file.display_to_full.contains_key(&index));
                        let checked = current_file
                            && file.is_some_and(|file| file.is_display_line_selected(index));
                        external_diff_row(
                            index,
                            line,
                            editable && supports_editing && changed,
                            checked,
                            &row_theme,
                            cx,
                        )
                    })
                    .collect()
            }),
        )
        .flex_1()
        .min_h_0();

        div()
            .flex()
            .flex_row()
            .flex_1()
            .min_h_0()
            .child(sidebar)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .child(header)
                    .child(rows),
            )
            .into_any_element()
    }
}

fn external_diff_row(
    index: usize,
    line: &jayjay_core::diff::DiffLine,
    selectable: bool,
    checked: bool,
    t: &Theme,
    cx: &mut Context<ExternalToolWindow>,
) -> AnyElement {
    let bg = line_bg_color(line.style, line.conflict_kind, t);
    let old_no = line
        .old_line_no
        .map(|number| number.to_string())
        .unwrap_or_default();
    let new_no = line
        .new_line_no
        .map(|number| number.to_string())
        .unwrap_or_default();
    let mark = if !selectable {
        ""
    } else if checked {
        "✓"
    } else {
        "○"
    };
    let row = div()
        .id(SharedString::from(format!("external-line-{index}")))
        .flex()
        .flex_row()
        .h(px(ROW_HEIGHT))
        .font_family(fonts::mono())
        .text_size(px(12.))
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .flex_none()
                .w(px(24.))
                .h(px(ROW_HEIGHT))
                .bg(rgb(bg))
                .text_color(rgb(t.selected_accent))
                .child(mark),
        )
        .child(gutter_cell(old_no, t, bg))
        .child(gutter_cell(new_no, t, bg))
        .child(content_row(line, t, None, None, px(7.2)));
    if selectable {
        row.cursor_pointer()
            .on_click(cx.listener(move |view, _, _, cx| {
                if let ExternalToolState::Diff(session) = &mut view.state
                    && let Some(file) = session.selected_mut()
                {
                    file.toggle_line(index);
                    cx.notify();
                }
            }))
            .into_any_element()
    } else {
        row.into_any_element()
    }
}
