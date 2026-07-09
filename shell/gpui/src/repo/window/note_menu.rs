//! Review-note gutter menu items + shell-side anchor construction. Anchor building never calls `Repo::review_note_anchor`/`jayjay_review::build_note_anchor` (the CLI add-note path): those hardcode `ignore_whitespace = false`, which can record a wrong-mode anchor that reconciles Stale in the very diff it was added from once the ignore-whitespace toggle is on.

use std::sync::Arc;

use gpui::Context;
use jayjay_core::DiffHunk;
use jayjay_core::diff::{
    DiffLine, DiffSide, anchor_side_and_number, build_diff_display_lines, change_group_for_anchor,
};
use jayjay_review::{NoteSide, NoteStatus};

use super::RepoWindow;
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::glyph;

/// Fully resolved at menu-build time (side/line/excerpt/context, change id, whitespace mode), so dispatch never re-derives anything from a possibly-stale selection or file switch made between build and click.
pub struct AddNoteRequest {
    pub(crate) change_id: String,
    pub(crate) path: String,
    pub(crate) identity: String,
    pub(crate) side: NoteSide,
    pub(crate) line: u32,
    pub(crate) anchor_excerpt: String,
    pub(crate) anchor_context: Vec<String>,
    pub(crate) ignore_whitespace: bool,
}

impl RepoWindow {
    /// `line_ix` is the exact right- or dot-clicked line, not necessarily the active selection's anchor.
    pub(super) fn note_menu_items(
        &self,
        hunk: &DiffHunk,
        line_ix: usize,
        cx: &Context<Self>,
    ) -> Vec<ContextMenuItem> {
        let Some(change_id) = self.review_notes_context(hunk, cx) else {
            return Vec::new();
        };
        let anchor = {
            let vm = self.vm.read(cx);
            let Some(fd) = vm.current_diff.as_ref() else {
                return Vec::new();
            };
            let display_lines = build_diff_display_lines(&fd.lines);
            let Some((side, line, anchor_excerpt)) = line_anchor(&display_lines, line_ix) else {
                return Vec::new();
            };
            let diff_side = match side {
                NoteSide::New => DiffSide::New,
                NoteSide::Old => DiffSide::Old,
            };
            let anchor_context =
                change_group_for_anchor(&display_lines, diff_side, line, &anchor_excerpt)
                    .map(|group| group.anchor_context)
                    .unwrap_or_default();
            (
                side,
                line,
                anchor_excerpt,
                anchor_context,
                vm.ignore_whitespace,
            )
        };
        let (side, line, anchor_excerpt, anchor_context, ignore_whitespace) = anchor;

        let notes_at_line: Vec<_> = self
            .notes_for_selected_hunk(cx)
            .into_iter()
            .filter(|s| s.note.side == side && s.note.line == line)
            .collect();

        if let Some(note) = notes_at_line
            .iter()
            .find(|s| s.status == NoteStatus::Current)
        {
            return vec![
                ContextMenuItem::new(
                    "Edit Review Note",
                    glyph::PENCIL_CIRCLE,
                    ContextAction::OpenEditReviewNote(note.note.id.clone().into()),
                ),
                ContextMenuItem::new(
                    "Resolve Review Note",
                    glyph::CHECK,
                    ContextAction::ResolveReviewNote(note.note.id.clone().into()),
                ),
                ContextMenuItem::new(
                    "Delete Review Note",
                    glyph::X_CIRCLE,
                    ContextAction::DeleteReviewNote(note.note.id.clone().into()),
                ),
            ];
        }

        let request = Arc::new(AddNoteRequest {
            change_id,
            path: hunk.path.clone(),
            identity: hunk.review_identity.clone(),
            side,
            line,
            anchor_excerpt,
            anchor_context,
            ignore_whitespace,
        });
        let mut items = vec![ContextMenuItem::new(
            "Add Review Note",
            glyph::PLUS_CIRCLE,
            ContextAction::OpenAddReviewNote(request),
        )];
        // A resolved note keeps only its dimmed dot; Delete here is the sole way to remove it (SwiftUI's popover offers the same).
        for resolved in notes_at_line
            .iter()
            .filter(|s| s.status == NoteStatus::Resolved)
        {
            items.push(ContextMenuItem::new(
                "Delete Review Note",
                glyph::X_CIRCLE,
                ContextAction::DeleteReviewNote(resolved.note.id.clone().into()),
            ));
        }
        items
    }
}

/// Side/line/excerpt for the line at `line_ix`, or `None` on a context/separator line — those can't anchor a note.
pub(super) fn line_anchor(
    display_lines: &[DiffLine],
    line_ix: usize,
) -> Option<(NoteSide, u32, String)> {
    let line = display_lines.get(line_ix)?;
    let (side, line_no) = anchor_side_and_number(line)?;
    let side = match side {
        DiffSide::New => NoteSide::New,
        DiffSide::Old => NoteSide::Old,
    };
    Some((side, line_no, line.text()))
}

/// Finds the display-line index matching an existing note's (side, line) anchor — used by the edit composer, which only has the note's recorded side/line, not the display index originally clicked.
pub(super) fn display_line_index_for(
    display_lines: &[DiffLine],
    side: NoteSide,
    line: u32,
) -> Option<usize> {
    let target = match side {
        NoteSide::New => DiffSide::New,
        NoteSide::Old => DiffSide::Old,
    };
    display_lines
        .iter()
        .position(|l| anchor_side_and_number(l) == Some((target, line)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jayjay_core::diff::{ConflictLineKind, DiffSpanStyle};

    fn line(style: DiffSpanStyle, old: Option<u32>, new: Option<u32>, text: &str) -> DiffLine {
        DiffLine {
            old_line_no: old,
            new_line_no: new,
            style,
            spans: vec![jayjay_core::diff::DiffSpan {
                text: text.to_owned(),
                style,
                token: jayjay_core::diff::syntax::SyntaxToken::Plain,
            }],
            conflict_kind: ConflictLineKind::None,
            no_eof_newline: false,
        }
    }

    #[test]
    fn line_anchor_uses_new_side_for_added_lines() {
        let lines = vec![line(DiffSpanStyle::Added, None, Some(3), "hello")];
        let (side, ln, excerpt) = line_anchor(&lines, 0).expect("anchor for added line");
        assert_eq!(side, NoteSide::New);
        assert_eq!(ln, 3);
        assert_eq!(excerpt, "hello");
    }

    #[test]
    fn line_anchor_uses_old_side_for_removed_lines() {
        let lines = vec![line(DiffSpanStyle::Removed, Some(4), None, "bye")];
        let (side, ln, _) = line_anchor(&lines, 0).expect("anchor for removed line");
        assert_eq!(side, NoteSide::Old);
        assert_eq!(ln, 4);
    }

    #[test]
    fn line_anchor_is_none_for_context_lines() {
        let lines = vec![line(DiffSpanStyle::Context, Some(1), Some(1), "same")];
        assert!(line_anchor(&lines, 0).is_none());
    }

    #[test]
    fn display_line_index_for_finds_matching_added_line() {
        let lines = vec![
            line(DiffSpanStyle::Context, Some(1), Some(1), "a"),
            line(DiffSpanStyle::Added, None, Some(2), "b"),
        ];
        assert_eq!(display_line_index_for(&lines, NoteSide::New, 2), Some(1));
        assert_eq!(display_line_index_for(&lines, NoteSide::Old, 2), None);
    }
}
