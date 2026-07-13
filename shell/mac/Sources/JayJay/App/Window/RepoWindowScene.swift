import AppKit
import SwiftUI

struct RepoWindowScene: View {
    let repoPath: String
    let windowManager: RepoWindowManager

    var body: some View {
        RepoWindow(repoPath: repoPath)
            .frame(minWidth: 900, minHeight: 500)
            .background(WindowConfigurator { window in
                window.representedURL = URL(fileURLWithPath: repoPath)
                windowManager.repoWindowDidAppear()
            })
            .onReceive(NotificationCenter.default.publisher(for: NSWindow.willCloseNotification)) { notification in
                guard let window = notification.object as? NSWindow,
                      window.representedURL?.standardizedFileURL.path
                      == URL(fileURLWithPath: repoPath).standardizedFileURL.path
                else { return }
                windowManager.repoWindowWillClose()
            }
    }
}
