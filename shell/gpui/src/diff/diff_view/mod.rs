mod body;
mod find_bar;
mod header;
mod placeholders;

use gpui::{
    AnyElement, Context, IntoElement, ParentElement, Styled, UniformListScrollHandle, div, rgb,
};
use jayjay_core::DiffHunk;
use jayjay_core::diff::FileDiff;

use crate::app::theme::theme;
use crate::log::LogView;
use crate::ui::icons::glyph;

use body::{side_by_side_body, unified_body};
use find_bar::render_find_bar;
use header::{file_header, hunk_is_git_lfs, hunk_is_submodule};
use placeholders::{placeholder, placeholder_card, placeholder_inner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffViewMode {
    Unified,
    SideBySide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailMode {
    Diff,
    Annotate,
}

/// Pure-data inputs for the diff/annotate body.
pub struct DiffViewState<'a> {
    pub hunk: Option<&'a DiffHunk>,
    pub file_diff: Option<&'a FileDiff>,
    pub view_mode: DiffViewMode,
    pub detail_mode: DetailMode,
    pub annotate_lines: Option<std::sync::Arc<Vec<jayjay_core::AnnotationLine>>>,
    pub loading_annotate: bool,
    pub path_just_copied: bool,
    pub unified_bounds: crate::log::PanelBoundsSlot,
    pub sbs_old_bounds: crate::log::PanelBoundsSlot,
    pub sbs_new_bounds: crate::log::PanelBoundsSlot,
}

/// Find-in-diff state.
pub struct FindState<'a> {
    pub query: Option<&'a str>,
    pub match_count: usize,
    pub match_current: usize,
}

pub fn diff_view(
    state: DiffViewState<'_>,
    find: FindState<'_>,
    scroll: UniformListScrollHandle,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let t = theme(cx).clone();

    let Some(hunk) = state.hunk else {
        return placeholder("Select a file", &t).into_any_element();
    };

    let is_annotating = matches!(state.detail_mode, DetailMode::Annotate);
    let header = file_header(
        hunk,
        state.view_mode,
        is_annotating,
        state.path_just_copied,
        &t,
        cx,
    );

    let query = find.query.filter(|q| !q.is_empty()).map(|q| q.to_owned());

    let body: AnyElement = if is_annotating {
        if state.loading_annotate {
            placeholder_inner("Loading annotations…", &t).into_any_element()
        } else if let Some(lines) = state.annotate_lines {
            if lines.is_empty() {
                placeholder_inner("No annotations available", &t).into_any_element()
            } else {
                crate::diff::annotate_view::annotate_body(lines, t.clone(), scroll.clone())
            }
        } else {
            placeholder_inner("Annotations unavailable", &t).into_any_element()
        }
    } else if crate::diff::image_diff::hunk_is_image(hunk) {
        crate::diff::image_diff::image_diff_view(hunk, &t)
    } else if hunk_is_submodule(hunk) {
        placeholder_card(
            glyph::PACKAGE,
            "Git submodule",
            "This submodule has working-copy changes, but JayJay does not render an inline text diff for submodule contents. Open or commit the submodule in its own repository.",
            &t,
        )
        .into_any_element()
    } else if hunk_is_git_lfs(hunk) {
        placeholder_card(
            glyph::HARD_DRIVE,
            "Git LFS-backed file",
            "This file is tracked through Git LFS. JayJay does not render an inline text diff between the committed pointer and the local binary object.",
            &t,
        )
        .into_any_element()
    } else {
        match (state.file_diff, state.view_mode) {
            (None, _) => placeholder_inner("Loading diff…", &t).into_any_element(),
            (Some(fd), _) if fd.lines.is_empty() => {
                placeholder_inner("No textual diff (binary, identical, or empty)", &t)
                    .into_any_element()
            }
            (Some(fd), DiffViewMode::Unified) => unified_body(
                fd,
                t.clone(),
                query.clone(),
                scroll.clone(),
                state.unified_bounds.clone(),
                cx,
            ),
            (Some(fd), DiffViewMode::SideBySide) => side_by_side_body(
                fd,
                t.clone(),
                query.clone(),
                scroll.clone(),
                state.sbs_old_bounds.clone(),
                state.sbs_new_bounds.clone(),
                cx,
            ),
        }
    };

    let find_bar = find
        .query
        .map(|q| render_find_bar(q, find.match_count, find.match_current, &t));

    let mut root = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .bg(rgb(t.detail_bg))
        .child(header);
    if let Some(bar) = find_bar {
        root = root.child(bar);
    }
    root.child(div().flex().flex_col().flex_1().min_h_0().child(body))
        .into_any_element()
}
