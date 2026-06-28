import Foundation
import JayJayCore
import JayJayDiffUI
import SwiftUI

extension NoteSide {
    init(_ side: DiffSide) {
        self = side == .old ? .old : .new
    }
}

extension DiffSide {
    init(_ side: NoteSide) {
        self = side == .old ? .old : .new
    }
}

/// One line of the diff excerpt shown in the note editor, so the user sees what they are annotating while the sheet covers the diff.
struct ReviewNoteContextLine: Hashable {
    let text: String
    let style: DiffSpanStyle
    let isAnchor: Bool
}

struct ReviewNoteEditorState: Identifiable {
    let id: String
    let noteId: String?
    let anchor: DiffReviewNoteAnchor?
    // Captured at creation so the sheet can save without the (rebuildable) DiffSection that spawned it.
    let path: String
    let identity: String
    let body: String
    let context: [ReviewNoteContextLine]

    static func adding(
        anchor: DiffReviewNoteAnchor,
        path: String,
        identity: String,
        context: [ReviewNoteContextLine]
    ) -> ReviewNoteEditorState {
        ReviewNoteEditorState(
            id: "add-\(UUID().uuidString)",
            noteId: nil,
            anchor: anchor,
            path: path,
            identity: identity,
            body: "",
            context: context
        )
    }

    static func editing(note: ReviewStore.ReviewNote, context: [ReviewNoteContextLine]) -> ReviewNoteEditorState {
        ReviewNoteEditorState(
            id: note.id,
            noteId: note.id,
            anchor: nil,
            path: note.path,
            identity: note.identity,
            body: note.body,
            context: context
        )
    }
}

extension DiffSection: DiffGutterNoteActions {
    var reviewNotesEnabled: Bool {
        reviewModeEnabled
    }

    func activeNotes(anchor: DiffReviewNoteAnchor) -> [DiffReviewNoteSummary] {
        loadedReviewNoteSummaries().filter {
            !$0.isResolved && $0.side == anchor.side && $0.line == anchor.line
        }
    }

    func addNote(anchor: DiffReviewNoteAnchor) {
        noteEditor = .adding(
            anchor: anchor,
            path: hunk.path,
            identity: hunk.reviewIdentity,
            context: noteContextLines(aroundDisplayLine: Int(anchor.displayLine))
        )
    }

    func editNote(id: String) {
        guard let reviewStore, let reviewChangeId else { return }
        let note = reviewStore
            .listNotes(changeId: reviewChangeId, includeResolved: true)
            .first { $0.id == id }
        guard let note else { return }
        let context = displayLineForNote(note).map { noteContextLines(aroundDisplayLine: $0) } ?? []
        noteEditor = .editing(note: note, context: context)
    }

    /// Two lines of surrounding diff on each side of the anchor, mirroring what the gutter shows so the sheet stands alone.
    private func noteContextLines(aroundDisplayLine displayLine: Int) -> [ReviewNoteContextLine] {
        guard let lines = loadedDisplayLines else { return [] }
        let anchorIndex = displayLine - 1
        guard lines.indices.contains(anchorIndex) else { return [] }
        let range = max(0, anchorIndex - 2) ... min(lines.count - 1, anchorIndex + 2)
        return range.compactMap { index in
            let line = lines[index]
            guard line.style != .separator else { return nil }
            return ReviewNoteContextLine(
                text: line.spans.map(\.text).joined(),
                style: line.style,
                isAnchor: index == anchorIndex
            )
        }
    }

    private func displayLineForNote(_ note: ReviewStore.ReviewNote) -> Int? {
        guard let lines = loadedDisplayLines else { return nil }
        let index = lines.firstIndex { line in
            switch note.side {
                case .new: line.style == .added && line.newLineNo == note.line
                case .old: line.style == .removed && line.oldLineNo == note.line
            }
        }
        return index.map { $0 + 1 }
    }

    func deleteNote(id: String) {
        reviewStore?.deleteNote(id: id)
        onReviewStateChanged?()
    }

    func resolveNote(id: String) {
        reviewStore?.resolveNote(id: id)
        onReviewStateChanged?()
    }

    /// Filters the store's in-memory notes rather than a load-time snapshot, so a note resolved elsewhere (CLI, another window) never resurfaces here. Resolved notes stay in the list — they keep a dimmed gutter marker — and only stop expanding.
    func loadedReviewNoteSummaries() -> [DiffReviewNoteSummary] {
        guard reviewNotesEnabled, let reviewStore, let reviewChangeId else { return [] }
        return reviewStore.notes
            .filter { note in
                note.changeId == reviewChangeId
                    && note.path == hunk.path
                    && note.identity == hunk.reviewIdentity
            }
            .sorted { lhs, rhs in
                if lhs.side != rhs.side {
                    return lhs.side == .new
                }
                if lhs.line != rhs.line {
                    return lhs.line < rhs.line
                }
                if lhs.resolved != rhs.resolved {
                    return !lhs.resolved
                }
                return lhs.id < rhs.id
            }
            .map(reviewNoteSummary)
    }

    /// Re-fetch the store's note list from disk so queries see marks and notes written by other windows or the CLI.
    func refreshActiveNotes() {
        guard reviewNotesEnabled, let reviewStore, let reviewChangeId else { return }
        _ = reviewStore.listNotes(changeId: reviewChangeId)
    }

    private func reviewNoteSummary(_ note: ReviewStore.ReviewNote) -> DiffReviewNoteSummary {
        DiffReviewNoteSummary(
            id: note.id,
            body: note.body,
            side: DiffSide(note.side),
            line: note.line,
            excerpt: note.anchorExcerpt,
            isStale: staleNoteIds.contains(note.id),
            isResolved: note.resolved
        )
    }
}
