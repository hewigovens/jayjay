import CoreGraphics
import JayJayCore

enum BookmarkDragChangeAction: Equatable {
    case ignore
    case beginPress
    case beginDragging
    case updateDragging
}

enum BookmarkDragEndAction: Equatable {
    case ignore
    case cancel
    case confirmDrop
}

/// Drives dragging a bookmark chip onto a change. Unlike rebase, there is no
/// immutability gate (bookmarks may point at immutable changes) and a plain
/// click is a no-op rather than a selection. Faster arm/preview than rebase
/// since moves are cheap and reversible.
enum BookmarkDragGesturePolicy {
    static let armDuration = 0.4
    static let previewDelayMs = 300
    static let dragStartDistance: CGFloat = 2

    static func changeAction(
        bookmarkName: String,
        drag: BookmarkDragState?,
        location: CGPoint
    ) -> BookmarkDragChangeAction {
        guard let drag, drag.bookmarkName == bookmarkName else { return .beginPress }
        let movement = DAGRebaseGesturePolicy.movementDistance(from: drag.startLocation, to: location)
        switch drag.phase {
            case .pressing, .armed:
                // Start dragging the instant the pointer moves — no hold required.
                return movement >= dragStartDistance ? .beginDragging : .ignore
            case .dragging:
                return .updateDragging
        }
    }

    static func endAction(bookmarkName: String, drag: BookmarkDragState?) -> BookmarkDragEndAction {
        guard let drag, drag.bookmarkName == bookmarkName else { return .ignore }
        switch drag.phase {
            case .pressing, .armed:
                return .cancel
            case .dragging:
                return .confirmDrop
        }
    }

    static func dropRequest(
        drag: BookmarkDragState?,
        previewTargetCommitId: String?,
        hoveredCommitId: String?,
        entries: [GraphEntry]
    ) -> DAGBookmarkMoveRequest? {
        guard let drag,
              let targetCommitId = previewTargetCommitId ?? hoveredCommitId,
              targetCommitId != drag.sourceCommitId,
              let targetEntry = entries.first(where: { $0.change.commitId.id == targetCommitId })
        else {
            return nil
        }

        return DAGBookmarkMoveRequest(
            bookmarkName: drag.bookmarkName,
            destRev: DAGRebaseGesturePolicy.revision(for: targetEntry.change),
            destCommitId: targetEntry.change.commitId.id,
            destLabel: DAGRebaseGesturePolicy.displayLabel(for: targetEntry.change)
        )
    }
}
