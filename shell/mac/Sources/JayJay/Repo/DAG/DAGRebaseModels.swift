import CoreGraphics
import Foundation
import JayJayCore

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

    var command: RepoRebaseCommand {
        RepoRebaseCommand(
            sourceRev: sourceRev,
            sourceLabel: sourceLabel,
            destRev: destRev,
            destLabel: destLabel,
            selectionRevs: selectionCommitIds
        )
    }
}

/// A rebase chosen from a menu, the palette, or a shortcut, which runs without the drop confirmation.
struct RepoRebaseCommand {
    let sourceRev: String
    let sourceLabel: String
    let destRev: String
    let destLabel: String
    var mode: RebaseMode = .source
    /// Every selected revision when a multi-selection moves together; empty for one change.
    var selectionRevs: [String] = []

    static func ontoTrunk(_ change: ChangeInfo, trunkName: String) -> RepoRebaseCommand {
        RepoRebaseCommand(
            sourceRev: change.selectionRevision,
            sourceLabel: DAGRebaseGesturePolicy.displayLabel(for: change),
            destRev: "trunk()",
            destLabel: trunkName,
            mode: .branch
        )
    }

    static func selection(_ revisions: [String], onto destRev: String, destLabel: String) -> RepoRebaseCommand {
        RepoRebaseCommand(
            sourceRev: revisions[0],
            sourceLabel: Localized.Repo.changeCount(count: revisions.count),
            destRev: destRev,
            destLabel: destLabel,
            selectionRevs: revisions
        )
    }

    func feedbackMessage(moved: Bool, hadConflicts: Bool) -> String {
        guard moved else { return Localized.Repo.alreadyOn(source: sourceLabel, destination: destLabel) }
        let base = Localized.Repo.rebasedOnto(source: sourceLabel, destination: destLabel)
        return hadConflicts ? Localized.Repo.rebasedWithConflicts(message: base) : base
    }
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
