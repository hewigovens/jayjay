import CoreGraphics
import JayJayCore

enum DAGRebaseGestureChangeAction: Equatable {
    case ignore
    case beginPress
    case cancelPress
    case beginDragging
    case updateDragging
}

enum DAGRebaseGestureEndAction: Equatable {
    case ignore
    case select
    case cancel
    case confirmDrop
}

enum DAGRebaseGesturePolicy {
    static let armDuration = 0.75
    static let previewDelayMs = 500
    static let pressMoveTolerance: CGFloat = 8
    static let dragStartDistance: CGFloat = 2

    static func dropRequest(
        rebaseDrag: DAGRebaseDragState?,
        previewTargetCommitId: String?,
        hoveredCommitId: String?,
        entries: [GraphEntry]
    ) -> DAGRebaseRequest? {
        guard let rebaseDrag,
              let targetCommitId = previewTargetCommitId ?? hoveredCommitId,
              targetCommitId != rebaseDrag.sourceCommitId,
              let targetEntry = entries.first(where: { $0.change.commitId.id == targetCommitId })
        else {
            return nil
        }
        // Dropping a change onto its only parent has nothing to rebase.
        let sourceParents = entries.first(where: { $0.change.commitId.id == rebaseDrag.sourceCommitId })?.change.parents
        if sourceParents == [targetCommitId] {
            return nil
        }

        return DAGRebaseRequest(
            sourceRev: rebaseDrag.sourceRev,
            sourceChangeId: rebaseDrag.sourceChangeId,
            sourceCommitId: rebaseDrag.sourceCommitId,
            sourceLabel: rebaseDrag.sourceLabel,
            destRev: revision(for: targetEntry.change),
            destChangeId: targetEntry.change.changeId.id,
            destCommitId: targetEntry.change.commitId.id,
            destLabel: displayLabel(for: targetEntry.change)
        )
    }

    static func changeAction(
        entryIsImmutable: Bool,
        sourceCommitId: String,
        rebaseDrag: DAGRebaseDragState?,
        location: CGPoint
    ) -> DAGRebaseGestureChangeAction {
        guard !entryIsImmutable else { return .ignore }
        guard let rebaseDrag else { return .beginPress }
        guard rebaseDrag.sourceCommitId == sourceCommitId else { return .beginPress }

        let movement = movementDistance(from: rebaseDrag.startLocation, to: location)
        switch rebaseDrag.phase {
            case .pressing:
                return movement > pressMoveTolerance ? .cancelPress : .ignore
            case .armed:
                return movement >= dragStartDistance ? .beginDragging : .ignore
            case .dragging:
                return .updateDragging
        }
    }

    static func endAction(
        entryIsImmutable: Bool,
        sourceCommitId: String,
        rebaseDrag: DAGRebaseDragState?,
        startLocation: CGPoint,
        location: CGPoint
    ) -> DAGRebaseGestureEndAction {
        if entryIsImmutable {
            return movementDistance(from: startLocation, to: location) <= pressMoveTolerance ? .select : .ignore
        }

        guard let rebaseDrag, rebaseDrag.sourceCommitId == sourceCommitId else { return .ignore }
        switch rebaseDrag.phase {
            case .pressing:
                return .select
            case .armed:
                return .cancel
            case .dragging:
                return .confirmDrop
        }
    }

    static func movementDistance(from startLocation: CGPoint, to location: CGPoint) -> CGFloat {
        hypot(location.x - startLocation.x, location.y - startLocation.y)
    }

    static func normalizedTargetCommitId(sourceCommitId: String, hoveredCommitId: String?) -> String? {
        hoveredCommitId == sourceCommitId ? nil : hoveredCommitId
    }

    static func revision(for change: ChangeInfo) -> String {
        change.selectionRevision
    }

    static func displayLabel(for change: ChangeInfo) -> String {
        if let bookmark = change.bookmarks.first, !bookmark.isEmpty {
            return bookmark
        }
        let firstLine = change.description
            .components(separatedBy: "\n")
            .first?
            .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if !firstLine.isEmpty {
            return firstLine
        }
        if change.isWorkingCopy {
            return "@"
        }
        return String(change.changeId.id.prefix(8))
    }
}
