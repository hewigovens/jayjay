import Foundation
import Observation

/// Shares the window's width so nothing overflows while a part can still give: the Operation Log pane narrows below the user's width, the revset field narrows, then the Operation Log toggle gives up its seat above the pane. Kept out of view state so a resize only invalidates the views that read it.
@MainActor @Observable
final class ToolbarRoom {
    static let revsetWidth: ClosedRange<CGFloat> = 200 ... 485
    /// The Operation Log toggle and the margins around it, which its pane's width already covers.
    static let paneToggleWidth: CGFloat = 51
    /// The window controls, trailing items, and the padding and margins AppKit adds around toolbar items.
    private static let fixedWidth: CGFloat = 282

    var toolbarWidth: CGFloat = 0
    var leadingItemWidth: CGFloat = 0
    var isSidebarHidden = false
    /// Only a drag changes it; a narrow window draws the pane narrower without forgetting it.
    var preferredOperationLogWidth = PaneLayout.operationLogWidth()

    var operationLogRange: ClosedRange<CGFloat> {
        guard toolbarWidth > 0 else { return PaneLayout.operationLog }
        return PaneLayout.operationLogRange(windowWidth: toolbarWidth, sidebarShown: !isSidebarHidden)
    }

    var operationLogWidth: CGFloat {
        min(max(preferredOperationLogWidth, operationLogRange.lowerBound), operationLogRange.upperBound)
    }

    var seatsPaneToggle: Bool {
        leadingItemWidth > 0 && flexibleWidth - Self.revsetWidth.lowerBound >= operationLogWidth - Self.paneToggleWidth
    }

    func revsetWidth(paneShown: Bool) -> CGFloat {
        guard leadingItemWidth > 0 else { return Self.revsetWidth.upperBound }
        let seat = paneShown && seatsPaneToggle ? operationLogWidth - Self.paneToggleWidth : 0
        return min(max(flexibleWidth - seat, Self.revsetWidth.lowerBound), Self.revsetWidth.upperBound)
    }

    private var flexibleWidth: CGFloat {
        toolbarWidth - leadingItemWidth - Self.fixedWidth
    }
}
