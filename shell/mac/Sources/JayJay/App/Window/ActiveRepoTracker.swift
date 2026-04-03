import AppKit
import SwiftUI

/// Actions the Repository menu can invoke on the active window.
@MainActor
protocol RepositoryMenuHandler: AnyObject {
    func showCommandPalette()
    func showUndo()
    func showBookmarkManager()
    func showNewWorkspace()
}

/// Tracks the active repo window's path, settings, and menu handler.
/// Menu commands call methods on `handler` directly — no NotificationCenter.
@MainActor
@Observable
final class ActiveRepoTracker {
    static let shared = ActiveRepoTracker()

    var repoPath: String?
    var settings: AppSettings?
    weak var handler: RepositoryMenuHandler?

    private init() {
        NotificationCenter.default.addObserver(
            forName: NSWindow.didBecomeKeyNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            guard let window = notification.object as? NSWindow else { return }
            if let path = window.representedURL?.path {
                self?.repoPath = path
            }
        }
    }

    func register(repoPath: String, settings: AppSettings, handler: RepositoryMenuHandler) {
        self.repoPath = repoPath
        self.settings = settings
        self.handler = handler
        NSApp.keyWindow?.representedURL = URL(fileURLWithPath: repoPath)
    }
}
