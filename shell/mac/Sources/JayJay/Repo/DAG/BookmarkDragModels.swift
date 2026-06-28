import CoreGraphics
import Foundation

/// Drag identity for the working copy. jj reserves "@", so no real bookmark can
/// use this name — it safely doubles as the working-copy ref's drag label.
let workingCopyDragLabel = "@"

/// A pending drag-to-move of a local bookmark onto `destCommitId`.
struct DAGBookmarkMoveRequest: Identifiable {
    let id = UUID()
    let bookmarkName: String
    let destRev: String
    let destCommitId: String
    let destLabel: String
}

/// In-flight state for dragging a bookmark chip. Reuses `DAGRebasePhase`.
struct BookmarkDragState {
    let bookmarkName: String
    let sourceCommitId: String
    let startLocation: CGPoint
    var armedAt: Date?
    var phase: DAGRebasePhase
    var location: CGPoint
    var hoveredCommitId: String?
}
