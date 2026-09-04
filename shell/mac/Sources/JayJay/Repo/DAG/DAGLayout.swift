import JayJayCore
import SwiftUI

let laneWidth: CGFloat = 16
let nodeRadius: CGFloat = 4
let dagRowLeadingPadding: CGFloat = 4
let dagRowVerticalPadding: CGFloat = 8
let dagNodeCenterY: CGFloat = 12
let dagCompactVisibleLanes = 4
let dagOverflowStroke = StrokeStyle(lineWidth: 1, dash: [10, 4, 10, 12])
let dagIndirectEdgeStroke = StrokeStyle(lineWidth: 1, dash: [3, 3])
let dagMissingEdgeStroke = StrokeStyle(lineWidth: 1, lineCap: .round, dash: [2, 2])
let dagSolidStroke = StrokeStyle(lineWidth: 1)

/// Thin Swift wrapper over `jayjay_core::dag::DagLayout` (via uniffi).
struct DAGLayout {
    private let lanes: [String: UInt32]
    private let maxLaneCount: Int
    private let activeLaneIndicesPerRow: [[UInt32]]
    private let passThroughLaneIndicesPerRow: [[UInt32]]
    private let missingAncestryRows: [Bool]
    private let overflowRows: [Bool]
    private let displayLaneCountValue: UInt32

    init(entries: [GraphEntry]) {
        let data = computeDagLayout(entries: entries)
        lanes = data.lanes
        maxLaneCount = Int(data.activeLanesPerRow.max() ?? 1)
        activeLaneIndicesPerRow = data.activeLaneIndicesPerRow
        passThroughLaneIndicesPerRow = data.passThroughLaneIndicesPerRow
        missingAncestryRows = data.missingAncestryRows
        overflowRows = data.overflowRows
        displayLaneCountValue = data.displayLaneCount
    }

    func lane(for commitId: String) -> Int {
        Int(lanes[commitId] ?? 0)
    }

    func maxLanes() -> Int {
        maxLaneCount
    }

    func displayLaneCount() -> Int {
        max(1, Int(displayLaneCountValue))
    }

    var graphWidth: CGFloat {
        CGFloat(displayLaneCount()) * laneWidth + 8
    }

    func displayLane(for lane: Int) -> Int {
        guard maxLanes() > dagCompactVisibleLanes else { return lane }
        return min(lane, dagCompactVisibleLanes - 1)
    }

    func activeLaneIndices(at row: Int) -> [Int] {
        guard activeLaneIndicesPerRow.indices.contains(row) else { return [] }
        return activeLaneIndicesPerRow[row].map(Int.init)
    }

    func passThroughLaneIndices(at row: Int) -> [Int] {
        guard passThroughLaneIndicesPerRow.indices.contains(row) else { return [] }
        return passThroughLaneIndicesPerRow[row].map(Int.init)
    }

    func hasMissingAncestry(at row: Int) -> Bool {
        guard missingAncestryRows.indices.contains(row) else { return false }
        return missingAncestryRows[row]
    }

    func hasLaneOverflow(at row: Int) -> Bool {
        guard overflowRows.indices.contains(row) else { return false }
        return overflowRows[row]
    }

    func xPosition(forDisplayLane displayLane: Int) -> CGFloat {
        CGFloat(displayLane) * laneWidth + laneWidth / 2 + 4
    }

    func xPosition(for lane: Int, at row: Int) -> CGFloat {
        xPosition(forDisplayLane: displayLane(for: lane))
    }
}
