mod description;
mod header;

use gpui::{AnyElement, Context, IntoElement, ParentElement, Styled, div, rgb};

use super::LogView;
use crate::app::theme::Theme;
use crate::diff::{DiffViewState, FindState, diff_view};
use crate::ui::primitives::divider_h;

use header::detail_header;

pub(super) fn detail_pane(view: &LogView, t: &Theme, cx: &mut Context<LogView>) -> AnyElement {
    let description_height = view.layout.description_height;
    let vm = view.vm.read(cx);
    let Some(change) = vm.selected_change().cloned() else {
        return div()
            .flex()
            .flex_1()
            .size_full()
            .items_center()
            .justify_center()
            .text_color(rgb(t.fg_dim))
            .child("Select a change")
            .into_any_element();
    };

    let stats = vm.change_stats.clone();
    let view_mode = vm.view_mode;
    let detail_mode = vm.detail_mode;
    let annotate_lines = vm.annotate_lines.clone();
    let loading_annotate = vm.loading.annotate;
    let current_diff = vm.current_diff.clone();
    let compare = vm.compare.clone();
    let file_count = vm.files.as_ref().map(|files| files.len());
    let selected_hunk = vm.selected_hunk().cloned();
    let path_just_copied =
        view.feedback.recently_copied.as_ref().map(|s| s.as_ref()) == Some("path");

    let diff_state = DiffViewState {
        hunk: selected_hunk.as_ref(),
        file_diff: current_diff.as_deref(),
        view_mode,
        detail_mode,
        annotate_lines,
        loading_annotate,
        path_just_copied,
        unified_bounds: view.diff.unified_bounds.clone(),
        sbs_old_bounds: view.diff.sbs_old_bounds.clone(),
        sbs_new_bounds: view.diff.sbs_new_bounds.clone(),
    };
    let find = FindState {
        query: view.find.query.as_deref(),
        match_count: view.find.matches.len(),
        match_current: view.find.current,
        caret_visible: view.find.caret_visible,
    };

    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .child(detail_header(
            &change,
            stats.as_ref(),
            compare.as_ref(),
            file_count,
            view.feedback.recently_copied.as_ref(),
            description_height,
            t,
            cx,
        ))
        .child(divider_h(t))
        .child(diff_view(diff_state, find, view.scrolls.diff.clone(), cx))
        .into_any_element()
}
