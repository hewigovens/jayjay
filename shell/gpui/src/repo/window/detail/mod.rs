mod description;
mod header;

use gpui::{AnyElement, Context, IntoElement, ParentElement, Styled, div, rgb};

use super::RepoWindow;
use crate::app::theme::Theme;
use crate::diff::{DiffViewState, FindState, MarkdownPreviewContent, SvgPreviewContent, diff_view};
use crate::ui::primitives::divider_h;

use header::{DetailHeaderState, detail_header};

pub(super) fn detail_pane(
    view: &RepoWindow,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
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
    let current_projection = vm.current_projection.clone();
    let current_svg_preview = vm.current_svg_preview.clone();
    let current_markdown_preview = vm.current_markdown_preview.clone();
    let compare = vm.compare.clone();
    let file_count = vm.files.as_ref().map(|files| files.len());
    let selected_hunk = vm.selected_hunk().cloned();
    let active_projection_preview = selected_hunk.as_ref().is_some_and(|hunk| {
        view.diff.rich_preview.as_ref().is_some_and(|selection| {
            selection.is_active(super::DiffRichPreviewKind::Projection, hunk.path.as_str())
        })
    });
    let active_svg_preview = selected_hunk.as_ref().is_some_and(|hunk| {
        view.diff.rich_preview.as_ref().is_some_and(|selection| {
            selection.is_active(super::DiffRichPreviewKind::Svg, hunk.path.as_str())
        })
    });
    let active_markdown_preview = selected_hunk.as_ref().is_some_and(|hunk| {
        view.diff.rich_preview.as_ref().is_some_and(|selection| {
            selection.is_active(super::DiffRichPreviewKind::Markdown, hunk.path.as_str())
        })
    });
    let html_external_url = selected_hunk
        .as_ref()
        .and_then(|hunk| crate::diff::projection::html_external_url(&vm.repo_path, hunk));
    let path_just_copied =
        view.feedback.recently_copied.as_ref().map(|s| s.as_ref()) == Some("path");

    let diff_state = DiffViewState {
        hunk: selected_hunk.as_ref(),
        file_diff: current_diff.as_deref(),
        loaded_projection: current_projection.as_ref(),
        active_projection_preview,
        active_markdown_preview,
        active_svg_preview,
        markdown_preview: current_markdown_preview
            .as_ref()
            .map(|preview| MarkdownPreviewContent {
                old: preview.old.as_ref(),
                new: preview.new.as_ref(),
            }),
        svg_preview: current_svg_preview
            .as_ref()
            .map(|preview| SvgPreviewContent {
                old: preview.old.as_deref(),
                new: preview.new.as_deref(),
            }),
        html_external_url: html_external_url.as_deref(),
        view_mode,
        detail_mode,
        annotate_lines,
        loading_annotate,
        path_just_copied,
        can_resolve_conflict: compare.is_none(),
        unified_bounds: view.diff.unified_bounds.clone(),
        sbs_old_bounds: view.diff.sbs_old_bounds.clone(),
        sbs_new_bounds: view.diff.sbs_new_bounds.clone(),
        wrap_cache: view.diff.wrap_cache.clone(),
    };
    let find = FindState {
        query: view.find.query.as_ref(),
        match_count: view.find.matches.len(),
        match_current: view.find.current,
    };

    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .child(detail_header(
            DetailHeaderState {
                change: &change,
                stats: stats.as_ref(),
                compare: compare.as_ref(),
                file_count,
                recently_copied: view.feedback.recently_copied.as_ref(),
                description_height,
            },
            t,
            cx,
        ))
        .child(divider_h(t))
        .child(diff_view(diff_state, find, view.scrolls.diff.clone(), cx))
        .into_any_element()
}
