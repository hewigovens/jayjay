//! One file of a stack of diffs that scrolls as one list: its header and its rows at their full height.

use std::sync::Arc;

use gpui::{AnyElement, App, Context, IntoElement, ParentElement, Styled, div, px};
use jayjay_core::diff::FileDiff;
use jayjay_core::{DiffHunk, FileDiffStats};
use jayjay_review::{NoteStatus, ReviewFileRollup, ReviewNoteStatus};

use super::header::{FileHeaderState, ProjectionHeaderState, file_header};
use super::row_list::RowPlacement;
use super::sbs_body::{SideBySideBodyState, side_by_side_body};
use super::sbs_note_banner::sbs_note_banner;
use super::state::{DiffViewMode, ReviewDisplayState};
use super::unified_body::{UnifiedBodyState, unified_body};
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::diff::wrap::wrap_cols_from_bounds;
use crate::repo::window::{DiffWrapCacheSlot, PanelBoundsSlot, RepoWindow};

const STACKED_NOTE_BANNER_HEIGHT: f32 = 28.;

pub(crate) struct StackedRows<'a> {
    pub(crate) file_diff: &'a Arc<FileDiff>,
    pub(crate) view_mode: DiffViewMode,
    pub(crate) notes: &'a [ReviewNoteStatus],
    pub(crate) wrap_cache: &'a DiffWrapCacheSlot,
    pub(crate) unified_bounds: &'a PanelBoundsSlot,
    pub(crate) sbs_old_bounds: &'a PanelBoundsSlot,
    pub(crate) sbs_new_bounds: &'a PanelBoundsSlot,
}

impl StackedRows<'_> {
    fn shows_note_banner(&self) -> bool {
        self.view_mode == DiffViewMode::SideBySide
            && self
                .notes
                .iter()
                .any(|note| note.status == NoteStatus::Current && !note.note.resolved)
    }

    pub(crate) fn banner_height(&self) -> f32 {
        if self.shows_note_banner() {
            STACKED_NOTE_BANNER_HEIGHT
        } else {
            0.
        }
    }

    pub(crate) fn rows_height(&self, theme: &Theme, cx: &App) -> f32 {
        let advance = fonts::mono_advance(cx, px(theme.code_font_size()));
        let mut cache = self.wrap_cache.borrow_mut();
        let label_cols = cache.moved_label_cols(self.file_diff, cx);
        let rows = match self.view_mode {
            DiffViewMode::Unified => {
                let cols = wrap_cols_from_bounds(self.unified_bounds.get(), advance);
                cache
                    .rows(self.file_diff, cols, self.notes, label_cols)
                    .rows
                    .len()
            }
            DiffViewMode::SideBySide => {
                let old_cols = wrap_cols_from_bounds(self.sbs_old_bounds.get(), advance);
                let new_cols = wrap_cols_from_bounds(self.sbs_new_bounds.get(), advance);
                cache
                    .side_by_side(self.file_diff, old_cols, new_cols, label_cols)
                    .len()
            }
        };
        rows as f32 * theme.code_line_height()
    }

    pub(crate) fn render(
        &self,
        active: bool,
        query: Option<String>,
        shows_review: bool,
        review: Option<Arc<ReviewDisplayState>>,
        theme: &Theme,
        cx: &mut Context<RepoWindow>,
    ) -> AnyElement {
        let placement = RowPlacement::Stacked { active };
        match self.view_mode {
            DiffViewMode::Unified => unified_body(
                UnifiedBodyState {
                    file_diff: self.file_diff,
                    theme: theme.clone(),
                    query,
                    placement,
                    bounds: self.unified_bounds.clone(),
                    wrap_cache: self.wrap_cache,
                    notes: self.notes,
                    shows_review,
                    review,
                },
                cx,
            ),
            DiffViewMode::SideBySide => {
                let body = side_by_side_body(
                    SideBySideBodyState {
                        file_diff: self.file_diff,
                        theme: theme.clone(),
                        query,
                        placement,
                        old_bounds: self.sbs_old_bounds.clone(),
                        new_bounds: self.sbs_new_bounds.clone(),
                        wrap_cache: self.wrap_cache,
                        shows_review,
                        review,
                    },
                    cx,
                );
                let banner = sbs_note_banner(self.notes, theme, cx).map(|banner| {
                    div()
                        .h(px(STACKED_NOTE_BANNER_HEIGHT))
                        .overflow_hidden()
                        .child(banner)
                });
                div()
                    .flex()
                    .flex_col()
                    .children(banner)
                    .child(body)
                    .into_any_element()
            }
        }
    }
}

pub(crate) struct StackedFileHeader<'a> {
    pub(crate) hunk: &'a DiffHunk,
    pub(crate) line_stats: Option<&'a FileDiffStats>,
    pub(crate) just_copied: bool,
    pub(crate) can_edit_file: bool,
    pub(crate) file_review: Option<ReviewFileRollup>,
    pub(crate) collapsed: bool,
    pub(crate) detail_width: f32,
}

/// Rich previews and annotations show a file alone, so a stacked header leaves their toggles out.
pub(crate) fn stacked_file_header(
    state: StackedFileHeader<'_>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    file_header(
        FileHeaderState {
            hunk: state.hunk,
            line_stats: state.line_stats,
            projection: ProjectionHeaderState {
                projection: None,
                view: None,
            },
            active_markdown_preview: false,
            can_render_markdown_preview: false,
            active_svg_preview: false,
            can_render_svg_preview: false,
            is_annotating: false,
            just_copied: state.just_copied,
            html_external_url: None,
            can_edit_file: state.can_edit_file,
            file_review: state.file_review,
            collapsed: Some(state.collapsed),
            detail_width: state.detail_width,
            focused: None,
        },
        t,
        cx,
    )
}
