import JayJayCore
import SwiftUI

let dagRowLeadingPadding: CGFloat = 4
let dagRowVerticalPadding: CGFloat = 8
let dagNodeCenterY: CGFloat = 12
let dagIndirectEdgeStroke = StrokeStyle(lineWidth: 1, dash: [3, 3])
let dagSolidStroke = StrokeStyle(lineWidth: 1)
let dagMissingEdgeStroke = StrokeStyle(lineWidth: 1, lineCap: .round, dash: [2, 2])
let dagGraphCornerRadius: CGFloat = 6
let dagLinkCenterFraction: CGFloat = 0.45

/// Thin Swift wrapper over `jayjay_core::dag::DagLayout` (via uniffi): the renderer-computed row shapes, indexed by commit id so rows never need to consult their neighbors to draw.
struct DAGLayout: Sendable {
    let rows: [DagRowShape]
    let logicalColumnCount: Int
    private let rowsByCommitId: [String: DagRowShape]

    init(entries: [GraphEntry]) {
        let data = computeDagLayout(entries: entries)
        rows = data.rows
        logicalColumnCount = max(1, Int(data.logicalColumnCount))
        rowsByCommitId = Dictionary(rows.map { ($0.commitId, $0) }, uniquingKeysWith: { first, _ in first })
    }

    func row(for commitId: String) -> DagRowShape? {
        rowsByCommitId[commitId]
    }
}
