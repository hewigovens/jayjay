import SwiftUI

/// A pane edge's hairline, drawn up through the toolbar like the system's column separators.
struct PaneSeparator: View {
    let paneEdge: HorizontalAlignment
    @Environment(\.displayScale) private var displayScale

    var body: some View {
        Rectangle()
            .fill(Color(nsColor: .separatorColor))
            .frame(width: 1 / displayScale)
            .frame(width: PaneLayout.dividerWidth, alignment: Alignment(horizontal: paneEdge, vertical: .center))
            .ignoresSafeArea(.container, edges: .top)
    }
}
