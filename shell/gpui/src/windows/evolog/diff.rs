use std::sync::Arc;

use gpui::{
    AnyElement, AppContext, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, div, px, rgb, uniform_list,
};
use jayjay_core::DiffHunk;
use jayjay_core::diff::{
    DEFAULT_WRAP_COLS, FileDiff, build_diff_display_lines, compute_file_diff, wrap_diff_lines,
};

use super::{EvologView, placeholder, placeholder_err};
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::diff::line::{content_row, gutter_cell, line_bg_color};
use crate::diff::{hunk_is_image, image_diff_view};
use crate::ui::icons::{glyph, icon};
use crate::ui::primitives::no_scrollbar_gutter;

impl EvologView {
    pub fn selected_diff_path(&self) -> Option<&str> {
        self.current_hunk.as_ref().map(|hunk| hunk.path.as_str())
    }

    pub(super) fn load_interdiff(&mut self, cx: &mut Context<Self>) {
        self.selection_generation = self.selection_generation.wrapping_add(1);
        let generation = self.selection_generation;
        self.file_generation = self.file_generation.wrapping_add(1);
        self.files = None;
        self.selected_file_ix = None;
        self.current_hunk = None;
        self.current_diff = None;
        self.diff_error = None;
        self.diff_loading = false;
        let Some((from, to)) = self.selected_endpoints() else {
            return;
        };
        if from == to {
            self.files = Some(Arc::new(Vec::new()));
            return;
        }
        self.diff_loading = true;
        let repo = self.repo.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.interdiff_summary(&from, &to) })
                .await;
            let _ = this.update(cx, move |view, cx| {
                if view.selection_generation != generation {
                    return;
                }
                view.diff_loading = false;
                match result {
                    Ok(detail) => {
                        let files = Arc::new(detail.diff);
                        view.files = Some(files.clone());
                        if !files.is_empty() {
                            view.selected_file_ix = Some(0);
                            view.load_file(0, cx);
                        }
                    }
                    Err(error) => view.diff_error = Some(format!("{error}").into()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn load_file(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some((from, to)) = self.selected_endpoints() else {
            return;
        };
        let Some(path) = self
            .files
            .as_deref()
            .and_then(|files| files.get(index))
            .map(|hunk| hunk.path.clone())
        else {
            return;
        };
        self.selected_file_ix = Some(index);
        self.current_hunk = None;
        self.current_diff = None;
        self.diff_error = None;
        self.diff_loading = true;
        self.file_generation = self.file_generation.wrapping_add(1);
        let generation = self.file_generation;
        let repo = self.repo.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    repo.interdiff_file(&from, &to, &path).map(|hunk| {
                        let old = hunk.old.content.clone().unwrap_or_default();
                        let new = hunk.new.content.clone().unwrap_or_default();
                        let diff = compute_file_diff(&hunk.path, &old, &new, false);
                        (hunk, diff)
                    })
                })
                .await;
            let _ = this.update(cx, move |view, cx| {
                if view.file_generation != generation {
                    return;
                }
                view.diff_loading = false;
                match result {
                    Ok((hunk, diff)) => {
                        view.current_hunk = Some(hunk);
                        view.current_diff = Some(Arc::new(diff));
                    }
                    Err(error) => view.diff_error = Some(format!("{error}").into()),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

pub(super) fn comparison(
    view: &EvologView,
    theme: &Theme,
    cx: &mut Context<EvologView>,
) -> AnyElement {
    let Some((from, to)) = view.selected_endpoints() else {
        return placeholder(
            "Select a version to compare it with the latest version",
            theme,
        );
    };
    let file_count = view.files.as_deref().map(Vec::len);
    let content = if let Some(error) = view.diff_error.as_ref() {
        placeholder_err(error, theme)
    } else if view.diff_loading && view.files.is_none() {
        placeholder("Loading diff…", theme)
    } else if let Some(files) = view.files.clone() {
        if files.is_empty() {
            placeholder("No changes between the selected versions", theme)
        } else {
            comparison_content(view, files, theme, cx)
        }
    } else {
        placeholder(
            "Select a version to compare it with the latest version",
            theme,
        )
    };

    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .child(comparison_header(
            &from,
            &to,
            file_count,
            view.can_reverse_comparison(),
            theme,
            cx,
        ))
        .child(content)
        .into_any_element()
}

fn comparison_content(
    view: &EvologView,
    files: Arc<Vec<DiffHunk>>,
    theme: &Theme,
    cx: &mut Context<EvologView>,
) -> AnyElement {
    let selected_file_ix = view.selected_file_ix;
    let files_for_list = files.clone();
    let file_theme = Arc::new(theme.clone());
    let file_list = uniform_list(
        "evolog-files",
        files.len(),
        cx.processor(move |_view, range: std::ops::Range<usize>, _window, cx| {
            range
                .map(|ix| {
                    let path: SharedString = files_for_list[ix].path.clone().into();
                    let row_theme = file_theme.clone();
                    div()
                        .id(("evolog-file", ix))
                        .debug_selector({
                            let path = path.clone();
                            move || format!("evolog-file-{path}")
                        })
                        .w_full()
                        .px_2()
                        .py_1()
                        .text_size(px(11.))
                        .text_color(rgb(row_theme.fg))
                        .bg(rgb(if selected_file_ix == Some(ix) {
                            row_theme.selected_bg
                        } else {
                            row_theme.detail_bg
                        }))
                        .hover(move |style| style.bg(rgb(row_theme.row_alt_bg)))
                        .cursor_pointer()
                        .on_click(cx.listener(move |view, _: &ClickEvent, _, cx| {
                            view.load_file(ix, cx);
                        }))
                        .child(path)
                })
                .collect()
        }),
    );

    let diff = if let Some(hunk) = view
        .current_hunk
        .as_ref()
        .filter(|hunk| hunk_is_image(hunk))
    {
        image_diff_view(hunk, theme)
    } else if let Some(diff) = view.current_diff.clone() {
        read_only_diff(diff, theme, cx)
    } else if view.diff_loading {
        placeholder("Loading file…", theme)
    } else {
        placeholder("Select a file", theme)
    };
    div()
        .flex()
        .flex_row()
        .flex_1()
        .min_h_0()
        .child(
            div()
                .w(px(220.))
                .h_full()
                .border_r_1()
                .border_color(rgb(theme.border))
                .child(no_scrollbar_gutter(file_list).h_full()),
        )
        .child(diff)
        .into_any_element()
}

fn comparison_header(
    from: &str,
    to: &str,
    file_count: Option<usize>,
    can_reverse: bool,
    theme: &Theme,
    cx: &mut Context<EvologView>,
) -> AnyElement {
    let mut header = div()
        .id("evolog-compare-banner")
        .debug_selector(|| "evolog-compare-banner".to_owned())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(14.))
        .py(px(8.))
        .bg(rgb(theme.compare_bg))
        .border_b_1()
        .border_color(rgb(theme.border))
        .child(comparison_direction_button(can_reverse, theme, cx))
        .child(
            div()
                .text_size(px(12.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(theme.fg))
                .child("Comparing Versions"),
        )
        .child(version_label(from, theme))
        .child(icon(glyph::ARROW_RIGHT, 10., theme.fg_dim))
        .child(version_label(to, theme))
        .child(div().flex_1());

    if let Some(file_count) = file_count {
        let label = if file_count == 1 {
            "1 file changed".to_owned()
        } else {
            format!("{file_count} files changed")
        };
        header = header.child(
            div()
                .text_size(px(11.))
                .text_color(rgb(theme.fg_dim))
                .child(label),
        );
    }

    header.into_any_element()
}

fn comparison_direction_button(
    can_reverse: bool,
    theme: &Theme,
    cx: &mut Context<EvologView>,
) -> AnyElement {
    let button = div()
        .id("evolog-compare-reverse")
        .debug_selector(|| "evolog-compare-reverse".to_owned())
        .flex()
        .items_center()
        .justify_center()
        .size(px(20.))
        .rounded_md()
        .child(icon(
            glyph::ARROWS_LEFT_RIGHT,
            17.,
            if can_reverse {
                theme.compare_accent
            } else {
                theme.fg_faint
            },
        ));

    if can_reverse {
        button
            .cursor_pointer()
            .hover(|style| style.bg(rgb(theme.row_alt_bg)))
            .on_click(cx.listener(|view, _, _, cx| view.reverse_comparison(cx)))
            .into_any_element()
    } else {
        button.into_any_element()
    }
}

fn version_label(commit_id: &str, theme: &Theme) -> AnyElement {
    div()
        .max_w(px(180.))
        .overflow_hidden()
        .font_family(fonts::mono())
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_size(px(12.))
        .text_color(rgb(theme.fg))
        .child(commit_id.chars().take(12).collect::<String>())
        .into_any_element()
}

fn read_only_diff(diff: Arc<FileDiff>, theme: &Theme, cx: &mut Context<EvologView>) -> AnyElement {
    let display_lines = build_diff_display_lines(&diff.lines);
    let lines = Arc::new(wrap_diff_lines(&display_lines, DEFAULT_WRAP_COLS));
    let count = lines.len();
    let theme = Arc::new(theme.clone());
    let advance = fonts::mono_advance(cx, px(12.));
    let list = uniform_list(
        "evolog-diff",
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
