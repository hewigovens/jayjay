import JayJayCore
import SwiftUI

let laneWidth: CGFloat = 16
let nodeRadius: CGFloat = 4
let dagRowLeadingPadding: CGFloat = 4
let dagRowVerticalPadding: CGFloat = 8
let dagNodeCenterY: CGFloat = 12

/// Thin Swift wrapper over `jayjay_core::dag::DagLayout` (via uniffi).
struct DAGLayout {
    private let lanes: [String: UInt32]
    private let activeLanesPerRow: [UInt32]
    private let activeLaneIndicesPerRow: [[UInt32]]

    init(entries: [GraphEntry]) {
        let data = computeDagLayout(entries: entries)
        lanes = data.lanes
        activeLanesPerRow = data.activeLanesPerRow
        activeLaneIndicesPerRow = data.activeLaneIndicesPerRow
    }

    func lane(for commitId: String) -> Int {
        Int(lanes[commitId] ?? 0)
    }

    func maxLanes() -> Int {
        Int(activeLanesPerRow.max() ?? 1)
    }

    func activeLaneIndices(at row: Int) -> [Int] {
        guard activeLaneIndicesPerRow.indices.contains(row) else { return [] }
        return activeLaneIndicesPerRow[row].map(Int.init)
    }
}
