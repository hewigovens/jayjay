import Foundation

/// Maps logical DAG columns and row bands to pixel positions for a given sidebar width.
///
/// One value is built per visible width and shared by every row, so column pitch never drifts row to row. Compression changes pixel geometry only — logical columns, and therefore node identity per column, never change.
struct DAGGeometry: Equatable {
    static let preferredLanePitch: CGFloat = 12
    static let absoluteGraphMaxWidth: CGFloat = 192
    static let maxSidebarFraction: CGFloat = 0.45
    static let horizontalPadding: CGFloat = 8
    static let preferredNodeRadius: CGFloat = 4
    static let minimumNodeRadius: CGFloat = 1.5

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
        let preferredWidth = Self.horizontalPadding + CGFloat(columns) * Self.preferredLanePitch
        // Never compress below one full lane pitch, even at a pathologically narrow sidebar.
        let widthFloor = Self.horizontalPadding + Self.preferredLanePitch
        graphWidth = max(widthFloor, min(preferredWidth, widthBudget))
        lanePitch = (graphWidth - Self.horizontalPadding) / CGFloat(columns)
        // Full radius at the preferred pitch; shrink proportionally only once the sidebar compresses lanes below it.
        nodeRadius = min(
            Self.preferredNodeRadius,
            max(Self.minimumNodeRadius, Self.preferredNodeRadius * lanePitch / Self.preferredLanePitch)
        )
    }

    func xPosition(forColumn column: Int) -> CGFloat {
        dagRowLeadingPadding + CGFloat(column) * lanePitch + lanePitch / 2
    }

    func linkTopY(forColumn column: Int, nodeColumn: Int, nodeY: CGFloat, nodeRadius: CGFloat) -> CGFloat {
        column == nodeColumn ? nodeY + nodeRadius : nodeY
    }
}
