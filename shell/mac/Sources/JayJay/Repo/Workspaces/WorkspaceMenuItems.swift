import AppKit
import JayJayCore
import SwiftUI

struct WorkspaceMenuItems: View {
    let workspace: WorkspaceInfo
    let onOpen: () -> Void

    var body: some View {
        if !workspace.isCurrent, workspace.isPathResolved {
            Button("Open in New Window", action: onOpen)
        }
        Button("Copy Workspace Name") {
            copy(workspace.name)
        }
        if workspace.isPathResolved {
            Button("Copy Path") {
                copy(workspace.path)
            }
        }
    }

    private func copy(_ value: String) {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(value, forType: .string)
    }
}
