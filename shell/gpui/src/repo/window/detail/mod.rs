mod description;
mod header;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Styled, Window, div, px,
    rgb,
};

use super::RepoWindow;
use crate::app::theme::Theme;
use crate::diff::{DiffViewState, FindState, SvgPreviewContent, diff_view};
use crate::ui::{
    icons::{self, glyph},
    primitives::divider_h,
};

use header::{DetailHeaderState, detail_header};

pub(super) fn detail_pane(
    view: &RepoWindow,
    t: &Theme,
    window: &Window,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let description_height = view.layout.description_height;
    let can_edit_file = view.can_edit_selected_working_copy_file(cx);
    let vm = view.vm.read(cx);
    if let Some(count) = vm.selection_without_diff_count() {
        return multi_selection_no_diff(count, t);
    }
    let Some(change) = vm.selected_change().cloned() else {
        return div()
            .debug_selector(|| "detail-pane".to_owned())
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
    let selected_file_has_conflict = change.has_conflict
        && selected_hunk
            .as_ref()
            .is_some_and(|hunk| hunk.is_conflict_only_placeholder());
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
    let notes = view.notes_for_selected_hunk(cx);
    let stale_or_orphaned_notes = vm.stale_or_orphaned_notes();
    let bookmarks = vm.graph.bookmarks.clone();

    let diff_state = DiffViewState {
        hunk: selected_hunk.as_ref(),
        no_changes: file_count == Some(0),
        file_diff: current_diff.as_ref(),
        loaded_projection: current_projection.as_ref(),
        active_projection_preview,
        active_markdown_preview,
        active_svg_preview,
        markdown_preview: current_markdown_preview.as_deref(),
        markdown_scroll: view.diff.markdown_scroll.clone(),
        markdown_bounds: view.diff.markdown_bounds.clone(),
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
        can_edit_file,
        selected_file_has_conflict,
        supports_conflict_editor: selected_hunk
            .as_ref()
            .is_some_and(|hunk| hunk.supports_conflict_editor)
            && !change.is_immutable,
        unified_bounds: view.diff.unified_bounds.clone(),
        sbs_old_bounds: view.diff.sbs_old_bounds.clone(),
        sbs_new_bounds: view.diff.sbs_new_bounds.clone(),
        wrap_cache: view.diff.wrap_cache.clone(),
        notes: &notes,
        stale_or_orphaned_notes: &stale_or_orphaned_notes,
        context_expansion_error: view.context_expansion_error(),
    };
    let find = FindState {
        query: view.find.query.as_ref(),
        match_count: view.find.matches.len(),
        match_current: view.find.current,
    };

    div()
        .debug_selector(|| "detail-pane".to_owned())
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
                bookmarks: bookmarks.as_ref(),
            },
            t,
            cx,
        ))
        .child(divider_h(t))
        .child(diff_view(
            diff_state,
            find,
            view.scrolls.diff.clone(),
            window,
            cx,
        ))
        .into_any_element()
}

fn multi_selection_no_diff(count: usize, t: &Theme) -> AnyElement {
    let modifier = if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl"
    };

    div()
        .debug_selector(|| "detail-multi-selection-no-diff".to_owned())
        .flex()
        .flex_1()
        .size_full()
        .items_center()
        .justify_center()
        .px(px(24.))
        .bg(rgb(t.detail_bg))
        .child(
            div()
                .debug_selector(|| "detail-multi-selection-content".to_owned())
                .flex()
                .flex_col()
                .items_center()
                .w_full()
                .max_w(px(460.))
                .gap(px(10.))
                .text_align(gpui::TextAlign::Center)
                .child(icons::icon(glyph::ARROWS_LEFT_RIGHT, 28., t.compare_accent))
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(t.fg))
                        .child(format!("{count} Changes Selected")),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .line_height(px(18.))
                        .text_color(rgb(t.fg_dim))
                        .child("These changes don’t form a consecutive linear range."),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(3.))
                        .text_size(px(11.))
                        .line_height(px(16.))
                        .text_color(rgb(t.fg_faint))
                        .child(format!(
                            "Shift-click to compare two changes, or {modifier}-click a consecutive range for a combined diff."
                        ))
                        .child("Right-click any selected change for batch actions."),
                ),
        )
        .into_any_element()
}
