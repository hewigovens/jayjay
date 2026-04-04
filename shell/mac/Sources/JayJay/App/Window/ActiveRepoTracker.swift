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
/// Keeps a registry of handlers per repo path so window switching works correctly.
@MainActor
@Observable
final class ActiveRepoTracker {
    static let shared = ActiveRepoTracker()

    var repoPath: String?
    var settings: AppSettings?

    /// The handler for the currently active window.
    var handler: RepositoryMenuHandler? {
        guard let repoPath else { return nil }
        return handlers[repoPath]?.value
    }

    private var handlers: [String: WeakRef] = [:]

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
        handlers[repoPath] = WeakRef(handler)
        NSApp.keyWindow?.representedURL = URL(fileURLWithPath: repoPath)
    }

    private struct WeakRef {
        weak var value: RepositoryMenuHandler?
        init(_ value: RepositoryMenuHandler) {
            self.value = value
        }
    }
}
