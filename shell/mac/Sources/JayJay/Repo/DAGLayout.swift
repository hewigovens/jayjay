import JayJayCore
import SwiftUI

let laneWidth: CGFloat = 16
let nodeRadius: CGFloat = 4
let dagRowLeadingPadding: CGFloat = 4
let dagRowVerticalPadding: CGFloat = 8
let dagNodeCenterY: CGFloat = 12

/// Pre-computes which lane (column) each commit occupies.
struct DAGLayout {
    /// Lane index for each commit ID.
    let lanes: [String: Int]
    /// Total number of active lanes at each row.
    let activeLanesPerRow: [Int]
    /// The exact lane indices that continue through each row.
    let activeLaneIndicesPerRow: [[Int]]

    init(entries: [GraphEntry]) {
        var lanes: [String: Int] = [:]
        var activeLanes: [String?] = []
        var activeCounts: [Int] = []
        var activeLaneIndices: [[Int]] = []

        for entry in entries {
            let cid = entry.change.commitId

            if lanes[cid] == nil {
                if let existing = activeLanes.firstIndex(of: cid) {
                    lanes[cid] = existing
                } else {
                    lanes[cid] = Self.assignLane(for: cid, in: &activeLanes)
                }
            }

            guard let myLane = lanes[cid] else { continue }
            if myLane < activeLanes.count { activeLanes[myLane] = nil }

            for edge in entry.edges where edge.edgeType != .missing {
                if lanes[edge.target] == nil {
                    lanes[edge.target] = Self.assignLane(
                        for: edge.target, in: &activeLanes, preferring: myLane
                    )
                }
            }

            activeCounts.append(activeLanes.count)
            activeLaneIndices.append(
                activeLanes.enumerated().compactMap { lane, commitId in
                    commitId == nil ? nil : lane
                }
            )
        }

        self.lanes = lanes
        activeLanesPerRow = activeCounts
        activeLaneIndicesPerRow = activeLaneIndices
    }

    func lane(for commitId: String) -> Int {
        lanes[commitId] ?? 0
    }

    func maxLanes() -> Int {
        activeLanesPerRow.max() ?? 1
    }

    func activeLaneIndices(at row: Int) -> [Int] {
        guard activeLaneIndicesPerRow.indices.contains(row) else { return [] }
        return activeLaneIndicesPerRow[row]
    }

    /// Assign a lane for `commitId`, reusing `preferring` if free, else first free slot, else new.
    private static func assignLane(
        for commitId: String, in activeLanes: inout [String?], preferring: Int? = nil
    ) -> Int {
        if let preferred = preferring, preferred < activeLanes.count, activeLanes[preferred] == nil {
            activeLanes[preferred] = commitId
            return preferred
        }
        if let free = activeLanes.firstIndex(of: nil) {
            activeLanes[free] = commitId
            return free
        }
        let lane = activeLanes.count
        activeLanes.append(commitId)
        return lane
    }
}
