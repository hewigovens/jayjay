import JayJayCore
import SwiftUI

struct FileCollapseButtons: View {
    /// Off in the repository window, where these chords may belong to a repository action.
    var bindsShortcuts = true
    let onExpandAll: () -> Void
    let onCollapseAll: () -> Void

    var body: some View {
        button(Localized.Repo.expandAll, systemImage: "rectangle.expand.vertical", action: onExpandAll)
            .keyboardShortcut(bindsShortcuts ? AppShortcut.expandAllFiles : nil)
            .accessibilityIdentifier(AID.Diff.expandAllFiles)
        button(Localized.Repo.collapseAll, systemImage: "rectangle.compress.vertical", action: onCollapseAll)
            .keyboardShortcut(bindsShortcuts ? AppShortcut.collapseAllFiles : nil)
            .accessibilityIdentifier(AID.Diff.collapseAllFiles)
    }

    private func button(_ title: String, systemImage: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            HStack(spacing: 4) {
                Image(systemName: systemImage)
                Text(title)
            }
        }
    }
}
