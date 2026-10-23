import AppKit
import JayJayCore
import SwiftUI

struct WorkspaceMenuItems: View {
    let workspace: WorkspaceInfo
    let onFilter: () -> Void
    let onOpen: () -> Void

    var body: some View {
        Button(Localized.Menu.filterByThisWorkspace, action: onFilter)
        Divider()
        if !workspace.isCurrent, workspace.isPathResolved {
            Button(Localized.Menu.openInNewWindow, action: onOpen)
        }
        Button(Localized.Menu.copyWorkspaceName) {
            copy(workspace.name)
        }
        if workspace.isPathResolved {
            Button(Localized.Menu.copyPath) {
                copy(workspace.path)
            }
        }
    }

    private func copy(_ value: String) {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(value, forType: .string)
    }
}
