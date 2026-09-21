import CoreGraphics

/// The sidebar keeps room for a minimum secondary pane and preview, the secondary pane for a minimum preview; GPUI applies the same rule.
enum PaneLayout {
    static let sidebar: ClosedRange<CGFloat> = 240 ... 600
    static let secondaryPane: ClosedRange<CGFloat> = 220 ... 480
    static let secondaryPaneDefault: CGFloat = 300
    static let previewMin: CGFloat = 420
    static let dividerWidth: CGFloat = 1
    static let headerHeight: CGFloat = 44
    /// Shared by the detail column's metadata block and the diff header so the two line up over the full-bleed diff.
    static let detailInset: CGFloat = 20

    static func fileRowHeight(baseFontSize: Double) -> CGFloat {
        46 * (baseFontSize / 12)
    }

    static func sidebarRange(windowWidth: CGFloat) -> ClosedRange<CGFloat> {
        sidebar.fitted(in: windowWidth - 2 * dividerWidth - secondaryPane.lowerBound - previewMin)
    }

    static func secondaryPaneRange(detailWidth: CGFloat) -> ClosedRange<CGFloat> {
        secondaryPane.fitted(in: detailWidth - dividerWidth - previewMin)
    }

    /// Evolog splits the detail three ways, so each pane only keeps the secondary-pane minimum.
    static func evologEntryListRange(detailWidth: CGFloat) -> ClosedRange<CGFloat> {
        secondaryPane.fitted(in: detailWidth - 2 * dividerWidth - 2 * secondaryPane.lowerBound)
    }

    static func evologFileListRange(paneWidth: CGFloat) -> ClosedRange<CGFloat> {
        secondaryPane.fitted(in: paneWidth - dividerWidth - secondaryPane.lowerBound)
    }
}

private extension ClosedRange<CGFloat> {
    func fitted(in room: CGFloat) -> ClosedRange<CGFloat> {
        lowerBound ... Swift.min(upperBound, Swift.max(lowerBound, room))
    }
}
