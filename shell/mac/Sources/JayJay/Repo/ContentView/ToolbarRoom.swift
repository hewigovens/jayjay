import Foundation
import Observation

/// Shares the toolbar's width so nothing overflows while a part can still give: the revset field narrows first, then the Operation Log toggle gives up its seat above the pane. Kept out of view state so a resize only invalidates the views that read it.
@MainActor @Observable
final class ToolbarRoom {
    static let revsetWidth: ClosedRange<CGFloat> = 200 ... 485
    /// The Operation Log toggle and the margins around it, which its pane's width already covers.
    static let paneToggleWidth: CGFloat = 51
    /// The window controls, the trailing items and the margins between items.
    private static let fixedWidth: CGFloat = 250

    var toolbarWidth: CGFloat = 0
    var leadingItemWidth: CGFloat = 0

    func seatsPaneToggle(paneWidth: CGFloat) -> Bool {
        leadingItemWidth > 0 && flexibleWidth - Self.revsetWidth.lowerBound >= paneWidth - Self.paneToggleWidth
    }

    func revsetWidth(paneShown: Bool, paneWidth: CGFloat) -> CGFloat {
        guard leadingItemWidth > 0 else { return Self.revsetWidth.upperBound }
        let seat = paneShown && seatsPaneToggle(paneWidth: paneWidth) ? paneWidth - Self.paneToggleWidth : 0
        return min(max(flexibleWidth - seat, Self.revsetWidth.lowerBound), Self.revsetWidth.upperBound)
    }

    private var flexibleWidth: CGFloat {
        toolbarWidth - leadingItemWidth - Self.fixedWidth
    }
}
