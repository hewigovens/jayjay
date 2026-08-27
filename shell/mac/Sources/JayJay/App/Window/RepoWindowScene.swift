import AppKit
import SwiftUI

struct RepoWindowScene: View {
    let repoPath: String
    let windowManager: RepoWindowManager
    @State private var windowNumber: Int?

    var body: some View {
        RepoWindow(repoPath: repoPath)
            .frame(minWidth: 900, minHeight: 500)
            .background(WindowFramePersistence(key: AppWindows.repo))
            .background(WindowConfigurator { window in
                window.representedURL = URL(fileURLWithPath: repoPath)
                windowNumber = window.windowNumber
                windowManager.repoWindowDidAppear()
            })
            .onReceive(NotificationCenter.default.publisher(for: NSWindow.willCloseNotification)) { notification in
                guard let closing = notification.object as? NSWindow, closing.windowNumber == windowNumber else { return }
                windowManager.repoWindowWillClose(at: repoPath)
            }
    }
}
