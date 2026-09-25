import AppKit
import SwiftUI

/// Actions the Repository menu can invoke on the active window.
@MainActor
protocol RepositoryMenuHandler: AnyObject {
    func showCommandPalette()
    func showUndo()
    func showBookmarkManager()
    func showOverview()
    func showNewWorkspace()
    func showPullRequestImport()
}

/// Tracks the active repo window's path, settings, and menu handler.
/// Keeps a registry of handlers per repo path so window switching works correctly.
@MainActor
@Observable
final class ActiveRepoTracker {
    static let shared = ActiveRepoTracker()

    var repoPath: String?
    var settings: AppSettings?

    /// The handler for the currently active window; an overview window binds the path-based commands only.
    var handler: RepositoryMenuHandler? {
        guard let repoPath, !keyWindowIsOverview else { return nil }
        return handlers[repoPath]?.value
    }

    private var handlers: [String: WeakRef] = [:]
    private var keyWindowIsOverview = false
    private let overviewWindows = NSHashTable<NSWindow>.weakObjects()

    private init() {
        NotificationCenter.default.addObserver(
            forName: NSWindow.didBecomeKeyNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            MainActor.assumeIsolated {
                guard let window = notification.object as? NSWindow else { return }
                if let path = window.representedURL?.path {
                    self?.repoPath = path
                    self?.keyWindowIsOverview = self?.overviewWindows.contains(window) == true
                } else if window.identifier?.rawValue == AppWindows.repoList {
                    self?.repoPath = nil
                }
            }
        }
    }

    func register(repoPath: String, settings: AppSettings, handler: RepositoryMenuHandler) {
        self.repoPath = repoPath
        self.settings = settings
        keyWindowIsOverview = false
        handlers[repoPath] = WeakRef(handler)
    }

    /// Registration can come after the window already became key, so it settles the flag and path too.
    func registerOverview(_ window: NSWindow) {
        overviewWindows.add(window)
        if window.isKeyWindow {
            keyWindowIsOverview = true
            repoPath = window.representedURL?.path ?? repoPath
        }
    }

    private struct WeakRef {
        weak var value: RepositoryMenuHandler?
        init(_ value: RepositoryMenuHandler) {
            self.value = value
        }
    }
}
