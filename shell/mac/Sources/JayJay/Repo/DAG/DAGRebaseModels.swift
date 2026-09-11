import CoreGraphics
import Foundation

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
}

enum DAGRebasePhase {
    case pressing, armed, dragging
}

struct DAGRebaseDragState {
    let sourceCommitId: String
    let sourceChangeId: String
    let sourceRev: String
    let sourceLabel: String
    let sourceParents: [String]
    let startLocation: CGPoint
    var armedAt: Date?
    var phase: DAGRebasePhase
    var location: CGPoint
    var hoveredCommitId: String?
    /// Rows a drop must refuse, computed once when the drag starts so hovering never crosses into Rust.
    var descendantCommitIds: Set<String> = []
    var targetRefusal: String?
}
