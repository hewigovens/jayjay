import SwiftUI

/// The Operation Log toggle followed by room as wide as the shown part of the pane, so the toggle sits at the pane's leading edge.
struct OperationLogToolbarSlot<Toggle: View>: View {
    let reveal: PaneReveal
    let room: ToolbarRoom
    @ViewBuilder let toggle: () -> Toggle

    var body: some View {
        let progress = room.seatsPaneToggle ? reveal.progress : 0
        HStack(spacing: 0) {
            toggle()
                // Matches the spacing of the system inspector's toolbar.
                .padding(.leading, 5 + 7.5 * progress)
            Color.clear.frame(width: max(0, room.operationLogWidth * progress - ToolbarRoom.paneToggleWidth), height: 1)
        }
    }
}
