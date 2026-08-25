import CoreGraphics

/// The sidebar keeps room for a minimum file column and preview, the file column for a minimum preview; GPUI applies the same rule.
enum PaneLayout {
    static let sidebar: ClosedRange<CGFloat> = 240 ... 600
    static let fileColumn: ClosedRange<CGFloat> = 220 ... 480
    static let previewMin: CGFloat = 420
    static let dividerWidth: CGFloat = 1

    static func sidebarRange(windowWidth: CGFloat) -> ClosedRange<CGFloat> {
        let room = windowWidth - 2 * dividerWidth - fileColumn.lowerBound - previewMin
        return sidebar.lowerBound ... min(sidebar.upperBound, max(sidebar.lowerBound, room))
    }

    static func fileColumnRange(detailWidth: CGFloat) -> ClosedRange<CGFloat> {
        let room = detailWidth - dividerWidth - previewMin
        return fileColumn.lowerBound ... min(fileColumn.upperBound, max(fileColumn.lowerBound, room))
    }
}
