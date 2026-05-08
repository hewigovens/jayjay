import CoreGraphics
import Foundation
import JayJayCore

typealias DAGRebasePlacement = RebasePlacement

extension RebasePlacement {
    var label: String {
        switch self {
            case .onto: "onto"
            case .after: "after"
            case .before: "before"
        }
    }

    var targetRole: String {
        switch self {
            case .onto: "New parent"
            case .after: "Insert after"
            case .before: "Insert before"
        }
    }

    var confirmationLabel: String {
        switch self {
            case .onto: "Rebase"
            case .after: "Insert After"
            case .before: "Insert Before"
        }
    }

    var releaseHint: String {
        switch self {
            case .onto: "Release to rebase onto"
            case .after: "Release to insert after"
            case .before: "Release to insert before"
        }
    }
}

struct DAGRebaseRequest: Identifiable {
    let id = UUID()
    let sourceRev: String
    let sourceChangeId: String
    let sourceCommitId: String
    let sourceLabel: String
    let destRev: String
    let destChangeId: String
    let destCommitId: String
    let destLabel: String
    let placement: DAGRebasePlacement
}

enum DAGRebasePhase {
    case pressing, armed, dragging
}

struct DAGRebaseDragState {
    let sourceCommitId: String
    let sourceChangeId: String
    let sourceRev: String
    let sourceLabel: String
    let startLocation: CGPoint
    var armedAt: Date?
    var phase: DAGRebasePhase
    var location: CGPoint
    var hoveredCommitId: String?
    var hoveredPlacement: DAGRebasePlacement?
}
