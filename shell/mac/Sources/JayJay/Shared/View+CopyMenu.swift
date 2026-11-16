import AppKit
import SwiftUI

extension View {
    /// Right-click Copy for an identifier, the Mac convention, instead of a copy button beside every value.
    func copyContextMenu(_ value: String, label: String) -> some View {
        contextMenu {
            Button(label) {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(value, forType: .string)
            }
        }
    }
}
