import Foundation

/// Maps logical DAG columns and row bands to pixel positions for a given sidebar width.
///
/// One value is built per visible width and shared by every row, so column pitch never drifts row to row. Protected topology may make the graph wider than its normal budget, but lanes never compress below a legible pitch.
struct DAGGeometry: Equatable {
    static let preferredLanePitch: CGFloat = 12
    static let minimumLegibleLanePitch: CGFloat = 10
    static let absoluteGraphMaxWidth: CGFloat = 192
    static let maxSidebarFraction: CGFloat = 0.45
    static let horizontalPadding: CGFloat = 8
    static let preferredNodeRadius: CGFloat = 4

    let logicalColumnCount: Int
    let lanePitch: CGFloat
    let nodeRadius: CGFloat
    let graphWidth: CGFloat

    init(logicalColumnCount: Int, availableSidebarWidth: CGFloat) {
        let columns = max(1, logicalColumnCount)
        self.logicalColumnCount = columns

        let widthBudget = min(
            Self.absoluteGraphMaxWidth,
            availableSidebarWidth * Self.maxSidebarFraction
        )
        let compressedPitch = (widthBudget - Self.horizontalPadding) / CGFloat(columns)
        lanePitch = min(Self.preferredLanePitch, max(Self.minimumLegibleLanePitch, compressedPitch))
        graphWidth = Self.horizontalPadding + CGFloat(columns) * lanePitch
        nodeRadius = Self.preferredNodeRadius
    }

    func xPosition(forColumn column: Int) -> CGFloat {
        dagRowLeadingPadding + CGFloat(column) * lanePitch + lanePitch / 2
    }

    func linkTopY(forColumn column: Int, nodeColumn: Int, nodeY: CGFloat, nodeRadius: CGFloat) -> CGFloat {
        column == nodeColumn ? nodeY + nodeRadius : nodeY
    }
}
