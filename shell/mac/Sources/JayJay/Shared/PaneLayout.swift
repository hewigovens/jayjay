import CoreGraphics

/// The sidebar keeps room for a minimum file column and preview, the file column for a minimum preview; GPUI applies the same rule.
enum PaneLayout {
    static let sidebar: ClosedRange<CGFloat> = 240 ... 600
    /// Shared width range and default for any side pane sized like the change-detail file column: the file column itself, and the evolog snapshot/filename panes, which persist the same preference.
    static let secondaryPaneWidth: ClosedRange<CGFloat> = 220 ... 480
    static let secondaryPaneWidthDefault: CGFloat = 260
    static let previewMin: CGFloat = 420
    static let dividerWidth: CGFloat = 1

    static func sidebarRange(windowWidth: CGFloat) -> ClosedRange<CGFloat> {
        let room = windowWidth - 2 * dividerWidth - secondaryPaneWidth.lowerBound - previewMin
        return sidebar.lowerBound ... min(sidebar.upperBound, max(sidebar.lowerBound, room))
    }

    static func secondaryPaneWidthRange(detailWidth: CGFloat) -> ClosedRange<CGFloat> {
        let room = detailWidth - dividerWidth - previewMin
        return secondaryPaneWidth.lowerBound ... min(secondaryPaneWidth.upperBound, max(secondaryPaneWidth.lowerBound, room))
    }

    /// Evolog nests snapshots+filenames+preview inside the detail pane's own width, which already excludes the sidebar; the app-wide `previewMin` would leave near-zero drag room, so this three-way split gets a smaller reservation.
    static let evologPreviewMin: CGFloat = 260

    static func evologSnapshotsRange(contentWidth: CGFloat) -> ClosedRange<CGFloat> {
        let room = contentWidth - 2 * dividerWidth - secondaryPaneWidth.lowerBound - evologPreviewMin
        return secondaryPaneWidth.lowerBound ... min(secondaryPaneWidth.upperBound, max(secondaryPaneWidth.lowerBound, room))
    }

    static func evologFilenamesRange(detailWidth: CGFloat) -> ClosedRange<CGFloat> {
        let room = detailWidth - dividerWidth - evologPreviewMin
        return secondaryPaneWidth.lowerBound ... min(secondaryPaneWidth.upperBound, max(secondaryPaneWidth.lowerBound, room))
    }
}
