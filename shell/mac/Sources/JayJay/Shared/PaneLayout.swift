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

    static let evologSnapshots: ClosedRange<CGFloat> = 240 ... 360
    static let evologFilenames: ClosedRange<CGFloat> = 180 ... 320
    static let evologSnapshotsDefault: CGFloat = 280
    static let evologFilenamesDefault: CGFloat = 220
    /// Evolog nests snapshots+filenames+preview inside the detail pane's own width, which already excludes the sidebar; the app-wide `previewMin` would leave near-zero drag room, so this three-way split gets a smaller reservation.
    static let evologPreviewMin: CGFloat = 260

    static func evologSnapshotsRange(contentWidth: CGFloat) -> ClosedRange<CGFloat> {
        let room = contentWidth - 2 * dividerWidth - evologFilenames.lowerBound - evologPreviewMin
        return evologSnapshots.lowerBound ... min(evologSnapshots.upperBound, max(evologSnapshots.lowerBound, room))
    }

    static func evologFilenamesRange(detailWidth: CGFloat) -> ClosedRange<CGFloat> {
        let room = detailWidth - dividerWidth - evologPreviewMin
        return evologFilenames.lowerBound ... min(evologFilenames.upperBound, max(evologFilenames.lowerBound, room))
    }
}
