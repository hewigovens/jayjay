import AppKit
import SwiftUI

struct RepoWindowScene: View {
    @Binding var repoPath: String
    let windowManager: RepoWindowManager
    @State private var windowNumber: Int?

    var body: some View {
        RepoWindow(repoPath: repoPath, windowNumber: windowNumber, onSwitchWorkspace: { repoPath = $0 })
            .frame(minWidth: 900, minHeight: 500)
            .background(WindowFramePersistence(key: AppWindows.repo))
            .background(WindowConfigurator { window in
                window.representedURL = URL(fileURLWithPath: repoPath)
                windowNumber = window.windowNumber
                windowManager.repoWindowDidAppear()
            })
            .onChange(of: repoPath) { _, path in
                guard let windowNumber, let window = NSApp.windows.first(where: { $0.windowNumber == windowNumber }) else { return }
                window.representedURL = URL(fileURLWithPath: path)
                windowManager.refreshOpenRepoPaths()
                ActiveRepoTracker.shared.repoPath = path
            }
            .onReceive(NotificationCenter.default.publisher(for: NSWindow.willCloseNotification)) { notification in
                guard let closing = notification.object as? NSWindow, closing.windowNumber == windowNumber else { return }
                windowManager.workspaceDrafts.clear(in: closing.windowNumber)
                windowManager.repoWindowWillClose(at: repoPath)
            }
    }
}
