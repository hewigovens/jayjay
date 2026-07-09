//! Right-click menu for unified-diff gutter rows.

use std::sync::Arc;

use gpui::{Context, Pixels, Point};
use jayjay_core::diff::{ConflictLineKind, build_diff_display_lines};
use jayjay_core::placeholder::is_editable_text;
use jayjay_core::{CoreError, DiffEditFileSelection, DiffHunk, HunkType};

use super::RepoWindow;
use crate::diff::{
    GutterLineSelection, display_range_to_diff_edit_range, selection_covers_whole_change_group,
};
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::glyph;

/// Fully resolved when the menu was built (line-range mapping run, content retained), so dispatch never re-derives anything or risks racing a selection/file change made after the click.
pub struct AbandonSelectedLinesRequest {
    rev: String,
    selection: DiffEditFileSelection,
    ignore_whitespace: bool,
    /// Completion only clears `diff.gutter_selection` if it still matches this, so a selection started elsewhere during the async gap between dispatch and completion survives.
    source_selection: GutterLineSelection,
}

impl RepoWindow {
    pub fn open_gutter_context_menu(
        &mut self,
        path: String,
        line_ix: usize,
        anchor: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let covered = self
            .diff
            .gutter_selection
            .as_ref()
            .is_some_and(|sel| sel.covers(&path, line_ix));
        if !covered {
            self.start_gutter_selection(path, line_ix, cx);
        }
        let hunk = self.vm.read(cx).selected_hunk().cloned();
        let items = hunk
            .map(|hunk| self.build_diff_gutter_menu(&hunk, line_ix, cx))
            .unwrap_or_default();
        self.open_context_menu(anchor, items, cx);
    }

    /// `line_ix` is the exact clicked line (gutter row or note dot), not necessarily the active selection's anchor.
    pub fn build_diff_gutter_menu(
        &self,
        hunk: &DiffHunk,
        line_ix: usize,
        cx: &Context<Self>,
    ) -> Vec<ContextMenuItem> {
        let mut items = self.note_menu_items(hunk, line_ix, cx);
        if let Some(item) = self.abandon_selected_lines_menu_item(hunk, cx) {
            items.push(item);
        }
        items
    }

    /// jj materializes an unresolved conflict's content with literal marker text, which passes `is_editable_text`, so conflicts need their own explicit rejection check here.
    fn abandon_selected_lines_menu_item(
        &self,
        hunk: &DiffHunk,
        cx: &Context<Self>,
    ) -> Option<ContextMenuItem> {
        let selection = self.diff.gutter_selection.as_ref()?;
        if selection.path != hunk.path {
            return None;
        }
        if !self.review_file_context(cx).0 {
            return None;
        }
        if hunk.projection.is_some() || hunk.hunk_type == HunkType::Renamed {
            return None;
        }

        let vm = self.vm.read(cx);
        let old_content = vm.current_diff_old_content.as_deref()?;
        let new_content = vm.current_diff_new_content.as_deref()?;
        if !is_editable_text(old_content) || !is_editable_text(new_content) {
            return None;
        }
        let raw_lines = &vm.current_diff.as_ref()?.lines;
        if raw_lines
            .iter()
            .any(|line| line.conflict_kind != ConflictLineKind::None)
        {
            return None;
        }
        // Indexes the display basis (post conflict-block collapse) used by `WrappedDiffLine::line_ix`/`GutterLineSelection`, never raw `FileDiff.lines`.
        let display_lines = build_diff_display_lines(raw_lines);
        let rev = vm.selected_revision()?;
        let line_range = selection.line_range();

        let line_ranges = display_range_to_diff_edit_range(
            &hunk.path,
            &display_lines,
            old_content,
            new_content,
            vm.ignore_whitespace,
            line_range.clone(),
        );
        if line_ranges.is_empty() {
            return None;
        }
        let is_whole_group = selection_covers_whole_change_group(&display_lines, line_range);

        let request = Arc::new(AbandonSelectedLinesRequest {
            rev,
            selection: DiffEditFileSelection {
                path: hunk.path.clone(),
                old_path: None,
                // Absent side must be None (not Some("")), mirroring SwiftUI: an Added file has no old side, a Removed file has no new side, so the staleness guard's materialized-vs-selection comparison matches for deleted/added files.
                old_content: (hunk.hunk_type != HunkType::Added).then(|| old_content.to_owned()),
                new_content: (hunk.hunk_type != HunkType::Removed).then(|| new_content.to_owned()),
                hunk_type: hunk.hunk_type,
                line_ranges,
            },
            ignore_whitespace: vm.ignore_whitespace,
            source_selection: selection.clone(),
        });
        let title = if is_whole_group {
            "Abandon Change Group"
        } else {
            "Abandon Selected Lines"
        };
        Some(ContextMenuItem::new(
            title,
            glyph::X_CIRCLE,
            ContextAction::AbandonSelectedLines(request),
        ))
    }

    /// Success is silent (no toast) — the diff refreshing with the same file re-selected is confirmation enough; `repo_write_task` still surfaces failures via `vm.present_error`.
    pub(super) fn abandon_selected_diff_lines(
        &mut self,
        request: Arc<AbandonSelectedLinesRequest>,
        cx: &mut Context<Self>,
    ) {
        let rev = request.rev.clone();
        let ignore_whitespace = request.ignore_whitespace;
        let selection = request.selection.clone();
        let source_selection = request.source_selection.clone();
        let task = self.vm.update(cx, |vm, cx| {
            vm.abandon_selected_diff_lines(rev, selection, ignore_whitespace, cx)
        });
        cx.spawn(async move |this, cx| {
            match task.await {
                Ok(()) => {
                    let _ = this.update(cx, move |view, _cx| {
                        if view.diff.gutter_selection.as_ref() == Some(&source_selection) {
                            view.diff.gutter_selection = None;
                        }
                    });
                }
                // This refresh also clears `present_error`'s banner, but the refreshed diff itself shows why the action didn't apply.
                Err(CoreError::DiffSelectionStale { .. }) => {
                    let _ = this.update(cx, |view, cx| {
                        view.vm.update(cx, |vm, cx| vm.refresh(false, cx));
                    });
                }
                Err(_) => {}
            }
        })
        .detach();
    }
}
