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
        hoveredPlacement: DAGRebasePlacement?,
        entries: [GraphEntry]
    ) -> DAGRebaseRequest? {
        guard let rebaseDrag,
              let targetCommitId = previewTargetCommitId ?? hoveredCommitId,
              targetCommitId != rebaseDrag.sourceCommitId,
              let targetEntry = entries.first(where: { $0.change.commitId == targetCommitId })
        else {
            return nil
        }

        let placement = hoveredPlacement ?? .onto
        guard !(targetEntry.change.isImmutable && placement == .before) else { return nil }

        return DAGRebaseRequest(
            sourceRev: rebaseDrag.sourceRev,
            sourceChangeId: rebaseDrag.sourceChangeId,
            sourceCommitId: rebaseDrag.sourceCommitId,
            sourceLabel: rebaseDrag.sourceLabel,
            destRev: revision(for: targetEntry.change),
            destChangeId: targetEntry.change.changeId,
            destCommitId: targetEntry.change.commitId,
            destLabel: displayLabel(for: targetEntry.change),
            placement: placement
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

    static func placement(location: CGPoint, rowFrame: CGRect?) -> DAGRebasePlacement? {
        guard let rowFrame, rowFrame.contains(location), rowFrame.height > 0 else { return nil }
        let relativeY = (location.y - rowFrame.minY) / rowFrame.height
        if relativeY < 0.3 { return .before }
        if relativeY > 0.7 { return .after }
        return .onto
    }

    static func validPlacement(
        location: CGPoint,
        rowFrame: CGRect?,
        targetIsImmutable: Bool
    ) -> DAGRebasePlacement? {
        let placement = placement(location: location, rowFrame: rowFrame)
        return targetIsImmutable && placement == .before ? nil : placement
    }

    static func revision(for change: ChangeInfo) -> String {
        change.isDivergent ? change.commitId : change.changeId
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
        return String(change.changeId.prefix(8))
    }
}
