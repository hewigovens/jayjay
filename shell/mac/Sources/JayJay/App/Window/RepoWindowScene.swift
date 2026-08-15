import AppKit
import SwiftUI

struct RepoWindowScene: View {
    let repoPath: String
    let windowManager: RepoWindowManager
    @State private var boundPath: String

    init(repoPath: String, windowManager: RepoWindowManager) {
        self.repoPath = repoPath
        self.windowManager = windowManager
        _boundPath = State(initialValue: repoPath)
    }

    var body: some View {
        RepoWindow(repoPath: boundPath, onBoundPathChange: { boundPath = $0 })
            .frame(minWidth: 900, minHeight: 500)
            .background(WindowConfigurator { window in
                window.representedURL = URL(fileURLWithPath: boundPath)
                windowManager.repoWindowDidAppear()
            })
            .onReceive(NotificationCenter.default.publisher(for: NSWindow.willCloseNotification)) { notification in
                guard let window = notification.object as? NSWindow,
                      window.representedURL?.standardizedFileURL.path
                      == URL(fileURLWithPath: boundPath).standardizedFileURL.path
                else { return }
                windowManager.repoWindowWillClose()
            }
    }
}
