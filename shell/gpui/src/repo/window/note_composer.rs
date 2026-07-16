//! Review-note composer: an overlay extension of `TextModalState`, not a `uniform_list` row — a row would clip and lose focus mid-edit once note rows scroll out of the fixed-height list.

use gpui::{App, AppContext, Context};
use jayjay_core::diff::{DiffSpanStyle, build_diff_display_lines};
use jayjay_review::{NoteAnchor, NoteEntry, NoteSide};

use super::note_menu::{AddNoteRequest, display_line_index_for};
use super::{RepoWindow, TextModalAction, TextModalContext, TextModalState};
use crate::ui::text_area::TextArea;

#[derive(Clone)]
pub(crate) struct NoteContextLine {
    pub(crate) text: String,
    pub(crate) style: DiffSpanStyle,
    pub(crate) is_anchor: bool,
}

/// `anchor` is `Some` only when adding a new note; editing only changes the body, never the anchor.
#[derive(Clone)]
pub(crate) struct NoteComposerTarget {
    pub(crate) note_id: Option<String>,
    pub(crate) path: String,
    pub(crate) identity: String,
    pub(crate) change_id: String,
    pub(crate) anchor: Option<NoteAnchorDraft>,
}

#[derive(Clone)]
pub(crate) struct NoteAnchorDraft {
    pub(crate) side: NoteSide,
    pub(crate) line: u32,
    pub(crate) anchor_excerpt: String,
    pub(crate) anchor_context: Vec<String>,
    pub(crate) ignore_whitespace: bool,
}

impl RepoWindow {
    pub(super) fn open_add_note_composer(
        &mut self,
        request: std::sync::Arc<AddNoteRequest>,
        cx: &mut Context<Self>,
    ) {
        let context = self.note_context_lines_at(request.line, request.side, cx);
        let target = NoteComposerTarget {
            note_id: None,
            path: request.path.clone(),
            identity: request.identity.clone(),
            change_id: request.change_id.clone(),
            anchor: Some(NoteAnchorDraft {
                side: request.side,
                line: request.line,
                anchor_excerpt: request.anchor_excerpt.clone(),
                anchor_context: request.anchor_context.clone(),
                ignore_whitespace: request.ignore_whitespace,
            }),
        };
        self.open_note_composer(
            "Add Review Note",
            "Add Note",
            target,
            String::new(),
            context,
            cx,
        );
    }

    pub(super) fn open_edit_note_composer(&mut self, note_id: String, cx: &mut Context<Self>) {
        let Some(note) = self.find_review_note(&note_id, cx) else {
            return;
        };
        let context = self.note_context_lines_at(note.line, note.side, cx);
        let target = NoteComposerTarget {
            note_id: Some(note.id.clone()),
            path: note.path.clone(),
            identity: note.identity.clone(),
            change_id: note.change_id.clone(),
            anchor: None,
        };
        self.open_note_composer(
            "Edit Review Note",
            "Save",
            target,
            note.body.clone(),
            context,
            cx,
        );
    }

    /// `pub` (not `pub(super)`, unlike its Delete sibling): `diff::diff_view::note_banner` calls this directly too.
    pub fn resolve_review_note(&mut self, note_id: String, cx: &mut Context<Self>) {
        super::review::mutate(&self.review_store, |store| {
            store.resolve_note(&note_id);
        });
        self.refresh_review_notes(cx);
    }

    pub(super) fn delete_review_note(&mut self, note_id: String, cx: &mut Context<Self>) {
        super::review::mutate(&self.review_store, |store| {
            store.delete_note(&note_id);
        });
        self.refresh_review_notes(cx);
    }

    pub(super) fn save_review_note(
        &mut self,
        target: NoteComposerTarget,
        body: String,
        cx: &mut Context<Self>,
    ) {
        let trimmed = body.trim();
        if trimmed.is_empty() {
            return;
        }
        match (target.note_id, target.anchor) {
            (Some(note_id), _) => {
                super::review::mutate(&self.review_store, |store| {
                    store.update_note(&note_id, trimmed);
                });
            }
            (None, Some(anchor)) => {
                let note_anchor = NoteAnchor {
                    change_id: target.change_id,
                    path: target.path,
                    identity: target.identity,
                    side: anchor.side,
                    line: anchor.line,
                    anchor_excerpt: anchor.anchor_excerpt,
                    anchor_context: anchor.anchor_context,
                    ignore_whitespace: anchor.ignore_whitespace,
                };
                super::review::mutate(&self.review_store, |store| {
                    store.add_note(note_anchor, trimmed);
                });
            }
            (None, None) => {
                // Unreachable in practice: the composer always carries either a note id (editing) or a fresh anchor (adding), never neither.
            }
        }
        self.refresh_review_notes(cx);
    }

    fn open_note_composer(
        &mut self,
        title: &'static str,
        primary_label: &'static str,
        target: NoteComposerTarget,
        body: String,
        context: Vec<NoteContextLine>,
        cx: &mut Context<Self>,
    ) {
        let subtitle = target.path.clone();
        let input = cx.new(|cx| TextArea::new(body, "Note", true, 130., cx));
        let context_text = context
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let emphasized_line = context.iter().position(|line| line.is_anchor);
        let context_input = cx.new(|cx| {
            TextArea::selectable_code_block(context_text, context.len(), emphasized_line, cx)
        });
        self.text_modal = Some(TextModalState {
            title: title.into(),
            subtitle: subtitle.into(),
            primary_label: primary_label.into(),
            action: TextModalAction::ReviewNote(target),
            input,
            focus_pending: true,
            context: Some(TextModalContext {
                lines: context,
                input: context_input,
            }),
            checkbox: None,
            file_list: None,
        });
        cx.notify();
    }

    /// Looks up by id in `vm.review_notes` (already includes resolved notes) rather than re-reading the store.
    fn find_review_note(&self, note_id: &str, cx: &App) -> Option<NoteEntry> {
        self.vm
            .read(cx)
            .review_notes
            .iter()
            .find(|s| s.note.id == note_id)
            .map(|s| s.note.clone())
    }

    fn note_context_lines_at(&self, line: u32, side: NoteSide, cx: &App) -> Vec<NoteContextLine> {
        let vm = self.vm.read(cx);
        let Some(fd) = vm.current_diff.as_ref() else {
            return Vec::new();
        };
        let display_lines = build_diff_display_lines(&fd.lines);
        let Some(anchor_ix) = display_line_index_for(&display_lines, side, line) else {
            return Vec::new();
        };
        let lo = anchor_ix.saturating_sub(2);
        let hi = (anchor_ix + 2).min(display_lines.len().saturating_sub(1));
        (lo..=hi)
            .filter_map(|ix| {
                let l = display_lines.get(ix)?;
                (l.style != DiffSpanStyle::Separator).then(|| NoteContextLine {
                    text: l.text(),
                    style: l.style,
                    is_anchor: ix == anchor_ix,
                })
            })
            .collect()
    }
}
