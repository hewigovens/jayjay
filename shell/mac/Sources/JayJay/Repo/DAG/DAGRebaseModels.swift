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
    /// Every selected commit when the dragged one was part of a multi-selection; empty for a single-change drag.
    var selectionCommitIds: [String] = []
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
    var phase: DAGRebasePhase
    var location: CGPoint
    var hoveredCommitId: String?
    /// Rows a drop must refuse, computed once when the drag starts so hovering never crosses into Rust.
    var descendantCommitIds: Set<String> = []
    var selectionCommitIds: [String] = []
    /// Rows the whole selection may land on, from the same capabilities as the batch context menu.
    var selectionTargets: Set<String> = []
    var targetRefusal: String?
}
