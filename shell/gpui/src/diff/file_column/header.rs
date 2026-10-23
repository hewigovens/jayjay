use gpui::{
    AnyElement, ClickEvent, Context, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::config;
use crate::app::theme::{HEADER_HEIGHT, Theme, ui_font_size};
use crate::repo::window::{FocusStop, RepoWindow, focus_ring};
use crate::ui::icons::glyph;
use crate::ui::input::{LineInput, line_input_content};
use crate::ui::primitives::{icon_button, text_tooltip};

pub(super) struct FileHeaderState {
    pub reviewed: usize,
    pub agent_marked: usize,
    pub count: usize,
    pub visible_count: usize,
    pub loading: bool,
    pub show_review: bool,
    pub hide_reviewed: bool,
    pub active_note_count: usize,
    pub notes_only: bool,
    pub file_filter_active: bool,
    pub file_filter_has_query: bool,
    pub tree_mode: bool,
    pub focused: Option<FocusStop>,
}

pub(super) fn file_column_header(
    state: FileHeaderState,
    cx: &mut Context<RepoWindow>,
    t: &Theme,
) -> impl IntoElement {
    let FileHeaderState {
        reviewed,
        agent_marked,
        count,
        visible_count,
        loading,
        show_review,
        hide_reviewed,
        active_note_count,
        notes_only,
        file_filter_active,
        file_filter_has_query,
        tree_mode,
        focused,
    } = state;
    let shows_reviewed = show_review && reviewed > 0;
    let label = if loading {
        jayjay_i18n::repo::loading_ellipsis()
    } else if file_filter_has_query {
        jayjay_i18n::repo::files_filtered(visible_count, count)
    } else if shows_reviewed {
        jayjay_i18n::diff::reviewed_of_total(reviewed, count)
    } else {
        jayjay_i18n::repo::file_count(visible_count)
    };
    let (tree_glyph, tree_help) = if tree_mode {
        (
            glyph::LIST_TREE,
            jayjay_i18n::repo::showing_files_as_a_tree(),
        )
    } else {
        (
            glyph::LIST,
            jayjay_i18n::repo::showing_files_as_a_flat_list(),
        )
    };
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(2.))
        .min_h(px(t.scaled_font_size(HEADER_HEIGHT)))
        .py(px(6.))
        .px(px(14.))
        .bg(rgb(t.header_bg))
        .debug_selector(|| "file-column-header".to_owned());
    if !file_filter_active {
        row = row.border_b_1().border_color(rgb(t.border));
    }

    if show_review && active_note_count > 0 {
        let (bg, fg) = if notes_only {
            (t.toggle_active_bg, t.toggle_active_fg)
        } else {
            (t.toggle_inactive_bg, t.toggle_inactive_fg)
        };
        row = row.child(
            div()
                .id("file-notes-only")
                .debug_selector(|| "file-notes-only".to_owned())
                .flex()
                .flex_row()
                .items_center()
                .gap(px(3.))
                .h(px(t.scaled_control_height(22., 10.)))
                .px(px(6.))
                .rounded_md()
                .bg(rgb(bg))
                .text_size(ui_font_size(10.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(fg))
                .cursor_pointer()
                .hover(|s| s.bg(rgb(t.row_alt_bg)))
                .on_click(cx.listener(|view, _event: &ClickEvent, _window, cx| {
                    view.toggle_notes_only_files(cx);
                }))
                .tooltip(text_tooltip(if notes_only {
                    jayjay_i18n::repo::showing_only_files_with_review_notes()
                } else {
                    jayjay_i18n::repo::show_only_files_with_review_notes()
                }))
                .child(SharedString::from(format!("\u{25cf} {active_note_count}"))),
        );
    }

    if shows_reviewed {
        row = row.child(
            icon_button(
                "file-split-reviewed",
                glyph::GIT_BRANCH,
                14.,
                24.,
                22.,
                t.fg_dim,
                t,
            )
            .debug_selector(|| "file-split-reviewed".to_owned())
            .tooltip(text_tooltip(jayjay_i18n::repo::split_checked_files(
                reviewed,
            )))
            .on_click(cx.listener(|view, _event: &ClickEvent, _window, cx| {
                view.open_reviewed_files_split_modal(cx);
            })),
        );
    }

    // The eye stays visible while the filter is on even once nothing is reviewed, or the toggle would be unrecoverable.
    if show_review && (reviewed > 0 || hide_reviewed) {
        let (bg, fg) = if hide_reviewed {
            (t.toggle_active_bg, t.toggle_active_fg)
        } else {
            (t.toggle_inactive_bg, t.toggle_inactive_fg)
        };
        row = row.child(
            icon_button(
                "file-hide-reviewed",
                if hide_reviewed {
                    glyph::EYE_OFF
                } else {
                    glyph::EYE
                },
                14.,
                24.,
                22.,
                fg,
                t,
            )
            .debug_selector(|| "file-hide-reviewed".to_owned())
            .bg(rgb(bg))
            .tooltip(text_tooltip(if hide_reviewed {
                jayjay_i18n::repo::showing_only_unreviewed_files()
            } else {
                jayjay_i18n::settings::hide_reviewed()
            }))
            .on_click(cx.listener(|view, _event: &ClickEvent, _window, cx| {
                view.toggle_hide_reviewed_files(cx);
            })),
        );
    }

    let (bg, fg) = if tree_mode {
        (t.toggle_active_bg, t.toggle_active_fg)
    } else {
        (t.toggle_inactive_bg, t.toggle_inactive_fg)
    };
    row = row.child(
        focus_ring(
            icon_button("toggle-file-tree", tree_glyph, 14., 24., 22., fg, t),
            focused == Some(FocusStop::TreeToggle),
            t,
        )
        .debug_selector(|| "toggle-file-tree".to_owned())
        .bg(rgb(bg))
        .tooltip(text_tooltip(tree_help))
        .on_click(|_, _, cx| config::update(cx, |c| c.diff.tree_file_list ^= true)),
    );

    let (bg, fg) = if file_filter_active {
        (t.toggle_active_bg, t.toggle_active_fg)
    } else {
        (t.toggle_inactive_bg, t.toggle_inactive_fg)
    };
    let count_label = div()
        .id("file-count")
        .flex_none()
        .text_size(ui_font_size(12.))
        .text_color(rgb(t.fg_dim))
        .child(SharedString::from(label));
    let count_label = if shows_reviewed {
        count_label
            .debug_selector(|| "file-reviewed-count".to_owned())
            .tooltip(text_tooltip(if agent_marked > 0 {
                jayjay_i18n::diff::files_reviewed_with_agent(reviewed, count, agent_marked)
            } else {
                jayjay_i18n::repo::files_reviewed_progress(reviewed, count)
            }))
    } else {
        count_label
    };
    row.child(
        focus_ring(
            icon_button("toggle-file-filter", glyph::SEARCH, 14., 24., 22., fg, t),
            focused == Some(FocusStop::FilterToggle),
            t,
        )
        .debug_selector(|| "toggle-file-filter".to_owned())
        .bg(rgb(bg))
        .tooltip(text_tooltip(jayjay_i18n::repo::filter_files()))
        .on_click(cx.listener(|view, _event: &ClickEvent, window, cx| {
            view.toggle_file_filter(window, cx);
        })),
    )
    .child(div().flex_1().min_w(px(8.)))
    .child(count_label)
}

pub(super) fn file_filter_bar(
    input: &LineInput,
    focus: &FocusHandle,
    cx: &mut Context<RepoWindow>,
    t: &Theme,
) -> AnyElement {
    div()
        .debug_selector(|| "file-filter-bar".to_owned())
        .flex()
        .flex_row()
        .items_center()
        .gap(px(4.))
        .px(px(14.))
        .pb(px(6.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(
            div()
                .id("file-filter-input")
                .debug_selector(|| "file-filter-input".to_owned())
                .flex()
                .items_center()
                .flex_1()
                .min_w_0()
                .h(px(t.scaled_control_height(26., 11.)))
                .px(px(8.))
                .rounded_md()
                .border_1()
                .border_color(rgb(t.border))
                .bg(rgb(t.detail_bg))
                .text_size(ui_font_size(11.))
                .cursor_text()
                .track_focus(focus)
                .focus(|style| style.border_color(rgb(t.selected_accent)))
                .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                    view.activate_file_filter(window, cx);
                }))
                .on_key_down(cx.listener(|view, event, window, cx| {
                    if view.handle_file_filter_key(event, window, cx) {
                        cx.stop_propagation();
                    }
                }))
                .relative()
                .child(LineInput::ime_layer(
                    cx.entity(),
                    Some(focus.clone()),
                    RepoWindow::file_filter_input,
                    RepoWindow::reconcile_file_selection,
                ))
                .child(line_input_content(
                    input,
                    jayjay_i18n::repo::filter_files(),
                    t,
                    Some("file-filter-caret"),
                )),
        )
        .child(
            icon_button(
                "file-filter-close",
                glyph::X_CIRCLE,
                13.,
                22.,
                22.,
                t.fg_faint,
                t,
            )
            .debug_selector(|| "file-filter-close".to_owned())
            .tooltip(text_tooltip(jayjay_i18n::diff::close_file_filter()))
            .on_click(cx.listener(|view, _: &ClickEvent, window, cx| {
                view.dismiss_file_filter(window, cx);
            })),
        )
        .into_any_element()
}
